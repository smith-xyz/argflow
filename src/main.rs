use anyhow::{Context as AnyhowContext, Result};
use clap::Parser;
use crypto_extractor_core::classifier::RulesClassifier;
use crypto_extractor_core::cli::{self, OutputFormat};
use crypto_extractor_core::discovery::cache::DiscoveryCache;
use crypto_extractor_core::discovery::filter::CryptoFileFilter;
use crypto_extractor_core::discovery::languages::go::{GoCryptoFilter, GoPackageLoader};
use crypto_extractor_core::discovery::loader::PackageLoader;
use crypto_extractor_core::logging::{self, Verbosity};
use crypto_extractor_core::output::OutputFormatter;
use crypto_extractor_core::scanner::{ScanResult, Scanner};
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{debug, info, trace, warn};

fn main() -> Result<()> {
    let args = cli::Args::parse();

    let verbosity = Verbosity::from_flags(args.verbose, args.quiet);
    logging::init(verbosity);

    info!(path = %args.path.display(), "starting crypto extraction");
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

    let classifier = RulesClassifier::from_bundled()
        .map_err(|e| anyhow::anyhow!("Failed to load classifier rules: {e}"))?;
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

    if args.path.is_dir() {
        scan_directory(
            &args.path,
            language,
            &scanner,
            &classifier,
            args.format,
            args.output_file.as_ref(),
            args.include_deps,
        )?;
    } else {
        scan_file(
            &args.path,
            language,
            &scanner,
            &classifier,
            args.format,
            args.output_file.as_ref(),
        )?;
    }

    Ok(())
}

fn scan_file(
    path: &Path,
    language: cli::Language,
    scanner: &Scanner,
    classifier: &RulesClassifier,
    output_format: OutputFormat,
    output_file: Option<&PathBuf>,
) -> Result<()> {
    debug!(file = %path.display(), "scanning file");

    let source = std::fs::read_to_string(path).context("Failed to read file")?;
    trace!(bytes = source.len(), "read source file");

    let tree = parse_source(&source, language)?;
    trace!("parsed source into AST");

    let result = scanner.scan_tree(
        &tree,
        source.as_bytes(),
        &path.to_string_lossy(),
        language.as_str(),
    );

    info!(calls = result.call_count(), "scan complete");

    output_results(&[result], classifier, output_format, output_file)?;
    Ok(())
}

fn scan_directory(
    path: &Path,
    language: cli::Language,
    scanner: &Scanner,
    classifier: &RulesClassifier,
    output_format: OutputFormat,
    output_file: Option<&PathBuf>,
    include_deps: bool,
) -> Result<()> {
    debug!(directory = %path.display(), include_deps, "scanning directory");

    match language {
        cli::Language::Go => {
            let loader = GoPackageLoader;
            let filter = GoCryptoFilter;
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

            info!("filtering for crypto usage");
            let crypto_files: Vec<_> = all_files
                .into_iter()
                .filter_map(|file| {
                    filter
                        .has_crypto_usage(&file.path)
                        .ok()
                        .and_then(|has_crypto| has_crypto.then_some(file))
                })
                .collect();
            info!(count = crypto_files.len(), "found files with crypto usage");

            let mut results = Vec::new();
            for file in &crypto_files {
                trace!(file = %file.path.display(), "scanning file");
                match std::fs::read_to_string(&file.path) {
                    Ok(source) => {
                        if let Ok(tree) = parse_source(&source, language) {
                            let result = scanner.scan_tree(
                                &tree,
                                source.as_bytes(),
                                &file.path.to_string_lossy(),
                                language.as_str(),
                            );
                            if result.call_count() > 0 {
                                debug!(
                                    file = %file.path.display(),
                                    calls = result.call_count(),
                                    "found crypto calls"
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

            output_results(&results, classifier, output_format, output_file)?;
        }
        _ => {
            warn!(
                language = language.as_str(),
                "discovery not yet implemented for this language"
            );
        }
    }

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
