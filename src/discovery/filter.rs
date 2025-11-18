use std::path::Path;

pub trait CryptoFileFilter {
    fn has_crypto_usage(&self, file_path: &Path) -> Result<bool, FilterError>;
}

#[derive(Debug, thiserror::Error)]
pub enum FilterError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("File read error: {0}")]
    FileRead(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_file_filter_trait_compiles() {
        struct TestFilter;

        impl CryptoFileFilter for TestFilter {
            fn has_crypto_usage(&self, _file_path: &Path) -> Result<bool, FilterError> {
                Ok(false)
            }
        }

        let filter = TestFilter;
        let file_path = std::path::PathBuf::from("/tmp/test.go");
        assert!(filter.has_crypto_usage(&file_path).is_ok());
    }

    #[test]
    fn test_filter_error_display() {
        let io_error = std::io::Error::from(std::io::ErrorKind::NotFound);
        let filter_error = FilterError::Io(io_error);
        assert!(filter_error.to_string().contains("IO error"));

        let read_error = FilterError::FileRead("test".to_string());
        assert!(read_error.to_string().contains("File read error"));
    }
}
