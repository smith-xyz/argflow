use anyhow::{Context as AnyhowContext, Result};
use clap::Parser;
use crypto_extractor_core::cli;

fn main() -> Result<()> {
    let args = cli::Args::parse();
    args.validate().context("Invalid arguments")?;

    let language = args
        .language
        .as_deref()
        .or_else(|| {
            if args.path.is_file() {
                cli::detect_language(&args.path)
            } else {
                None
            }
        })
        .context("Could not detect language. Please specify --language")?;

    if args.path.is_dir() {
        println!("Analyzing {} directory: {:?}", language, args.path);
        println!("Output format: {}", args.output);

        eprintln!("WARNING: Scanner and resolver modules not yet implemented");
        eprintln!("   This will be implemented in Phase 5-8 of the TODO");
    } else {
        println!("Analyzing {} file: {:?}", language, args.path);
        println!("Output format: {}", args.output);

        eprintln!("WARNING: Scanner and resolver modules not yet implemented");
        eprintln!("   This will be implemented in Phase 5-8 of the TODO");
    }

    Ok(())
}
