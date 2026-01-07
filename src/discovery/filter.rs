use std::path::Path;

use crate::cli::Language;

pub trait ImportFileFilter: Send + Sync {
    fn has_matching_imports(&self, file_path: &Path) -> Result<bool, FilterError>;

    fn language(&self) -> Language;
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
    fn test_import_file_filter_trait_compiles() {
        struct TestFilter;

        impl ImportFileFilter for TestFilter {
            fn has_matching_imports(&self, _file_path: &Path) -> Result<bool, FilterError> {
                Ok(false)
            }

            fn language(&self) -> Language {
                Language::Go
            }
        }

        let filter = TestFilter;
        let file_path = std::path::PathBuf::from("/tmp/test.go");
        assert!(filter.has_matching_imports(&file_path).is_ok());
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
