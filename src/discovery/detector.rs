use std::fs;
use std::path::Path;

use crate::cli::Language;
use crate::discovery::cache::{CacheError, DiscoveryCache};

pub struct LanguageDetector {
    cache: DiscoveryCache,
}

impl LanguageDetector {
    pub fn new() -> Result<Self, CacheError> {
        Ok(Self {
            cache: DiscoveryCache::new()?,
        })
    }

    pub fn detect(&mut self, root: &Path) -> Vec<Language> {
        let root_path = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

        if let Some(cached) = self.cache.get_detection(&root_path) {
            return cached;
        }

        let mut detected = Vec::new();

        if self.detect_go(&root_path) {
            detected.push(Language::Go);
        }

        if self.detect_python(&root_path) {
            detected.push(Language::Python);
        }

        if self.detect_javascript(&root_path) {
            detected.push(Language::Javascript);
        }

        if self.detect_rust(&root_path) {
            detected.push(Language::Rust);
        }

        self.cache.set_detection(root_path, detected.clone());
        detected
    }

    fn detect_go(&self, root: &Path) -> bool {
        root.join("go.mod").exists() || root.join("go.sum").exists()
    }

    fn detect_python(&self, root: &Path) -> bool {
        root.join("requirements.txt").exists()
            || root.join("pyproject.toml").exists()
            || root.join("setup.py").exists()
            || root.join("Pipfile").exists()
            || root.join("poetry.lock").exists()
            || self.has_python_files(root)
    }

    fn detect_javascript(&self, root: &Path) -> bool {
        root.join("package.json").exists()
            || root.join("yarn.lock").exists()
            || root.join("pnpm-lock.yaml").exists()
            || root.join("node_modules").exists()
    }

    fn detect_rust(&self, root: &Path) -> bool {
        root.join("Cargo.toml").exists() || root.join("Cargo.lock").exists()
    }

    #[allow(clippy::only_used_in_recursion)]
    fn has_python_files(&self, root: &Path) -> bool {
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "py" {
                            return true;
                        }
                    }
                } else if path.is_dir() {
                    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if dir_name != "venv"
                        && dir_name != ".venv"
                        && dir_name != "__pycache__"
                        && self.has_python_files(&path)
                    {
                        return true;
                    }
                }
            }
        }
        false
    }
}

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            cache: DiscoveryCache::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_go() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::File::create(root.join("go.mod")).unwrap();

        let mut detector = LanguageDetector::default();
        let languages = detector.detect(root);

        assert!(languages.contains(&Language::Go));
    }

    #[test]
    fn test_detect_python() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::File::create(root.join("requirements.txt")).unwrap();

        let mut detector = LanguageDetector::default();
        let languages = detector.detect(root);

        assert!(languages.contains(&Language::Python));
    }

    #[test]
    fn test_detect_javascript() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::File::create(root.join("package.json")).unwrap();

        let mut detector = LanguageDetector::default();
        let languages = detector.detect(root);

        assert!(languages.contains(&Language::Javascript));
    }

    #[test]
    fn test_detect_rust() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::File::create(root.join("Cargo.toml")).unwrap();

        let mut detector = LanguageDetector::default();
        let languages = detector.detect(root);

        assert!(languages.contains(&Language::Rust));
    }

    #[test]
    fn test_detect_multiple_languages() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::File::create(root.join("go.mod")).unwrap();
        fs::File::create(root.join("package.json")).unwrap();

        let mut detector = LanguageDetector::default();
        let languages = detector.detect(root);

        assert!(languages.contains(&Language::Go));
        assert!(languages.contains(&Language::Javascript));
        assert_eq!(languages.len(), 2);
    }

    #[test]
    fn test_detect_python_files() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::File::create(root.join("main.py")).unwrap();

        let mut detector = LanguageDetector::default();
        let languages = detector.detect(root);

        assert!(languages.contains(&Language::Python));
    }
}
