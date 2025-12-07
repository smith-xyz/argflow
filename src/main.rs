use anyhow::{Context as AnyhowContext, Result};
use clap::Parser;
use crypto_extractor_core::cli::{self, OutputFormat};
use crypto_extractor_core::discovery::filter::CryptoFileFilter;
use crypto_extractor_core::discovery::languages::go::{GoCryptoFilter, GoPackageLoader};
use crypto_extractor_core::discovery::loader::PackageLoader;
use crypto_extractor_core::scanner::{CryptoCall, ScanResult, Scanner};
use serde::Serialize;
use std::path::Path;

fn main() -> Result<()> {
    let args = cli::Args::parse();
    args.validate().context("Invalid arguments")?;

    let language = args
        .language
        .or_else(|| {
            if args.path.is_file() {
                cli::detect_language(&args.path)
            } else {
                None
            }
        })
        .context("Could not detect language. Please specify --language")?;

    let scanner = Scanner::new();

    if args.path.is_dir() {
        scan_directory(&args.path, language, &scanner, args.output)?;
    } else {
        scan_file(&args.path, language, &scanner, args.output)?;
    }

    Ok(())
}

fn scan_file(
    path: &Path,
    language: cli::Language,
    scanner: &Scanner,
    output_format: OutputFormat,
) -> Result<()> {
    let source = std::fs::read_to_string(path).context("Failed to read file")?;
    let tree = parse_source(&source, language)?;

    let result = scanner.scan_tree(
        &tree,
        source.as_bytes(),
        &path.to_string_lossy(),
        language.as_str(),
    );

    output_results(&[result], output_format)?;
    Ok(())
}

fn scan_directory(
    path: &Path,
    language: cli::Language,
    scanner: &Scanner,
    output_format: OutputFormat,
) -> Result<()> {
    match language {
        cli::Language::Go => {
            let loader = GoPackageLoader;
            let filter = GoCryptoFilter;

            eprintln!("Discovering files...");
            let all_files = loader
                .load_user_code(path)
                .context("Failed to discover user code files")?;
            eprintln!("Found {} files", all_files.len());

            eprintln!("Filtering for crypto usage...");
            let crypto_files: Vec<_> = all_files
                .into_iter()
                .filter_map(|file| {
                    filter
                        .has_crypto_usage(&file)
                        .ok()
                        .and_then(|has_crypto| has_crypto.then_some(file))
                })
                .collect();
            eprintln!("Found {} files with crypto usage", crypto_files.len());

            let mut results = Vec::new();
            for file in &crypto_files {
                match std::fs::read_to_string(file) {
                    Ok(source) => {
                        if let Ok(tree) = parse_source(&source, language) {
                            let result = scanner.scan_tree(
                                &tree,
                                source.as_bytes(),
                                &file.to_string_lossy(),
                                language.as_str(),
                            );
                            if result.call_count() > 0 {
                                results.push(result);
                            }
                        }
                    }
                    Err(e) => eprintln!("Warning: Failed to read {}: {}", file.display(), e),
                }
            }

            output_results(&results, output_format)?;
        }
        _ => {
            eprintln!(
                "WARNING: Discovery not yet implemented for {}",
                language.as_str()
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
    findings: Vec<Finding>,
}

#[derive(Serialize)]
struct Finding {
    file: String,
    line: usize,
    column: usize,
    function: String,
    package: Option<String>,
    full_name: String,
    arguments: Vec<ArgumentValue>,
    raw_text: String,
}

#[derive(Serialize)]
struct ArgumentValue {
    index: usize,
    resolved: bool,
    value: serde_json::Value,
}

fn output_results(results: &[ScanResult], format: OutputFormat) -> Result<()> {
    let total_calls: usize = results.iter().map(|r| r.call_count()).sum();

    let findings: Vec<Finding> = results
        .iter()
        .flat_map(|r| r.calls.iter().map(call_to_finding))
        .collect();

    let output = JsonOutput {
        files_scanned: results.len(),
        total_calls,
        findings,
    };

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Cbom => {
            eprintln!("CBOM output not yet implemented, using JSON");
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }

    Ok(())
}

fn call_to_finding(call: &CryptoCall) -> Finding {
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
        full_name: call.full_name(),
        arguments,
        raw_text: call.raw_text.clone(),
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
