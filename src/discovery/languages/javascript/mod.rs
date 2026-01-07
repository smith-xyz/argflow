use std::path::Path;

use crate::cli::Language;
use crate::discovery::filter::ImportFileFilter;
use crate::discovery::languages::LanguageModule;
use crate::discovery::loader::PackageLoader;

pub mod config;
pub mod deps;
pub mod filter;
pub mod loader;

pub use filter::JavaScriptImportFilter;
pub use loader::JavaScriptPackageLoader;

pub struct JavaScriptModule;

impl LanguageModule for JavaScriptModule {
    fn create_loader(&self) -> Box<dyn PackageLoader> {
        Box::new(JavaScriptPackageLoader)
    }

    fn create_filter(&self) -> Box<dyn ImportFileFilter> {
        Box::new(
            JavaScriptImportFilter::from_bundled()
                .expect("Failed to load bundled JavaScript import filter"),
        )
    }

    fn language(&self) -> Language {
        Language::Javascript
    }

    fn detect(&self, root: &Path) -> bool {
        root.join("package.json").exists()
            || root.join("yarn.lock").exists()
            || root.join("pnpm-lock.yaml").exists()
            || root.join("node_modules").exists()
    }
}
