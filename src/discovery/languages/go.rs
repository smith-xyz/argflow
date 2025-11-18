use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::discovery::filter::{CryptoFileFilter, FilterError};
use crate::discovery::loader::{LoadError, PackageLoader};

const EXCLUDED_DIRS: &[&str] = &["vendor", "testdata", ".git", "node_modules"];

const GO_MOD_CACHE_SUFFIXES: &[&str] = &["pkg/mod", "go/pkg/mod"];

const CRYPTO_PATTERNS: &[&str] = &[
    "crypto/",
    "\"crypto/",
    "`crypto/",
    "golang.org/x/crypto",
    "github.com/cloudflare/circl",
];

const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

pub struct GoPackageLoader;

impl PackageLoader for GoPackageLoader {
    fn load_user_code(&self, root: &Path) -> Result<Vec<PathBuf>, LoadError> {
        let root_path = root;

        if !root_path.exists() {
            return Err(LoadError::InvalidPath(format!(
                "Path does not exist: {}",
                root_path.display()
            )));
        }

        if !root_path.is_dir() {
            return Err(LoadError::InvalidPath(format!(
                "Path is not a directory: {}",
                root_path.display()
            )));
        }

        let mut files = Vec::new();

        for entry in WalkDir::new(root_path).into_iter().filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            if e.file_type().is_dir() {
                !EXCLUDED_DIRS.contains(&name.as_ref())
            } else {
                true
            }
        }) {
            let entry = entry.map_err(|e| LoadError::DirectoryScanError {
                path: root_path.to_path_buf(),
                source: e,
            })?;

            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "go" {
                        let path = entry.path();
                        let mut is_excluded = false;

                        for component in path.components() {
                            if let std::path::Component::Normal(name) = component {
                                let name_str = name.to_string_lossy();
                                if EXCLUDED_DIRS.contains(&name_str.as_ref()) {
                                    is_excluded = true;
                                    break;
                                }
                            }
                        }

                        if !is_excluded {
                            files.push(entry.path().to_path_buf());
                        }
                    }
                }
            }
        }

        Ok(files)
    }

    fn load_dependencies(&self, root: &Path) -> Result<Vec<PathBuf>, LoadError> {
        let root_path = root;
        let mut files = Vec::new();

        let vendor_path = root_path.join("vendor");
        if vendor_path.exists() && vendor_path.is_dir() {
            files.extend(self.scan_vendor(&vendor_path)?);
        } else {
            files.extend(self.scan_go_mod_cache(root_path)?);
        }

        Ok(files)
    }
}

impl GoPackageLoader {
    pub fn exclude_stdlib(&self, files: Vec<PathBuf>) -> Vec<PathBuf> {
        let goroot = env::var("GOROOT").ok();

        files
            .into_iter()
            .filter(|path| {
                if let Some(ref goroot) = goroot {
                    let goroot_path = PathBuf::from(goroot);
                    !path.starts_with(&goroot_path)
                } else {
                    true
                }
            })
            .collect()
    }

    fn scan_vendor(&self, vendor_path: &Path) -> Result<Vec<PathBuf>, LoadError> {
        let mut files = Vec::new();

        for entry in WalkDir::new(vendor_path)
            .into_iter()
            .filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.'))
        {
            let entry = entry.map_err(|e| LoadError::DirectoryScanError {
                path: vendor_path.to_path_buf(),
                source: e,
            })?;

            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "go" {
                        files.push(entry.path().to_path_buf());
                    }
                }
            }
        }

        Ok(files)
    }

    fn scan_go_mod_cache(&self, project_root: &Path) -> Result<Vec<PathBuf>, LoadError> {
        let go_mod_path = project_root.join("go.mod");

        if !go_mod_path.exists() {
            return Ok(vec![]);
        }

        let mut files = Vec::new();

        let base_paths: Vec<Option<String>> = vec![env::var("GOPATH").ok(), env::var("HOME").ok()];

        let module_cache_paths: Vec<Option<PathBuf>> = base_paths
            .into_iter()
            .zip(GO_MOD_CACHE_SUFFIXES.iter())
            .map(|(base_opt, suffix)| base_opt.map(|base| PathBuf::from(base).join(suffix)))
            .collect();

        for cache_path_opt in module_cache_paths {
            if let Some(cache_path) = cache_path_opt {
                if cache_path.exists() && cache_path.is_dir() {
                    for entry in WalkDir::new(&cache_path)
                        .into_iter()
                        .filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.'))
                    {
                        let entry = entry.map_err(|e| LoadError::DirectoryScanError {
                            path: cache_path.clone(),
                            source: e,
                        })?;

                        if entry.file_type().is_file() {
                            if let Some(ext) = entry.path().extension() {
                                if ext == "go" {
                                    files.push(entry.path().to_path_buf());
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(files)
    }
}

pub struct GoCryptoFilter;

impl CryptoFileFilter for GoCryptoFilter {
    fn has_crypto_usage(&self, file_path: &Path) -> Result<bool, FilterError> {
        let metadata = fs::metadata(file_path).map_err(|e| {
            FilterError::FileRead(format!(
                "Failed to read metadata for {}: {}",
                file_path.display(),
                e
            ))
        })?;

        if metadata.len() > MAX_FILE_SIZE {
            return Err(FilterError::FileRead(format!(
                "File too large: {} bytes (max: {} bytes)",
                metadata.len(),
                MAX_FILE_SIZE
            )));
        }

        let content = fs::read_to_string(file_path).map_err(|e| {
            FilterError::FileRead(format!(
                "Failed to read file {}: {}",
                file_path.display(),
                e
            ))
        })?;

        Ok(self.check_crypto_imports(&content))
    }
}

impl GoCryptoFilter {
    fn check_crypto_imports(&self, content: &str) -> bool {
        CRYPTO_PATTERNS
            .iter()
            .any(|pattern| content.contains(*pattern))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_load_user_code_finds_go_files() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        fs::create_dir_all(root.join("pkg")).unwrap();
        fs::File::create(root.join("main.go"))
            .unwrap()
            .write_all(b"package main")
            .unwrap();
        fs::File::create(root.join("pkg/helper.go"))
            .unwrap()
            .write_all(b"package pkg")
            .unwrap();
        fs::File::create(root.join("README.md"))
            .unwrap()
            .write_all(b"# Project")
            .unwrap();

        let loader = GoPackageLoader;
        let files = loader.load_user_code(&root).unwrap();

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.file_name().unwrap() == "main.go"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "helper.go"));
    }

    #[test]
    fn test_load_user_code_excludes_vendor() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        fs::create_dir_all(root.join("vendor/pkg")).unwrap();
        fs::File::create(root.join("main.go"))
            .unwrap()
            .write_all(b"package main")
            .unwrap();
        fs::File::create(root.join("vendor/pkg/lib.go"))
            .unwrap()
            .write_all(b"package pkg")
            .unwrap();

        let loader = GoPackageLoader;
        let files = loader.load_user_code(&root).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files.iter().any(|f| f.file_name().unwrap() == "main.go"));
        assert!(!files.iter().any(|f| f.to_string_lossy().contains("vendor")));
    }

    #[test]
    fn test_load_dependencies_finds_vendor() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        fs::create_dir_all(root.join("vendor/pkg")).unwrap();
        fs::File::create(root.join("vendor/pkg/lib.go"))
            .unwrap()
            .write_all(b"package pkg")
            .unwrap();

        let loader = GoPackageLoader;
        let files = loader.load_dependencies(&root).unwrap();

        assert!(files.len() >= 1);
        assert!(files.iter().any(|f| f.to_string_lossy().contains("vendor")));
    }

    #[test]
    fn test_exclude_stdlib_filters_goroot() {
        let temp_dir = TempDir::new().unwrap();
        let goroot = temp_dir.path().to_path_buf();

        env::set_var("GOROOT", goroot.to_str().unwrap());

        let files = vec![
            goroot.join("src/crypto/aes/cipher.go"),
            PathBuf::from("/tmp/user/main.go"),
        ];

        let loader = GoPackageLoader;
        let filtered = loader.exclude_stdlib(files);

        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].to_string_lossy().contains("user"));

        env::remove_var("GOROOT");
    }

    #[test]
    fn test_go_crypto_filter_detects_crypto_imports() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("main.go");

        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"package main\nimport \"crypto/aes\"\n")
            .unwrap();

        let filter = GoCryptoFilter;
        assert!(filter.has_crypto_usage(&file_path).unwrap());
    }

    #[test]
    fn test_go_crypto_filter_detects_third_party_crypto() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("main.go");

        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"package main\nimport \"golang.org/x/crypto/bcrypt\"\n")
            .unwrap();

        let filter = GoCryptoFilter;
        assert!(filter.has_crypto_usage(&file_path).unwrap());
    }

    #[test]
    fn test_go_crypto_filter_no_crypto() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("main.go");

        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"package main\nimport \"fmt\"\n").unwrap();

        let filter = GoCryptoFilter;
        assert!(!filter.has_crypto_usage(&file_path).unwrap());
    }
}
