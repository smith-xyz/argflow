use anyhow::{Context as AnyhowContext, Result};
use argflow::classifier::RulesClassifier;
use argflow::cli::{self, OutputFormat};
use argflow::discovery::cache::DiscoveryCache;
use argflow::discovery::filter::ImportFileFilter;
use argflow::discovery::languages::go::{GoImportFilter, GoPackageLoader};
use argflow::discovery::languages::javascript::{JavaScriptImportFilter, JavaScriptPackageLoader};
use argflow::discovery::languages::python::{PythonImportFilter, PythonPackageLoader};
use argflow::discovery::languages::rust::{RustImportFilter, RustPackageLoader};
use argflow::discovery::loader::PackageLoader;
use argflow::logging::{self, Verbosity};
use argflow::output::OutputFormatter;
use argflow::presets;
use argflow::scanner::{ScanResult, Scanner};
use clap::Parser;
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{debug, info, trace, warn};

struct ScanContext<'a> {
    scanner: &'a Scanner,
    classifier: &'a RulesClassifier,
    output_format: OutputFormat,
    output_file: Option<&'a PathBuf>,
    preset_paths: &'a [PathBuf],
}

fn main() -> Result<()> {
    let args = cli::Args::parse();

    let verbosity = Verbosity::from_flags(args.verbose, args.quiet);
    logging::init(verbosity);

    info!(path = %args.path.display(), "starting argflow analysis");
    debug!(?args, "parsed command line arguments");

    args.validate().context("Invalid arguments")?;

    let language = args
        .language
        .or_else(|| {
            if args.path.is_file() {
                let detected = cli::detect_language(&args.path);
                if let Some(lang) = detected {
                    debug!(language = lang.as_str(), "auto-detected language");
                }
                detected
            } else {
                None
            }
        })
        .context("Could not detect language. Please specify --language")?;

    info!(language = language.as_str(), "using language");

    // Load preset paths for both classifier and filters
    let preset_paths = get_preset_paths(&args)?;

    // Load classifier from presets or custom rules
    let classifier = load_classifier(&args, &preset_paths)?;
    debug!(
        classifications = classifier.classification_count(),
        mappings = classifier.mapping_count(),
        "classifier loaded"
    );

    // Create scanner with classifier mappings and struct field detection
    // Only calls with explicit API mappings will be detected (high precision)
    let scanner = Scanner::with_mappings_and_struct_fields(
        classifier.get_mappings().clone(),
        classifier.get_struct_fields().clone(),
    );
    trace!("scanner initialized with classifier mappings and struct fields");

    let ctx = ScanContext {
        scanner: &scanner,
        classifier: &classifier,
        output_format: args.format,
        output_file: args.output_file.as_ref(),
        preset_paths: &preset_paths,
    };

    if args.path.is_dir() {
        scan_directory(&args.path, language, &ctx, args.include_deps)?;
    } else {
        scan_file(&args.path, language, &ctx)?;
    }

    Ok(())
}

fn get_preset_paths(args: &cli::Args) -> Result<Vec<PathBuf>> {
    if args.preset.is_empty() && args.rules.is_none() {
        anyhow::bail!(
            "No preset or rules specified. Use --preset <name> (e.g., --preset crypto) or --rules <path>"
        );
    }

    if !args.preset.is_empty() {
        info!(presets = ?args.preset, "loading presets");
        return presets::load_presets(&args.preset);
    }

    // Custom rules file - return empty preset paths (classifier loads from rules file)
    Ok(vec![])
}

fn load_classifier(args: &cli::Args, preset_paths: &[PathBuf]) -> Result<RulesClassifier> {
    if let Some(ref rules_path) = args.rules {
        info!(rules = %rules_path.display(), "loading custom rules");
        return RulesClassifier::from_file(rules_path)
            .map_err(|e| anyhow::anyhow!("Failed to load custom rules: {e}"));
    }

    if let Some(preset_path) = preset_paths.first() {
        return RulesClassifier::from_preset_path(preset_path)
            .map_err(|e| anyhow::anyhow!("Failed to load preset: {e}"));
    }

    RulesClassifier::from_bundled()
        .map_err(|e| anyhow::anyhow!("Failed to load classifier rules: {e}"))
}

fn scan_file(path: &Path, language: cli::Language, ctx: &ScanContext) -> Result<()> {
    debug!(file = %path.display(), "scanning file");

    let source = std::fs::read_to_string(path).context("Failed to read file")?;
    trace!(bytes = source.len(), "read source file");

    let tree = parse_source(&source, language)?;
    trace!("parsed source into AST");

    let result = ctx.scanner.scan_tree(
        &tree,
        source.as_bytes(),
        &path.to_string_lossy(),
        language.as_str(),
    );

    info!(calls = result.call_count(), "scan complete");

    output_results(
        &[result],
        ctx.classifier,
        ctx.output_format,
        ctx.output_file,
    )?;
    Ok(())
}

fn scan_directory(
    path: &Path,
    language: cli::Language,
    ctx: &ScanContext,
    include_deps: bool,
) -> Result<()> {
    debug!(directory = %path.display(), include_deps, "scanning directory");

    if ctx.preset_paths.is_empty() {
        anyhow::bail!(
            "Directory scanning requires a preset for import filtering. \
             Use --preset <name> (e.g., --preset crypto). \
             For custom rules, scan individual files instead."
        );
    }

    match language {
        cli::Language::Go => {
            let loader = GoPackageLoader;
            let filter = GoImportFilter::new(ctx.preset_paths)
                .context("Failed to create Go import filter")?;
            scan_with_loader_and_filter(path, language, ctx, include_deps, &loader, &filter)?;
        }
        cli::Language::Python => {
            let loader = PythonPackageLoader;
            let filter = PythonImportFilter::new(ctx.preset_paths)
                .context("Failed to create Python import filter")?;
            scan_with_loader_and_filter(path, language, ctx, include_deps, &loader, &filter)?;
        }
        cli::Language::Javascript | cli::Language::Typescript => {
            let loader = JavaScriptPackageLoader;
            let filter = JavaScriptImportFilter::new(ctx.preset_paths)
                .context("Failed to create JavaScript import filter")?;
            scan_with_loader_and_filter(path, language, ctx, include_deps, &loader, &filter)?;
        }
        cli::Language::Rust => {
            let loader = RustPackageLoader;
            let filter = RustImportFilter::new(ctx.preset_paths)
                .context("Failed to create Rust import filter")?;
            scan_with_loader_and_filter(path, language, ctx, include_deps, &loader, &filter)?;
        }
    }

    Ok(())
}

fn scan_with_loader_and_filter(
    path: &Path,
    language: cli::Language,
    ctx: &ScanContext,
    include_deps: bool,
    loader: &dyn PackageLoader,
    filter: &dyn ImportFileFilter,
) -> Result<()> {
    let mut cache = DiscoveryCache::default();

    // Discover user code files
    info!("discovering user code files");
    let mut all_files = loader
        .load_user_code(path)
        .context("Failed to discover user code files")?;
    info!(count = all_files.len(), "found user code files");

    // Optionally include dependency files
    if include_deps {
        info!("discovering dependency files");
        match loader.load_dependencies(path, &mut cache) {
            Ok(dep_files) => {
                info!(count = dep_files.len(), "found dependency files");
                all_files.extend(dep_files);
            }
            Err(e) => {
                warn!(error = %e, "failed to load dependencies, continuing with user code only");
            }
        }
    }

    info!(total = all_files.len(), "total files to scan");

    info!("filtering for matching imports");
    let matched_files: Vec<_> = all_files
        .into_iter()
        .filter_map(|file| {
            filter
                .has_matching_imports(&file.path)
                .ok()
                .and_then(|has_match| has_match.then_some(file))
        })
        .collect();
    info!(count = matched_files.len(), "found files with matching imports");

    let mut results = Vec::new();
    for file in &matched_files {
        trace!(file = %file.path.display(), "scanning file");
        match std::fs::read_to_string(&file.path) {
            Ok(source) => {
                if let Ok(tree) = parse_source(&source, language) {
                    let result = ctx.scanner.scan_tree(
                        &tree,
                        source.as_bytes(),
                        &file.path.to_string_lossy(),
                        language.as_str(),
                    );
                    if result.call_count() > 0 {
                        debug!(
                            file = %file.path.display(),
                            calls = result.call_count(),
                            "found matching calls"
                        );
                        results.push(result);
                    }
                }
            }
            Err(e) => warn!(file = %file.path.display(), error = %e, "failed to read file"),
        }
    }

    let total_calls: usize = results.iter().map(|r| r.call_count()).sum();
    info!(files = results.len(), calls = total_calls, "scan complete");

    output_results(&results, ctx.classifier, ctx.output_format, ctx.output_file)?;
    Ok(())
}

fn parse_source(source: &str, language: cli::Language) -> Result<tree_sitter::Tree> {
    let mut parser = tree_sitter::Parser::new();

    let ts_language = match language {
        cli::Language::Go => tree_sitter_go::LANGUAGE.into(),
        cli::Language::Python => tree_sitter_python::LANGUAGE.into(),
        cli::Language::Rust => tree_sitter_rust::LANGUAGE.into(),
        cli::Language::Javascript => tree_sitter_javascript::LANGUAGE.into(),
        cli::Language::Typescript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
    };

    parser
        .set_language(&ts_language)
        .context("Failed to set parser language")?;

    parser
        .parse(source, None)
        .context("Failed to parse source code")
}

fn output_results(
    results: &[ScanResult],
    classifier: &RulesClassifier,
    format: OutputFormat,
    output_file: Option<&PathBuf>,
) -> Result<()> {
    let output = OutputFormatter::format(results, classifier, format)?;

    match output_file {
        Some(path) => {
            let mut file = std::fs::File::create(path)
                .with_context(|| format!("Failed to create output file: {}", path.display()))?;
            file.write_all(output.as_bytes())
                .with_context(|| format!("Failed to write to output file: {}", path.display()))?;
            info!(path = %path.display(), "wrote output to file");
        }
        None => {
            println!("{output}");
        }
    }

    Ok(())
}
