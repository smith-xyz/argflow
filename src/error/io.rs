use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IoError {
    #[error("file not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("directory not found: {path}")]
    DirectoryNotFound { path: PathBuf },

    #[error("permission denied: {path}")]
    PermissionDenied { path: PathBuf },

    #[error("failed to read file '{path}': {source}")]
    ReadError {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to write file '{path}': {source}")]
    WriteError {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("path is neither file nor directory: {path}")]
    InvalidPath { path: PathBuf },
}

impl IoError {
    pub fn file_not_found(path: impl Into<PathBuf>) -> Self {
        Self::FileNotFound { path: path.into() }
    }

    pub fn directory_not_found(path: impl Into<PathBuf>) -> Self {
        Self::DirectoryNotFound { path: path.into() }
    }

    pub fn read_error(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::ReadError {
            path: path.into(),
            source,
        }
    }

    pub fn write_error(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::WriteError {
            path: path.into(),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_not_found_display() {
        let err = IoError::file_not_found("/path/to/file.rs");
        assert_eq!(err.to_string(), "file not found: /path/to/file.rs");
    }

    #[test]
    fn test_directory_not_found_display() {
        let err = IoError::directory_not_found("/path/to/dir");
        assert_eq!(err.to_string(), "directory not found: /path/to/dir");
    }
}
