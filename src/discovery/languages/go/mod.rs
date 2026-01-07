use std::path::Path;

use crate::cli::Language;
use crate::discovery::filter::ImportFileFilter;
use crate::discovery::languages::LanguageModule;
use crate::discovery::loader::PackageLoader;

pub mod config;
pub mod deps;
pub mod filter;
pub mod loader;

pub use filter::GoImportFilter;
pub use loader::GoPackageLoader;

pub struct GoModule;

impl LanguageModule for GoModule {
    fn create_loader(&self) -> Box<dyn PackageLoader> {
        Box::new(GoPackageLoader)
    }

    fn create_filter(&self) -> Box<dyn ImportFileFilter> {
        Box::new(GoImportFilter::from_bundled().expect("Failed to load bundled Go import filter"))
    }

    fn language(&self) -> Language {
        Language::Go
    }

    fn detect(&self, root: &Path) -> bool {
        root.join("go.mod").exists() || root.join("go.sum").exists()
    }
}
