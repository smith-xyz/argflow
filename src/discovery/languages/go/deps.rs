use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
use std::sync::OnceLock;

use crate::discovery::cache::DiscoveryCache;
use crate::discovery::loader::LoadError;
use crate::discovery::utils::walk_source_files;

use super::config::*;

static STDLIB_CACHE: OnceLock<HashSet<String>> = OnceLock::new();

fn get_stdlib_packages() -> &'static HashSet<String> {
    STDLIB_CACHE.get_or_init(|| {
        query_go_stdlib().expect(
            "Failed to query Go stdlib packages. Go tooling is required for dependency discovery.",
        )
    })
}

fn query_go_stdlib() -> Result<HashSet<String>, LoadError> {
    let output = Command::new(GO_COMMAND)
        .args(GO_LIST_STD_ARGS)
        .output()
        .map_err(|e| LoadError::PackageManager(format!("Failed to run 'go list std': {e}")))?;

    if !output.status.success() {
        return Err(LoadError::PackageManager("go list std failed".to_string()));
    }

    let stdout = str::from_utf8(&output.stdout)
        .map_err(|e| LoadError::PackageManager(format!("Invalid UTF-8 from go list std: {e}")))?;

    let mut packages = HashSet::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            packages.insert(trimmed.to_string());
            let root = trimmed.split('/').next().unwrap_or(trimmed);
            if root != trimmed {
                packages.insert(root.to_string());
            }
        }
    }

    Ok(packages)
}

pub fn is_stdlib_package(package_path: &str) -> bool {
    if package_path.starts_with(STD_PREFIX) || package_path.starts_with(INTERNAL_PREFIX) {
        return true;
    }

    if !package_path.contains('.') {
        return is_known_stdlib_package(package_path);
    }

    false
}

fn is_known_stdlib_package(package_path: &str) -> bool {
    let stdlib = get_stdlib_packages();
    let root_package = package_path.split('/').next().unwrap_or("");
    stdlib.contains(package_path) || stdlib.contains(root_package)
}

pub fn scan_dependencies_using_go_tooling(
    project_root: &Path,
    _cache: &mut DiscoveryCache,
) -> Result<Vec<(PathBuf, bool)>, LoadError> {
    let go_mod_path = project_root.join("go.mod");

    if !go_mod_path.exists() {
        return Ok(vec![]);
    }

    let dependency_packages = get_dependency_packages(project_root)?;

    if dependency_packages.is_empty() {
        return Ok(vec![]);
    }

    resolve_package_paths_to_files(project_root, &dependency_packages)
}

fn get_dependency_packages(project_root: &Path) -> Result<Vec<String>, LoadError> {
    let output = Command::new(GO_COMMAND)
        .args(GO_LIST_DEPS_ARGS)
        .args([GO_LIST_IMPORT_PATH_TEMPLATE, GO_LIST_PACKAGE_PATTERN])
        .current_dir(project_root)
        .output()
        .map_err(|e| LoadError::PackageManager(format!("Failed to run 'go list': {e}")))?;

    if !output.status.success() {
        let stderr = str::from_utf8(&output.stderr)
            .unwrap_or("Unknown error")
            .to_string();
        return Err(LoadError::PackageManager(format!(
            "go list failed: {stderr}"
        )));
    }

    let stdout = str::from_utf8(&output.stdout)
        .map_err(|e| LoadError::PackageManager(format!("Invalid UTF-8 from go list: {e}")))?;

    let mut packages: Vec<String> = stdout
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    packages.sort();
    packages.dedup();

    Ok(packages)
}

fn resolve_package_paths_to_files(
    project_root: &Path,
    packages: &[String],
) -> Result<Vec<(PathBuf, bool)>, LoadError> {
    let mut files = Vec::new();
    let mut processed = HashSet::new();

    for package_path in packages {
        if processed.contains(package_path) {
            continue;
        }
        processed.insert(package_path.to_string());

        let is_stdlib = is_stdlib_package(package_path);
        if let Some(package_files) = get_package_files(project_root, package_path)? {
            for file in package_files {
                files.push((file, is_stdlib));
            }
        }
    }

    Ok(files)
}

fn get_package_files(
    project_root: &Path,
    package_path: &str,
) -> Result<Option<Vec<PathBuf>>, LoadError> {
    let output = Command::new(GO_COMMAND)
        .args(GO_LIST_DIR_ARGS)
        .args([GO_LIST_DIR_TEMPLATE, package_path])
        .current_dir(project_root)
        .output()
        .map_err(|e| {
            LoadError::PackageManager(format!("Failed to run 'go list' for {package_path}: {e}"))
        })?;

    if !output.status.success() {
        return Ok(None);
    }

    let stdout = str::from_utf8(&output.stdout)
        .map_err(|e| LoadError::PackageManager(format!("Invalid UTF-8 from go list: {e}")))?;

    let dir_str = stdout.trim();
    if dir_str.is_empty() {
        return Ok(None);
    }

    let package_dir = PathBuf::from(dir_str);
    if !package_dir.exists() || !package_dir.is_dir() {
        return Ok(None);
    }

    let files = walk_source_files(&package_dir, FILE_EXTENSIONS[0], &[], true)?;

    if files.is_empty() {
        Ok(None)
    } else {
        Ok(Some(files))
    }
}
