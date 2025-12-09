use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
use std::sync::OnceLock;

use crate::cli::Language;
use crate::discovery::cache::DiscoveryCache;
use crate::discovery::loader::LoadError;
use crate::discovery::utils::{load_stdlib_from_mappings, walk_source_files};

use super::config::*;

static STDLIB_CACHE: OnceLock<HashSet<String>> = OnceLock::new();

fn get_stdlib_packages() -> &'static HashSet<String> {
    STDLIB_CACHE.get_or_init(|| {
        load_stdlib_from_mappings(Language::Rust, &["std", "core"]).unwrap_or_else(|_| {
            let mut packages = HashSet::new();
            packages.insert("std".to_string());
            packages.insert("core".to_string());
            packages
        })
    })
}

pub fn is_stdlib_package(package_path: &str) -> bool {
    if package_path.starts_with("std::") || package_path.starts_with("core::") {
        return true;
    }

    let stdlib = get_stdlib_packages();
    let root_package = package_path.split("::").next().unwrap_or("");
    stdlib.contains(package_path) || stdlib.contains(root_package)
}

pub fn scan_dependencies_using_cargo_tooling(
    project_root: &Path,
    _cache: &mut DiscoveryCache,
) -> Result<Vec<(PathBuf, bool)>, LoadError> {
    let cargo_toml = project_root.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Ok(vec![]);
    }

    let output = Command::new(CARGO_COMMAND)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(project_root)
        .output()
        .map_err(|e| LoadError::PackageManager(format!("Failed to run 'cargo metadata': {e}")))?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let stdout = str::from_utf8(&output.stdout).map_err(|e| {
        LoadError::PackageManager(format!("Invalid UTF-8 from cargo metadata: {e}"))
    })?;

    let _metadata: serde_json::Value = serde_json::from_str(stdout)
        .map_err(|e| LoadError::PackageManager(format!("Failed to parse cargo metadata: {e}")))?;

    let target_dir = project_root.join("target");
    if !target_dir.exists() {
        return Ok(vec![]);
    }

    let deps_dir = target_dir.join("debug").join("deps");
    if !deps_dir.exists() {
        return Ok(vec![]);
    }

    let mut files = Vec::new();
    if let Ok(package_files) = walk_source_files(&deps_dir, FILE_EXTENSIONS[0], &[], true) {
        for file in package_files {
            let is_stdlib = is_stdlib_package(
                file.strip_prefix(&deps_dir)
                    .unwrap_or(&file)
                    .components()
                    .next()
                    .and_then(|c| c.as_os_str().to_str())
                    .unwrap_or(""),
            );
            files.push((file, is_stdlib));
        }
    }

    Ok(files)
}
