use std::path::Path;

use crate::cli::Language;
use crate::discovery::cache::DiscoveryCache;
use crate::discovery::SourceFile;

pub trait PackageLoader: Send + Sync {
    fn load_user_code(&self, root: &Path) -> Result<Vec<SourceFile>, LoadError>;

    fn load_dependencies(
        &self,
        root: &Path,
        cache: &mut DiscoveryCache,
    ) -> Result<Vec<SourceFile>, LoadError>;

    fn language(&self) -> Language;
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Package manager error: {0}")]
    PackageManager(String),

    #[error("Failed to scan directory at {path}: {source}")]
    DirectoryScanError {
        path: std::path::PathBuf,
        source: walkdir::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::cache::DiscoveryCache;
    use std::path::PathBuf;

    #[test]
    fn test_package_loader_trait_compiles() {
        struct TestLoader;

        impl PackageLoader for TestLoader {
            fn load_user_code(&self, _root: &Path) -> Result<Vec<SourceFile>, LoadError> {
                Ok(vec![])
            }

            fn load_dependencies(
                &self,
                _root: &Path,
                _cache: &mut DiscoveryCache,
            ) -> Result<Vec<SourceFile>, LoadError> {
                Ok(vec![])
            }

            fn language(&self) -> Language {
                Language::Go
            }
        }

        let loader = TestLoader;
        let mut cache = DiscoveryCache::default();
        let root = PathBuf::from("/tmp");
        assert!(loader.load_user_code(&root).is_ok());
        assert!(loader.load_dependencies(&root, &mut cache).is_ok());
    }

    #[test]
    fn test_load_error_display() {
        let io_error = std::io::Error::from(std::io::ErrorKind::NotFound);
        let load_error = LoadError::Io(io_error);
        assert!(load_error.to_string().contains("IO error"));

        let path_error = LoadError::InvalidPath("invalid".to_string());
        assert!(path_error.to_string().contains("Invalid path"));

        let pkg_error = LoadError::PackageManager("test".to_string());
        assert!(pkg_error.to_string().contains("Package manager error"));

        let test_path = std::path::PathBuf::from("/nonexistent/test/path");
        let walkdir_result = walkdir::WalkDir::new(&test_path).into_iter().next();
        if let Some(Err(walkdir_error)) = walkdir_result {
            let scan_error = LoadError::DirectoryScanError {
                path: test_path.clone(),
                source: walkdir_error,
            };
            let error_msg = scan_error.to_string();
            assert!(error_msg.contains("Failed to scan directory"));
            assert!(error_msg.contains("/nonexistent/test/path"));
        } else {
            panic!("Expected walkdir error but got success");
        }
    }
}
