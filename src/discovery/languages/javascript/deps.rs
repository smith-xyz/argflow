use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::cli::Language;
use crate::discovery::cache::DiscoveryCache;
use crate::discovery::loader::LoadError;
use crate::discovery::utils::{load_stdlib_from_mappings, walk_source_files};

use super::config::*;

static STDLIB_CACHE: OnceLock<HashSet<String>> = OnceLock::new();

fn get_stdlib_packages() -> &'static HashSet<String> {
    STDLIB_CACHE.get_or_init(|| {
        load_stdlib_from_mappings(Language::Javascript, &["crypto"]).unwrap_or_else(|_| {
            let mut packages = HashSet::new();
            packages.insert("crypto".to_string());
            packages
        })
    })
}

pub fn is_stdlib_package(package_path: &str) -> bool {
    let stdlib = get_stdlib_packages();
    let root_package = package_path.split('/').next().unwrap_or("");
    stdlib.contains(package_path) || stdlib.contains(root_package)
}

pub fn scan_dependencies_using_javascript_tooling(
    project_root: &Path,
    _cache: &mut DiscoveryCache,
) -> Result<Vec<(PathBuf, bool)>, LoadError> {
    let node_modules = project_root.join("node_modules");
    if !node_modules.exists() {
        return Ok(vec![]);
    }

    let mut files = Vec::new();

    for ext in FILE_EXTENSIONS {
        if let Ok(package_files) = walk_source_files(&node_modules, ext, &[], true) {
            for file in package_files {
                let is_stdlib = is_stdlib_package(
                    file.strip_prefix(&node_modules)
                        .unwrap_or(&file)
                        .components()
                        .next()
                        .and_then(|c| c.as_os_str().to_str())
                        .unwrap_or(""),
                );
                files.push((file, is_stdlib));
            }
        }
    }

    Ok(files)
}
