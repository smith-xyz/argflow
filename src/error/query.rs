use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("invalid query pattern for {language}/{query_name}: {message}")]
    InvalidPattern {
        language: String,
        query_name: String,
        message: String,
    },

    #[error("query '{query_name}' not found for language: {language}")]
    QueryNotFound {
        language: String,
        query_name: String,
    },

    #[error("language '{language}' is not supported")]
    LanguageNotSupported { language: String },

    #[error("query execution failed: {message}")]
    ExecutionError { message: String },
}

impl QueryError {
    pub fn invalid_pattern(
        language: impl Into<String>,
        query_name: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::InvalidPattern {
            language: language.into(),
            query_name: query_name.into(),
            message: message.into(),
        }
    }

    pub fn query_not_found(language: impl Into<String>, query_name: impl Into<String>) -> Self {
        Self::QueryNotFound {
            language: language.into(),
            query_name: query_name.into(),
        }
    }

    pub fn language_not_supported(language: impl Into<String>) -> Self {
        Self::LanguageNotSupported {
            language: language.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_pattern_display() {
        let err = QueryError::invalid_pattern("go", "imports", "missing capture name");
        assert_eq!(
            err.to_string(),
            "invalid query pattern for go/imports: missing capture name"
        );
    }

    #[test]
    fn test_query_not_found_display() {
        let err = QueryError::query_not_found("python", "exports");
        assert_eq!(
            err.to_string(),
            "query 'exports' not found for language: python"
        );
    }

    #[test]
    fn test_language_not_supported_display() {
        let err = QueryError::language_not_supported("fortran");
        assert_eq!(err.to_string(), "language 'fortran' is not supported");
    }
}
