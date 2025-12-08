pub mod cache;
pub mod detector;
pub mod filter;
pub mod languages;
pub mod loader;
pub mod utils;

pub use cache::DiscoveryCache;
pub use detector::LanguageDetector;
pub use filter::CryptoFileFilter;
pub use languages::{GoCryptoFilter, GoPackageLoader, LanguageModule, LanguageRegistry};
pub use loader::PackageLoader;
pub use utils::walk_source_files;

use std::path::PathBuf;
use std::time::SystemTime;

use crate::cli::Language;

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: PathBuf,
    pub language: Language,
    pub source_type: SourceType,
    pub package: Option<String>,
    pub metadata: FileMetadata,
}

#[derive(Debug, Clone)]
pub enum SourceType {
    UserCode,
    Dependency {
        package: String,
        version: Option<String>,
    },
    Stdlib,
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub hash: Option<String>,
}
