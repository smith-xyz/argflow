use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
use std::sync::OnceLock;

use crate::discovery::filter::{CryptoFileFilter, FilterError};
use crate::discovery::loader::{LoadError, PackageLoader};
use crate::discovery::utils::walk_source_files;

const EXCLUDED_DIRS: &[&str] = &["vendor", "testdata", ".git"];

const CRYPTO_PATTERNS: &[&str] = &[
    "crypto/",
    "\"crypto/",
    "`crypto/",
    "golang.org/x/crypto",
    "github.com/cloudflare/circl",
];

const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

const GO_COMMAND: &str = "go";

const STD_PREFIX: &str = "std/";
const INTERNAL_PREFIX: &str = "internal/";
const VENDOR_PREFIX: &str = "vendor/";

const GO_LIST_STD_ARGS: &[&str] = &["list", "std"];

const GO_LIST_DEPS_ARGS: &[&str] = &["list", "-deps", "-f"];
const GO_LIST_IMPORT_PATH_TEMPLATE: &str = "{{.ImportPath}}";
const GO_LIST_PACKAGE_PATTERN: &str = "./...";

const GO_LIST_DIR_ARGS: &[&str] = &["list", "-f"];
const GO_LIST_DIR_TEMPLATE: &str = "{{.Dir}}";

static STDLIB_CACHE: OnceLock<HashSet<String>> = OnceLock::new();

fn get_stdlib_packages() -> &'static HashSet<String> {
    STDLIB_CACHE.get_or_init(|| {
        query_go_stdlib().expect(
            "Failed to query Go stdlib packages. Go tooling is required for dependency discovery.",
        )
    })
}

fn query_go_stdlib() -> Result<HashSet<String>, LoadError> {
    let output = Command::new(GO_COMMAND)
        .args(GO_LIST_STD_ARGS)
        .output()
        .map_err(|e| LoadError::PackageManager(format!("Failed to run 'go list std': {e}")))?;

    if !output.status.success() {
        return Err(LoadError::PackageManager("go list std failed".to_string()));
    }

    let stdout = str::from_utf8(&output.stdout)
        .map_err(|e| LoadError::PackageManager(format!("Invalid UTF-8 from go list std: {e}")))?;

    let mut packages = HashSet::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            packages.insert(trimmed.to_string());
            let root = trimmed.split('/').next().unwrap_or(trimmed);
            if root != trimmed {
                packages.insert(root.to_string());
            }
        }
    }

    Ok(packages)
}

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

        walk_source_files(root_path, "go", EXCLUDED_DIRS, false)
    }

    fn load_dependencies(&self, root: &Path) -> Result<Vec<PathBuf>, LoadError> {
        let root_path = root;

        // Find all vendor directories recursively (handles nested vendor dirs like apis/vendor/)
        let vendor_dirs = self.find_all_vendor_dirs(root_path)?;

        if !vendor_dirs.is_empty() {
            let mut all_files = Vec::new();
            for vendor_path in vendor_dirs {
                let files = self.scan_vendor(&vendor_path)?;
                all_files.extend(files);
            }
            return Ok(all_files);
        }

        self.scan_dependencies_using_go_tooling(root_path)
    }
}

impl GoPackageLoader {
    fn find_all_vendor_dirs(&self, root: &Path) -> Result<Vec<PathBuf>, LoadError> {
        let mut vendor_dirs = Vec::new();

        fn walk_for_vendor(dir: &Path, vendor_dirs: &mut Vec<PathBuf>) -> std::io::Result<()> {
            if !dir.is_dir() {
                return Ok(());
            }

            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if !path.is_dir() {
                    continue;
                }

                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                // Skip hidden directories and other excluded dirs (but not vendor - we want to find it!)
                if dir_name.starts_with('.') || dir_name == "testdata" {
                    continue;
                }

                // Found a vendor directory
                if dir_name == "vendor" {
                    vendor_dirs.push(path);
                    // Don't recurse into vendor directories
                    continue;
                }

                // Recurse into other directories to find nested vendor dirs
                walk_for_vendor(&path, vendor_dirs)?;
            }

            Ok(())
        }

        walk_for_vendor(root, &mut vendor_dirs).map_err(LoadError::Io)?;

        Ok(vendor_dirs)
    }

    pub fn is_stdlib_package(&self, package_path: &str) -> bool {
        if package_path.starts_with(STD_PREFIX) || package_path.starts_with(INTERNAL_PREFIX) {
            return true;
        }

        if package_path.starts_with(VENDOR_PREFIX) {
            return true;
        }

        if !package_path.contains('.') {
            return self.is_known_stdlib_package(package_path);
        }

        false
    }

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
        walk_source_files(vendor_path, "go", &[], true)
    }

    fn scan_dependencies_using_go_tooling(
        &self,
        project_root: &Path,
    ) -> Result<Vec<PathBuf>, LoadError> {
        let go_mod_path = project_root.join("go.mod");

        if !go_mod_path.exists() {
            return Ok(vec![]);
        }

        let dependency_packages = self.get_dependency_packages(project_root)?;

        if dependency_packages.is_empty() {
            return Ok(vec![]);
        }

        self.resolve_package_paths_to_files(project_root, &dependency_packages)
    }

    fn get_dependency_packages(&self, project_root: &Path) -> Result<Vec<String>, LoadError> {
        let output = Command::new(GO_COMMAND)
            .args(GO_LIST_DEPS_ARGS)
            .args([GO_LIST_IMPORT_PATH_TEMPLATE, GO_LIST_PACKAGE_PATTERN])
            .current_dir(project_root)
            .output()
            .map_err(|e| LoadError::PackageManager(format!("Failed to run 'go list': {e}")))?;

        if !output.status.success() {
            let stderr = str::from_utf8(&output.stderr)
                .unwrap_or("Unknown error")
                .to_string();
            return Err(LoadError::PackageManager(format!(
                "go list failed: {stderr}"
            )));
        }

        let stdout = str::from_utf8(&output.stdout)
            .map_err(|e| LoadError::PackageManager(format!("Invalid UTF-8 from go list: {e}")))?;

        let mut packages: Vec<String> = stdout
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        packages.sort();
        packages.dedup();

        Ok(packages)
    }

    fn resolve_package_paths_to_files(
        &self,
        project_root: &Path,
        packages: &[String],
    ) -> Result<Vec<PathBuf>, LoadError> {
        let mut files = Vec::new();
        let mut processed = std::collections::HashSet::new();

        for package_path in packages {
            if self.is_stdlib_package(package_path) {
                continue;
            }

            if processed.contains(package_path) {
                continue;
            }
            processed.insert(package_path.to_string());

            if let Some(package_files) = self.get_package_files(project_root, package_path)? {
                files.extend(package_files);
            }
        }

        Ok(files)
    }

    fn is_known_stdlib_package(&self, package_path: &str) -> bool {
        let stdlib = get_stdlib_packages();
        let root_package = package_path.split('/').next().unwrap_or("");
        stdlib.contains(package_path) || stdlib.contains(root_package)
    }

    fn get_package_files(
        &self,
        project_root: &Path,
        package_path: &str,
    ) -> Result<Option<Vec<PathBuf>>, LoadError> {
        let output = Command::new(GO_COMMAND)
            .args(GO_LIST_DIR_ARGS)
            .args([GO_LIST_DIR_TEMPLATE, package_path])
            .current_dir(project_root)
            .output()
            .map_err(|e| {
                LoadError::PackageManager(format!(
                    "Failed to run 'go list' for {package_path}: {e}"
                ))
            })?;

        if !output.status.success() {
            return Ok(None);
        }

        let stdout = str::from_utf8(&output.stdout)
            .map_err(|e| LoadError::PackageManager(format!("Invalid UTF-8 from go list: {e}")))?;

        let dir_str = stdout.trim();
        if dir_str.is_empty() {
            return Ok(None);
        }

        let package_dir = PathBuf::from(dir_str);
        if !package_dir.exists() || !package_dir.is_dir() {
            return Ok(None);
        }

        let files = walk_source_files(&package_dir, "go", &[], true)?;

        if files.is_empty() {
            Ok(None)
        } else {
            Ok(Some(files))
        }
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

        assert!(!files.is_empty());
        assert!(files.iter().any(|f| f.to_string_lossy().contains("vendor")));
    }

    #[test]
    fn test_load_dependencies_finds_nested_vendor() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        // Create root vendor
        fs::create_dir_all(root.join("vendor/pkg1")).unwrap();
        fs::File::create(root.join("vendor/pkg1/lib1.go"))
            .unwrap()
            .write_all(b"package pkg1")
            .unwrap();

        // Create nested vendor (like apis/vendor in hive)
        fs::create_dir_all(root.join("apis/vendor/pkg2")).unwrap();
        fs::File::create(root.join("apis/vendor/pkg2/lib2.go"))
            .unwrap()
            .write_all(b"package pkg2")
            .unwrap();

        // Create another nested vendor
        fs::create_dir_all(root.join("submodule/vendor/pkg3")).unwrap();
        fs::File::create(root.join("submodule/vendor/pkg3/lib3.go"))
            .unwrap()
            .write_all(b"package pkg3")
            .unwrap();

        let loader = GoPackageLoader;
        let files = loader.load_dependencies(&root).unwrap();

        // Should find files from all vendor directories
        assert!(
            files.len() >= 3,
            "Expected at least 3 files, got {}",
            files.len()
        );
        assert!(files
            .iter()
            .any(|f| f.to_string_lossy().contains("vendor/pkg1")));
        assert!(files
            .iter()
            .any(|f| f.to_string_lossy().contains("apis/vendor/pkg2")));
        assert!(files
            .iter()
            .any(|f| f.to_string_lossy().contains("submodule/vendor/pkg3")));
    }

    #[test]
    fn test_find_all_vendor_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        // Create multiple vendor directories
        fs::create_dir_all(root.join("vendor")).unwrap();
        fs::create_dir_all(root.join("apis/vendor")).unwrap();
        fs::create_dir_all(root.join("cmd/app/vendor")).unwrap();

        let loader = GoPackageLoader;
        let vendor_dirs = loader.find_all_vendor_dirs(&root).unwrap();

        assert_eq!(
            vendor_dirs.len(),
            3,
            "Expected 3 vendor dirs, got {:?}",
            vendor_dirs
        );
    }

    #[test]
    fn test_exclude_stdlib_filters_goroot() {
        let temp_dir = TempDir::new().unwrap();
        let goroot = temp_dir.path().to_path_buf();

        unsafe { env::set_var("GOROOT", goroot.to_str().unwrap()) };

        let files = vec![
            goroot.join("src/crypto/aes/cipher.go"),
            PathBuf::from("/tmp/user/main.go"),
        ];

        let loader = GoPackageLoader;
        let filtered = loader.exclude_stdlib(files);

        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].to_string_lossy().contains("user"));

        unsafe { env::remove_var("GOROOT") };
    }

    #[test]
    fn test_is_stdlib_package_known_stdlib() {
        let loader = GoPackageLoader;
        assert!(loader.is_stdlib_package("fmt"));
        assert!(loader.is_stdlib_package("crypto/aes"));
        assert!(loader.is_stdlib_package("os"));
        assert!(loader.is_stdlib_package("net/http"));
        assert!(loader.is_stdlib_package("internal/poll"));
        assert!(loader.is_stdlib_package("std/encoding"));
    }

    #[test]
    fn test_is_stdlib_package_third_party() {
        let loader = GoPackageLoader;
        assert!(!loader.is_stdlib_package("github.com/user/pkg"));
        assert!(!loader.is_stdlib_package("golang.org/x/crypto"));
        assert!(!loader.is_stdlib_package("example.com/mypackage"));
    }

    #[test]
    fn test_is_stdlib_package_edge_cases() {
        let loader = GoPackageLoader;
        assert!(!loader.is_stdlib_package("mylocalpkg"));
        assert!(!loader.is_stdlib_package("unknownpkg"));
        assert!(loader.is_stdlib_package("vendor/some/pkg"));
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
