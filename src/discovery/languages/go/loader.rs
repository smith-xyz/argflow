use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::Language;
use crate::discovery::cache::DiscoveryCache;
use crate::discovery::loader::{LoadError, PackageLoader};
use crate::discovery::utils::walk_source_files;
use crate::discovery::{FileMetadata, SourceFile, SourceType};

use super::config::*;
use super::deps;

pub struct GoPackageLoader;

impl PackageLoader for GoPackageLoader {
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
                language: Language::Go,
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
        let cache_key = format!("{}:go", root.display());

        if let Some(cached_paths) = cache.get_dependencies(&cache_key) {
            return Ok(cached_paths
                .into_iter()
                .map(|path| {
                    let metadata = get_file_metadata(&path);
                    SourceFile {
                        path,
                        language: Language::Go,
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

        let vendor_dirs = find_all_vendor_dirs(root)?;

        let mut all_files = Vec::new();
        if !vendor_dirs.is_empty() {
            for vendor_path in vendor_dirs {
                let files = scan_vendor(&vendor_path)?;
                for file in files {
                    let metadata = get_file_metadata(&file);
                    all_files.push(SourceFile {
                        path: file,
                        language: Language::Go,
                        source_type: SourceType::Dependency {
                            package: "unknown".to_string(),
                            version: None,
                        },
                        package: None,
                        metadata,
                    });
                }
            }
        } else {
            let dep_results = deps::scan_dependencies_using_go_tooling(root, cache)?;
            for (path, is_stdlib) in dep_results {
                let metadata = get_file_metadata(&path);
                all_files.push(SourceFile {
                    path,
                    language: Language::Go,
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
        }

        let paths_for_cache: Vec<_> = all_files.iter().map(|f| f.path.clone()).collect();
        cache.set_dependencies(cache_key, paths_for_cache);

        Ok(all_files)
    }

    fn language(&self) -> Language {
        Language::Go
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

fn find_all_vendor_dirs(root: &Path) -> Result<Vec<PathBuf>, LoadError> {
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

            if dir_name.starts_with('.') || dir_name == "testdata" {
                continue;
            }

            if dir_name == "vendor" {
                vendor_dirs.push(path);
                continue;
            }

            walk_for_vendor(&path, vendor_dirs)?;
        }

        Ok(())
    }

    walk_for_vendor(root, &mut vendor_dirs).map_err(LoadError::Io)?;
    Ok(vendor_dirs)
}

fn scan_vendor(vendor_path: &Path) -> Result<Vec<PathBuf>, LoadError> {
    walk_source_files(vendor_path, FILE_EXTENSIONS[0], &[], true)
}
