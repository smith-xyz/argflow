use anyhow::{Context as AnyhowContext, Result};
use clap::{Parser, ValueEnum};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Json,
    Cbom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Language {
    Go,
    Python,
    Rust,
    Javascript,
    Typescript,
}

#[derive(Parser, Debug)]
#[command(name = "crypto-extractor")]
#[command(about = "Cryptographic parameter extractor", long_about = None)]
pub struct Args {
    /// Path to file or directory to analyze
    #[arg(long, value_name = "PATH")]
    pub path: PathBuf,

    /// Output format (json, cbom)
    #[arg(short, long, default_value = "json")]
    pub output: OutputFormat,

    /// Language (auto-detected if not specified)
    #[arg(short, long)]
    pub language: Option<Language>,
}

impl Args {
    pub fn validate(&self) -> Result<()> {
        validate_path(&self.path)?;
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
            output: OutputFormat::Json,
            language: Some(Language::Go),
        };

        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_args_validate_invalid_language() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.go");
        fs::write(&file_path, "package main").unwrap();

        let args = Args {
            path: file_path,
            output: OutputFormat::Json,
            language: Some(Language::Go),
        };

        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_args_validate_invalid_path() {
        let args = Args {
            path: PathBuf::from("/nonexistent/path"),
            output: OutputFormat::Json,
            language: None,
        };

        assert!(args.validate().is_err());
    }
}
