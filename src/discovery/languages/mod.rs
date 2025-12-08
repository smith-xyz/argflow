use std::path::Path;

use crate::cli::Language;
use crate::discovery::filter::CryptoFileFilter;
use crate::discovery::loader::PackageLoader;

pub mod go;

pub use go::{GoCryptoFilter, GoPackageLoader};

pub trait LanguageModule: Send + Sync {
    fn create_loader(&self) -> Box<dyn PackageLoader>;

    fn create_filter(&self) -> Box<dyn CryptoFileFilter>;

    fn language(&self) -> Language;

    fn detect(&self, root: &Path) -> bool;
}

pub struct LanguageRegistry {
    modules: Vec<Box<dyn LanguageModule>>,
}

impl LanguageRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            modules: Vec::new(),
        };

        registry.register(Box::new(go::GoModule));
        registry
    }

    pub fn register(&mut self, module: Box<dyn LanguageModule>) {
        self.modules.push(module);
    }

    pub fn get_module(&self, language: Language) -> Option<&dyn LanguageModule> {
        self.modules
            .iter()
            .find(|m| m.language() == language)
            .map(|m| m.as_ref())
    }

    pub fn detect_languages(&self, root: &Path) -> Vec<Language> {
        self.modules
            .iter()
            .filter(|m| m.detect(root))
            .map(|m| m.language())
            .collect()
    }

    pub fn all_modules(&self) -> &[Box<dyn LanguageModule>] {
        &self.modules
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = LanguageRegistry::new();
        assert!(!registry.modules.is_empty());
    }

    #[test]
    fn test_get_module() {
        let registry = LanguageRegistry::new();
        assert!(registry.get_module(Language::Go).is_some());
    }
}
