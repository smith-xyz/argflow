use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClassifierError {
    #[error("failed to read rules file '{path}': {message}")]
    RulesFileReadError { path: PathBuf, message: String },

    #[error("failed to parse rules file '{path}': {message}")]
    RulesParseError { path: PathBuf, message: String },

    #[error("unsupported rules format: {format} (expected json or yaml)")]
    UnsupportedFormat { format: String },

    #[error("missing classification key: {key}")]
    MissingClassificationKey { key: String },

    #[error("invalid classification schema: {message}")]
    InvalidSchema { message: String },
}

impl ClassifierError {
    pub fn rules_file_read_error(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::RulesFileReadError {
            path: path.into(),
            message: message.into(),
        }
    }

    pub fn rules_parse_error(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::RulesParseError {
            path: path.into(),
            message: message.into(),
        }
    }

    pub fn unsupported_format(format: impl Into<String>) -> Self {
        Self::UnsupportedFormat {
            format: format.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rules_file_read_error_display() {
        let err = ClassifierError::rules_file_read_error("/path/to/rules.json", "file not found");
        assert_eq!(
            err.to_string(),
            "failed to read rules file '/path/to/rules.json': file not found"
        );
    }

    #[test]
    fn test_unsupported_format_display() {
        let err = ClassifierError::unsupported_format("xml");
        assert_eq!(
            err.to_string(),
            "unsupported rules format: xml (expected json or yaml)"
        );
    }
}
