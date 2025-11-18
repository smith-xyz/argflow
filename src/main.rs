use anyhow::{Context as AnyhowContext, Result};
use clap::Parser;
use crypto_extractor_core::cli;
use crypto_extractor_core::discovery::filter::CryptoFileFilter;
use crypto_extractor_core::discovery::languages::go::{GoCryptoFilter, GoPackageLoader};
use crypto_extractor_core::discovery::loader::PackageLoader;

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

    if args.path.is_dir() {
        println!("Analyzing {} directory: {:?}", language.as_str(), args.path);
        println!("Output format: {}", args.output.as_str());
        println!();

        match language {
            cli::Language::Go => {
                let loader = GoPackageLoader;
                let filter = GoCryptoFilter;

                println!("Discovering files...");
                let all_files = loader
                    .load_user_code(&args.path)
                    .context("Failed to discover user code files")?;

                println!("Found {} files", all_files.len());

                println!("\nFiltering for crypto usage...");
                let crypto_files: Vec<_> = all_files
                    .into_iter()
                    .filter_map(|file| {
                        filter
                            .has_crypto_usage(&file)
                            .ok()
                            .and_then(|has_crypto| has_crypto.then_some(file))
                    })
                    .collect();

                println!("Found {} files with crypto usage", crypto_files.len());

                println!("\nDiscovering dependencies...");
                let dep_files = loader
                    .load_dependencies(&args.path)
                    .context("Failed to discover dependency files")?;

                println!("Found {} dependency files", dep_files.len());

                println!("\nFiltering dependencies for crypto usage...");
                let crypto_dep_files: Vec<_> = dep_files
                    .into_iter()
                    .filter_map(|file| {
                        filter
                            .has_crypto_usage(&file)
                            .ok()
                            .and_then(|has_crypto| has_crypto.then_some(file))
                    })
                    .collect();

                println!(
                    "Found {} dependency files with crypto usage",
                    crypto_dep_files.len()
                );

                println!("\nFiles that would be scanned:");
                println!("\nUser code ({} files):", crypto_files.len());
                for file in &crypto_files {
                    println!("  - {}", file.display());
                }

                if !crypto_dep_files.is_empty() {
                    println!("\nDependencies ({} files):", crypto_dep_files.len());
                    let go_jose_files: Vec<_> = crypto_dep_files
                        .iter()
                        .filter(|f| {
                            f.to_string_lossy().contains("go-jose")
                                || f.to_string_lossy().contains("go-jose/v3")
                        })
                        .collect();

                    if !go_jose_files.is_empty() {
                        println!("  go-jose files:");
                        for file in &go_jose_files {
                            println!("    - {}", file.display());
                        }
                    }

                    let other_dep_files: Vec<_> = crypto_dep_files
                        .iter()
                        .filter(|f| {
                            !f.to_string_lossy().contains("go-jose")
                                && !f.to_string_lossy().contains("go-jose/v3")
                        })
                        .take(5)
                        .collect();

                    if !other_dep_files.is_empty() {
                        println!("  Other dependencies (showing first 5):");
                        for file in &other_dep_files {
                            println!("    - {}", file.display());
                        }
                        if crypto_dep_files.len() > other_dep_files.len() + go_jose_files.len() {
                            println!(
                                "    ... and {} more",
                                crypto_dep_files.len()
                                    - other_dep_files.len()
                                    - go_jose_files.len()
                            );
                        }
                    }
                }

                if crypto_files.is_empty() && crypto_dep_files.is_empty() {
                    println!("\nNo crypto usage detected.");
                } else {
                    println!("\nNOTE: Scanner and resolver modules not yet implemented");
                    println!("   Full parameter extraction will be implemented in Phase 5-8");
                }
            }
            _ => {
                eprintln!(
                    "WARNING: Discovery not yet implemented for {}",
                    language.as_str()
                );
                eprintln!("   This will be implemented in future phases");
            }
        }
    } else {
        println!("Analyzing {} file: {:?}", language.as_str(), args.path);
        println!("Output format: {}", args.output.as_str());

        eprintln!("\nWARNING: Scanner and resolver modules not yet implemented");
        eprintln!("   This will be implemented in Phase 5-8 of the TODO");
    }

    Ok(())
}
