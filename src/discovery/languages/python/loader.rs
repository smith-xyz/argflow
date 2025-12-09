use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::Language;
use crate::discovery::cache::DiscoveryCache;
use crate::discovery::loader::{LoadError, PackageLoader};
use crate::discovery::utils::walk_source_files;
use crate::discovery::{FileMetadata, SourceFile, SourceType};

use super::config::*;
use super::deps;

pub struct PythonPackageLoader;

impl PackageLoader for PythonPackageLoader {
    fn load_user_code(&self, root: &Path) -> Result<Vec<SourceFile>, LoadError> {
        if !root.exists() {
            return Err(LoadError::InvalidPath(format!(
                "Path does not exist: {}",
                root.display()
            )));
        }

        if !root.is_dir() {
            return Err(LoadError::InvalidPath(format!(
                "Path is not a directory: {}",
                root.display()
            )));
        }

        let paths = walk_source_files(root, FILE_EXTENSIONS[0], EXCLUDED_DIRS, false)?;
        Ok(paths
            .into_iter()
            .map(|path| SourceFile {
                path: path.clone(),
                language: Language::Python,
                source_type: SourceType::UserCode,
                package: None,
                metadata: get_file_metadata(&path),
            })
            .collect())
    }

    fn load_dependencies(
        &self,
        root: &Path,
        cache: &mut DiscoveryCache,
    ) -> Result<Vec<SourceFile>, LoadError> {
        let cache_key = format!("{}:python", root.display());

        if let Some(cached_paths) = cache.get_dependencies(&cache_key) {
            return Ok(cached_paths
                .into_iter()
                .map(|path| {
                    let metadata = get_file_metadata(&path);
                    SourceFile {
                        path,
                        language: Language::Python,
                        source_type: SourceType::Dependency {
                            package: "unknown".to_string(),
                            version: None,
                        },
                        package: None,
                        metadata,
                    }
                })
                .collect());
        }

        let dep_results = deps::scan_dependencies_using_python_tooling(root, cache)?;
        let mut all_files = Vec::new();

        for (path, is_stdlib) in dep_results {
            let metadata = get_file_metadata(&path);
            all_files.push(SourceFile {
                path,
                language: Language::Python,
                source_type: if is_stdlib {
                    SourceType::Stdlib
                } else {
                    SourceType::Dependency {
                        package: "unknown".to_string(),
                        version: None,
                    }
                },
                package: None,
                metadata,
            });
        }

        let paths_for_cache: Vec<_> = all_files.iter().map(|f| f.path.clone()).collect();
        cache.set_dependencies(cache_key, paths_for_cache);

        Ok(all_files)
    }

    fn language(&self) -> Language {
        Language::Python
    }
}

fn get_file_metadata(path: &PathBuf) -> FileMetadata {
    fs::metadata(path)
        .ok()
        .map(|m| FileMetadata {
            size: m.len(),
            modified: m.modified().ok(),
            hash: None,
        })
        .unwrap_or_else(|| FileMetadata {
            size: 0,
            modified: None,
            hash: None,
        })
}
