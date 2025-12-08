use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("unsupported language: {language}")]
    UnsupportedLanguage { language: String },

    #[error("failed to set parser language: {language}")]
    LanguageSetupFailed { language: String },

    #[error("failed to parse source code in {path}")]
    ParseFailed { path: PathBuf },

    #[error("syntax error at line {line}, column {column}: {message}")]
    SyntaxError {
        line: usize,
        column: usize,
        message: String,
    },

    #[error("invalid node type: expected {expected}, found {found}")]
    InvalidNodeType { expected: String, found: String },
}

impl ParserError {
    pub fn unsupported_language(language: impl Into<String>) -> Self {
        Self::UnsupportedLanguage {
            language: language.into(),
        }
    }

    pub fn language_setup_failed(language: impl Into<String>) -> Self {
        Self::LanguageSetupFailed {
            language: language.into(),
        }
    }

    pub fn parse_failed(path: impl Into<PathBuf>) -> Self {
        Self::ParseFailed { path: path.into() }
    }

    pub fn syntax_error(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self::SyntaxError {
            line,
            column,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsupported_language_display() {
        let err = ParserError::unsupported_language("brainfuck");
        assert_eq!(err.to_string(), "unsupported language: brainfuck");
    }

    #[test]
    fn test_syntax_error_display() {
        let err = ParserError::syntax_error(10, 5, "unexpected token");
        assert_eq!(
            err.to_string(),
            "syntax error at line 10, column 5: unexpected token"
        );
    }
}
