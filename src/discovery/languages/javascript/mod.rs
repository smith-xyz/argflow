use std::path::Path;

use crate::cli::Language;
use crate::discovery::filter::CryptoFileFilter;
use crate::discovery::languages::LanguageModule;
use crate::discovery::loader::PackageLoader;

pub mod config;
pub mod deps;
pub mod filter;
pub mod loader;

pub use filter::JavaScriptCryptoFilter;
pub use loader::JavaScriptPackageLoader;

pub struct JavaScriptModule;

impl LanguageModule for JavaScriptModule {
    fn create_loader(&self) -> Box<dyn PackageLoader> {
        Box::new(JavaScriptPackageLoader)
    }

    fn create_filter(&self) -> Box<dyn CryptoFileFilter> {
        Box::new(JavaScriptCryptoFilter)
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
