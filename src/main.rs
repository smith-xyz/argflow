use anyhow::{Context as AnyhowContext, Result};
use clap::Parser;
use crypto_extractor_core::classifier::{classify_call, RulesClassifier};
use crypto_extractor_core::cli::{self, OutputFormat};
use crypto_extractor_core::discovery::cache::DiscoveryCache;
use crypto_extractor_core::discovery::filter::CryptoFileFilter;
use crypto_extractor_core::discovery::languages::go::{GoCryptoFilter, GoPackageLoader};
use crypto_extractor_core::discovery::loader::PackageLoader;
use crypto_extractor_core::logging::{self, Verbosity};
use crypto_extractor_core::scanner::{CryptoCall, CryptoConfig, ScanResult, Scanner};
use serde::Serialize;
use std::path::Path;
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
            args.output,
            args.include_deps,
        )?;
    } else {
        scan_file(&args.path, language, &scanner, &classifier, args.output)?;
    }

    Ok(())
}

fn scan_file(
    path: &Path,
    language: cli::Language,
    scanner: &Scanner,
    classifier: &RulesClassifier,
    output_format: OutputFormat,
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

    output_results(&[result], classifier, output_format)?;
    Ok(())
}

fn scan_directory(
    path: &Path,
    language: cli::Language,
    scanner: &Scanner,
    classifier: &RulesClassifier,
    output_format: OutputFormat,
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

            output_results(&results, classifier, output_format)?;
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

#[derive(Serialize)]
struct JsonOutput {
    files_scanned: usize,
    total_calls: usize,
    total_configs: usize,
    findings: Vec<Finding>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    configs: Vec<ConfigFinding>,
}

#[derive(Serialize)]
struct Finding {
    file: String,
    line: usize,
    column: usize,
    function: String,
    package: Option<String>,
    import_path: Option<String>,
    full_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    algorithm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    finding_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    operation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    primitive: Option<String>,
    arguments: Vec<ArgumentValue>,
    raw_text: String,
}

#[derive(Serialize)]
struct ArgumentValue {
    index: usize,
    resolved: bool,
    value: serde_json::Value,
}

#[derive(Serialize)]
struct ConfigFinding {
    file: String,
    line: usize,
    column: usize,
    struct_type: String,
    full_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    package: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    import_path: Option<String>,
    fields: Vec<ConfigFieldValue>,
    raw_text: String,
}

#[derive(Serialize)]
struct ConfigFieldValue {
    field_name: String,
    resolved: bool,
    value: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    classification_key: Option<String>,
}

fn output_results(
    results: &[ScanResult],
    classifier: &RulesClassifier,
    format: OutputFormat,
) -> Result<()> {
    let total_calls: usize = results.iter().map(|r| r.call_count()).sum();
    let total_configs: usize = results.iter().map(|r| r.config_count()).sum();

    let findings: Vec<Finding> = results
        .iter()
        .flat_map(|r| r.calls.iter().map(|call| call_to_finding(call, classifier)))
        .collect();

    let configs: Vec<ConfigFinding> = results
        .iter()
        .flat_map(|r| r.configs.iter().map(config_to_finding))
        .collect();

    let output = JsonOutput {
        files_scanned: results.len(),
        total_calls,
        total_configs,
        findings,
        configs,
    };

    match format {
        OutputFormat::Json => {
            trace!("outputting JSON format");
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Cbom => {
            warn!("CBOM output not yet implemented, using JSON");
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }

    Ok(())
}

fn call_to_finding(call: &CryptoCall, classifier: &RulesClassifier) -> Finding {
    let classification = classify_call(call, classifier);

    let arguments: Vec<ArgumentValue> = call
        .arguments
        .iter()
        .enumerate()
        .map(|(i, v)| ArgumentValue {
            index: i,
            resolved: v.is_resolved,
            value: value_to_json(v),
        })
        .collect();

    Finding {
        file: call.file_path.clone(),
        line: call.line,
        column: call.column,
        function: call.function_name.clone(),
        package: call.package.clone(),
        import_path: call.import_path.clone(),
        full_name: call.full_name(),
        algorithm: classification.algorithm,
        finding_type: if classification.finding_type.is_empty() {
            None
        } else {
            Some(classification.finding_type)
        },
        operation: if classification.operation.is_empty() {
            None
        } else {
            Some(classification.operation)
        },
        primitive: classification.primitive,
        arguments,
        raw_text: call.raw_text.clone(),
    }
}

fn config_to_finding(config: &CryptoConfig) -> ConfigFinding {
    let fields: Vec<ConfigFieldValue> = config
        .fields
        .iter()
        .map(|f| ConfigFieldValue {
            field_name: f.field_name.clone(),
            resolved: f.value.is_resolved,
            value: value_to_json(&f.value),
            classification_key: f.classification_key.clone(),
        })
        .collect();

    ConfigFinding {
        file: config.file_path.clone(),
        line: config.line,
        column: config.column,
        struct_type: config.struct_type.clone(),
        full_type: config.full_type(),
        package: config.package.clone(),
        import_path: config.import_path.clone(),
        fields,
        raw_text: config.raw_text.clone(),
    }
}

fn value_to_json(value: &crypto_extractor_core::Value) -> serde_json::Value {
    if !value.int_values.is_empty() {
        if value.int_values.len() == 1 {
            serde_json::Value::Number(value.int_values[0].into())
        } else {
            serde_json::Value::Array(
                value
                    .int_values
                    .iter()
                    .map(|&v| serde_json::Value::Number(v.into()))
                    .collect(),
            )
        }
    } else if !value.string_values.is_empty() {
        if value.string_values.len() == 1 {
            serde_json::Value::String(value.string_values[0].clone())
        } else {
            serde_json::Value::Array(
                value
                    .string_values
                    .iter()
                    .map(|s| serde_json::Value::String(s.clone()))
                    .collect(),
            )
        }
    } else if !value.expression.is_empty() {
        serde_json::json!({
            "expression": value.expression,
            "partial": true
        })
    } else if !value.source.is_empty() {
        serde_json::json!({
            "unresolved": value.source
        })
    } else {
        serde_json::Value::Null
    }
}
