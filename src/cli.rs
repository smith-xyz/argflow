use anyhow::{Context as AnyhowContext, Result};
use clap::{Parser, ValueEnum};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Json,
    Cbom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
pub enum Language {
    Go,
    Python,
    Rust,
    Javascript,
    Typescript,
}

#[derive(Parser, Debug)]
#[command(name = "argflow")]
#[command(about = "Argument flow analyzer - trace where function arguments come from", long_about = None)]
pub struct Args {
    /// Path to file or directory to analyze
    #[arg(long, value_name = "PATH")]
    pub path: PathBuf,

    /// Preset to use (e.g., crypto, tls). Can be specified multiple times.
    #[arg(long, value_name = "PRESET")]
    pub preset: Vec<String>,

    /// Custom rules file (JSON format)
    #[arg(long, value_name = "FILE")]
    pub rules: Option<PathBuf>,

    /// Output file path (prints to stdout if not specified)
    #[arg(short = 'O', long, value_name = "FILE")]
    pub output_file: Option<PathBuf>,

    /// Output format (json, cbom)
    #[arg(short = 'f', long, default_value = "json")]
    pub format: OutputFormat,

    /// Language (auto-detected if not specified)
    #[arg(short, long)]
    pub language: Option<Language>,

    /// Include dependencies (vendor/, go mod cache, node_modules/, etc.)
    #[arg(long)]
    pub include_deps: bool,

    /// Increase verbosity (-v info, -vv debug, -vvv trace)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress all output except errors
    #[arg(short, long)]
    pub quiet: bool,
}

impl Args {
    pub fn validate(&self) -> Result<()> {
        validate_path(&self.path)?;
        if let Some(ref rules_path) = self.rules {
            if !rules_path.exists() {
                anyhow::bail!("Rules file does not exist: {}", rules_path.display());
            }
        }
        Ok(())
    }
}

pub fn detect_language(file_path: &Path) -> Option<Language> {
    file_path.extension()?.to_str().and_then(|ext| match ext {
        "go" => Some(Language::Go),
        "py" => Some(Language::Python),
        "rs" => Some(Language::Rust),
        "js" => Some(Language::Javascript),
        "ts" => Some(Language::Typescript),
        _ => None,
    })
}

pub fn validate_path(path: &Path) -> Result<()> {
    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    if path.is_file() {
        std::fs::metadata(path).with_context(|| format!("Cannot read file: {}", path.display()))?;
    } else if path.is_dir() {
        std::fs::metadata(path)
            .with_context(|| format!("Cannot read directory: {}", path.display()))?;
    } else {
        anyhow::bail!("Path is neither a file nor a directory: {}", path.display());
    }

    Ok(())
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Go => "go",
            Language::Python => "python",
            Language::Rust => "rust",
            Language::Javascript => "javascript",
            Language::Typescript => "typescript",
        }
    }

    pub fn preset_language_name(&self) -> &'static str {
        match self {
            Language::Go => "go",
            Language::Python => "python",
            Language::Rust => "rust",
            Language::Javascript => "javascript",
            Language::Typescript => "javascript",
        }
    }

    pub fn path_separator(&self) -> &'static str {
        match self {
            Language::Rust => "::",
            Language::Python => ".",
            Language::Go => "/",
            Language::Javascript => "/",
            Language::Typescript => "/",
        }
    }
}

impl OutputFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            OutputFormat::Json => "json",
            OutputFormat::Cbom => "cbom",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_language_go() {
        let path = Path::new("test.go");
        assert_eq!(detect_language(path), Some(Language::Go));
    }

    #[test]
    fn test_detect_language_python() {
        let path = Path::new("test.py");
        assert_eq!(detect_language(path), Some(Language::Python));
    }

    #[test]
    fn test_detect_language_rust() {
        let path = Path::new("test.rs");
        assert_eq!(detect_language(path), Some(Language::Rust));
    }

    #[test]
    fn test_detect_language_javascript() {
        let path = Path::new("test.js");
        assert_eq!(detect_language(path), Some(Language::Javascript));
    }

    #[test]
    fn test_detect_language_typescript() {
        let path = Path::new("test.ts");
        assert_eq!(detect_language(path), Some(Language::Typescript));
    }

    #[test]
    fn test_detect_language_unknown() {
        let path = Path::new("test.txt");
        assert_eq!(detect_language(path), None);
    }

    #[test]
    fn test_detect_language_no_extension() {
        let path = Path::new("test");
        assert_eq!(detect_language(path), None);
    }

    #[test]
    fn test_language_as_str() {
        assert_eq!(Language::Go.as_str(), "go");
        assert_eq!(Language::Python.as_str(), "python");
        assert_eq!(Language::Rust.as_str(), "rust");
        assert_eq!(Language::Javascript.as_str(), "javascript");
        assert_eq!(Language::Typescript.as_str(), "typescript");
    }

    #[test]
    fn test_language_preset_language_name() {
        assert_eq!(Language::Go.preset_language_name(), "go");
        assert_eq!(Language::Python.preset_language_name(), "python");
        assert_eq!(Language::Rust.preset_language_name(), "rust");
        assert_eq!(Language::Javascript.preset_language_name(), "javascript");
        assert_eq!(Language::Typescript.preset_language_name(), "javascript");
    }

    #[test]
    fn test_language_path_separator() {
        assert_eq!(Language::Rust.path_separator(), "::");
        assert_eq!(Language::Python.path_separator(), ".");
        assert_eq!(Language::Go.path_separator(), "/");
        assert_eq!(Language::Javascript.path_separator(), "/");
        assert_eq!(Language::Typescript.path_separator(), "/");
    }

    #[test]
    fn test_output_format_as_str() {
        assert_eq!(OutputFormat::Json.as_str(), "json");
        assert_eq!(OutputFormat::Cbom.as_str(), "cbom");
    }

    #[test]
    fn test_validate_path_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.go");
        fs::write(&file_path, "package main").unwrap();

        assert!(validate_path(&file_path).is_ok());
    }

    #[test]
    fn test_validate_path_directory_exists() {
        let temp_dir = TempDir::new().unwrap();
        assert!(validate_path(temp_dir.path()).is_ok());
    }

    #[test]
    fn test_validate_path_not_exists() {
        let path = Path::new("/nonexistent/path/that/does/not/exist");
        assert!(validate_path(path).is_err());
    }

    #[test]
    fn test_args_validate_all_valid() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.go");
        fs::write(&file_path, "package main").unwrap();

        let args = Args {
            path: file_path,
            preset: vec![],
            rules: None,
            output_file: None,
            format: OutputFormat::Json,
            language: Some(Language::Go),
            include_deps: false,
            verbose: 0,
            quiet: false,
        };

        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_args_validate_with_preset() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.go");
        fs::write(&file_path, "package main").unwrap();

        let args = Args {
            path: file_path,
            preset: vec!["crypto".to_string()],
            rules: None,
            output_file: None,
            format: OutputFormat::Json,
            language: Some(Language::Go),
            include_deps: false,
            verbose: 0,
            quiet: false,
        };

        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_args_validate_invalid_path() {
        let args = Args {
            path: PathBuf::from("/nonexistent/path"),
            preset: vec![],
            rules: None,
            output_file: None,
            format: OutputFormat::Json,
            language: None,
            include_deps: false,
            verbose: 0,
            quiet: false,
        };

        assert!(args.validate().is_err());
    }

    #[test]
    fn test_verbose_flag_incremental() {
        let args = Args {
            path: PathBuf::from("."),
            preset: vec![],
            rules: None,
            output_file: None,
            format: OutputFormat::Json,
            language: None,
            include_deps: false,
            verbose: 2,
            quiet: false,
        };

        assert_eq!(args.verbose, 2);
    }
}
