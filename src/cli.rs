use anyhow::{Context as AnyhowContext, Result};
use clap::Parser;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "crypto-extractor")]
#[command(about = "Cryptographic parameter extractor", long_about = None)]
pub struct Args {
    /// Path to file or directory to analyze
    #[arg(long, value_name = "PATH")]
    pub path: PathBuf,

    /// Output format (json, cbom)
    #[arg(short, long, default_value = "json")]
    pub output: String,

    /// Language (auto-detected if not specified)
    #[arg(short, long)]
    pub language: Option<String>,
}

impl Args {
    pub fn validate(&self) -> Result<()> {
        validate_path(&self.path)?;
        validate_output_format(&self.output)?;
        if let Some(ref lang) = self.language {
            validate_language(lang)?;
        }
        Ok(())
    }
}

pub fn detect_language(file_path: &Path) -> Option<&'static str> {
    file_path.extension()?.to_str().and_then(|ext| match ext {
        "go" => Some("go"),
        "py" => Some("python"),
        "rs" => Some("rust"),
        "js" => Some("javascript"),
        "ts" => Some("typescript"),
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

pub fn validate_language(lang: &str) -> Result<()> {
    const SUPPORTED_LANGUAGES: &[&str] = &["go", "python", "rust", "javascript", "typescript"];

    if !SUPPORTED_LANGUAGES.contains(&lang) {
        anyhow::bail!(
            "Unsupported language: {}. Supported languages: {}",
            lang,
            SUPPORTED_LANGUAGES.join(", ")
        );
    }

    Ok(())
}

pub fn validate_output_format(format: &str) -> Result<()> {
    const SUPPORTED_FORMATS: &[&str] = &["json", "cbom"];

    if !SUPPORTED_FORMATS.contains(&format) {
        anyhow::bail!(
            "Unsupported output format: {}. Supported formats: {}",
            format,
            SUPPORTED_FORMATS.join(", ")
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_language_go() {
        let path = Path::new("test.go");
        assert_eq!(detect_language(path), Some("go"));
    }

    #[test]
    fn test_detect_language_python() {
        let path = Path::new("test.py");
        assert_eq!(detect_language(path), Some("python"));
    }

    #[test]
    fn test_detect_language_rust() {
        let path = Path::new("test.rs");
        assert_eq!(detect_language(path), Some("rust"));
    }

    #[test]
    fn test_detect_language_javascript() {
        let path = Path::new("test.js");
        assert_eq!(detect_language(path), Some("javascript"));
    }

    #[test]
    fn test_detect_language_typescript() {
        let path = Path::new("test.ts");
        assert_eq!(detect_language(path), Some("typescript"));
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
    fn test_validate_language_valid() {
        assert!(validate_language("go").is_ok());
        assert!(validate_language("python").is_ok());
        assert!(validate_language("rust").is_ok());
        assert!(validate_language("javascript").is_ok());
        assert!(validate_language("typescript").is_ok());
    }

    #[test]
    fn test_validate_language_invalid() {
        assert!(validate_language("java").is_err());
        assert!(validate_language("cpp").is_err());
        assert!(validate_language("").is_err());
    }

    #[test]
    fn test_validate_output_format_valid() {
        assert!(validate_output_format("json").is_ok());
        assert!(validate_output_format("cbom").is_ok());
    }

    #[test]
    fn test_validate_output_format_invalid() {
        assert!(validate_output_format("xml").is_err());
        assert!(validate_output_format("yaml").is_err());
        assert!(validate_output_format("").is_err());
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
            output: "json".to_string(),
            language: Some("go".to_string()),
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
            output: "json".to_string(),
            language: Some("java".to_string()),
        };

        assert!(args.validate().is_err());
    }

    #[test]
    fn test_args_validate_invalid_output() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.go");
        fs::write(&file_path, "package main").unwrap();

        let args = Args {
            path: file_path,
            output: "xml".to_string(),
            language: None,
        };

        assert!(args.validate().is_err());
    }

    #[test]
    fn test_args_validate_invalid_path() {
        let args = Args {
            path: PathBuf::from("/nonexistent/path"),
            output: "json".to_string(),
            language: None,
        };

        assert!(args.validate().is_err());
    }
}
