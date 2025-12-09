use std::path::Path;

use crate::cli::Language;
use crate::discovery::filter::CryptoFileFilter;
use crate::discovery::languages::LanguageModule;
use crate::discovery::loader::PackageLoader;

pub mod config;
pub mod deps;
pub mod filter;
pub mod loader;

pub use filter::RustCryptoFilter;
pub use loader::RustPackageLoader;

pub struct RustModule;

impl LanguageModule for RustModule {
    fn create_loader(&self) -> Box<dyn PackageLoader> {
        Box::new(RustPackageLoader)
    }

    fn create_filter(&self) -> Box<dyn CryptoFileFilter> {
        Box::new(RustCryptoFilter)
    }

    fn language(&self) -> Language {
        Language::Rust
    }

    fn detect(&self, root: &Path) -> bool {
        root.join("Cargo.toml").exists() || root.join("Cargo.lock").exists()
    }
}
