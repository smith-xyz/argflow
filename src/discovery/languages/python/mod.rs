use std::path::Path;

use crate::cli::Language;
use crate::discovery::filter::ImportFileFilter;
use crate::discovery::languages::LanguageModule;
use crate::discovery::loader::PackageLoader;

pub mod config;
pub mod deps;
pub mod filter;
pub mod loader;

pub use filter::PythonImportFilter;
pub use loader::PythonPackageLoader;

pub struct PythonModule;

impl LanguageModule for PythonModule {
    fn create_loader(&self) -> Box<dyn PackageLoader> {
        Box::new(PythonPackageLoader)
    }

    fn create_filter(&self) -> Box<dyn ImportFileFilter> {
        Box::new(
            PythonImportFilter::from_bundled()
                .expect("Failed to load bundled Python import filter"),
        )
    }

    fn language(&self) -> Language {
        Language::Python
    }

    fn detect(&self, root: &Path) -> bool {
        root.join("requirements.txt").exists()
            || root.join("pyproject.toml").exists()
            || root.join("setup.py").exists()
            || root.join("Pipfile").exists()
            || root.join("poetry.lock").exists()
            || root.join("uv.lock").exists()
    }
}
