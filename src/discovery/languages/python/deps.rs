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
        load_stdlib_from_mappings(
            Language::Python,
            &["hashlib", "hmac", "ssl", "secrets", "os", "crypto"],
        )
        .unwrap_or_else(|_| {
            let mut packages = HashSet::new();
            packages.insert("hashlib".to_string());
            packages.insert("hmac".to_string());
            packages.insert("ssl".to_string());
            packages.insert("secrets".to_string());
            packages.insert("os".to_string());
            packages
        })
    })
}

pub fn is_stdlib_package(package_path: &str) -> bool {
    if package_path.starts_with("_") {
        return true;
    }

    let stdlib = get_stdlib_packages();
    let root_package = package_path.split('.').next().unwrap_or("");
    stdlib.contains(package_path) || stdlib.contains(root_package)
}

pub fn scan_dependencies_using_python_tooling(
    project_root: &Path,
    _cache: &mut DiscoveryCache,
) -> Result<Vec<(PathBuf, bool)>, LoadError> {
    let mut files = Vec::new();

    if project_root.join("requirements.txt").exists() {
        if let Ok(pip_files) = scan_pip_dependencies(project_root) {
            files.extend(pip_files);
        }
    }

    if project_root.join("pyproject.toml").exists() || project_root.join("poetry.lock").exists() {
        if let Ok(poetry_files) = scan_poetry_dependencies(project_root) {
            files.extend(poetry_files);
        }
    }

    if project_root.join("uv.lock").exists() || project_root.join("pyproject.toml").exists() {
        if let Ok(uv_files) = scan_uv_dependencies(project_root) {
            files.extend(uv_files);
        }
    }

    if files.is_empty() {
        if let Ok(site_packages_files) = scan_site_packages(project_root) {
            files.extend(site_packages_files);
        }
    }

    Ok(files)
}

fn scan_pip_dependencies(project_root: &Path) -> Result<Vec<(PathBuf, bool)>, LoadError> {
    let output = Command::new(PIP_COMMAND)
        .args(["list", "--format=json"])
        .current_dir(project_root)
        .output()
        .map_err(|e| LoadError::PackageManager(format!("Failed to run 'pip list': {e}")))?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let stdout = str::from_utf8(&output.stdout)
        .map_err(|e| LoadError::PackageManager(format!("Invalid UTF-8 from pip list: {e}")))?;

    let packages: Vec<serde_json::Value> = serde_json::from_str(stdout)
        .map_err(|e| LoadError::PackageManager(format!("Failed to parse pip list output: {e}")))?;

    let mut files = Vec::new();
    for package in packages {
        if let (Some(name), Some(location)) = (
            package.get("name").and_then(|n| n.as_str()),
            package.get("location").and_then(|l| l.as_str()),
        ) {
            let package_path = PathBuf::from(location);
            if package_path.exists() {
                let is_stdlib = is_stdlib_package(name);
                if let Ok(package_files) =
                    walk_source_files(&package_path, FILE_EXTENSIONS[0], &[], true)
                {
                    for file in package_files {
                        files.push((file, is_stdlib));
                    }
                }
            }
        }
    }

    Ok(files)
}

fn scan_poetry_dependencies(project_root: &Path) -> Result<Vec<(PathBuf, bool)>, LoadError> {
    let output = Command::new(POETRY_COMMAND)
        .args(["show", "--no-ansi"])
        .current_dir(project_root)
        .output()
        .map_err(|e| LoadError::PackageManager(format!("Failed to run 'poetry show': {e}")))?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    Ok(vec![])
}

fn scan_uv_dependencies(project_root: &Path) -> Result<Vec<(PathBuf, bool)>, LoadError> {
    let output = Command::new(UV_COMMAND)
        .args(["pip", "list", "--format=json"])
        .current_dir(project_root)
        .output()
        .map_err(|e| LoadError::PackageManager(format!("Failed to run 'uv pip list': {e}")))?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let stdout = str::from_utf8(&output.stdout)
        .map_err(|e| LoadError::PackageManager(format!("Invalid UTF-8 from uv pip list: {e}")))?;

    let packages: Vec<serde_json::Value> = serde_json::from_str(stdout).map_err(|e| {
        LoadError::PackageManager(format!("Failed to parse uv pip list output: {e}"))
    })?;

    let mut files = Vec::new();
    for package in packages {
        if let (Some(name), Some(location)) = (
            package.get("name").and_then(|n| n.as_str()),
            package.get("location").and_then(|l| l.as_str()),
        ) {
            let package_path = PathBuf::from(location);
            if package_path.exists() {
                let is_stdlib = is_stdlib_package(name);
                if let Ok(package_files) =
                    walk_source_files(&package_path, FILE_EXTENSIONS[0], &[], true)
                {
                    for file in package_files {
                        files.push((file, is_stdlib));
                    }
                }
            }
        }
    }

    Ok(files)
}

fn scan_site_packages(project_root: &Path) -> Result<Vec<(PathBuf, bool)>, LoadError> {
    let venv_paths = vec![
        project_root.join("venv"),
        project_root.join(".venv"),
        project_root.join("env"),
    ];

    for venv_path in venv_paths {
        if venv_path.exists() {
            let site_packages = venv_path.join("lib").join("python3").join("site-packages");
            if site_packages.exists() {
                let mut files = Vec::new();
                if let Ok(package_files) =
                    walk_source_files(&site_packages, FILE_EXTENSIONS[0], &[], true)
                {
                    for file in package_files {
                        let is_stdlib = is_stdlib_package(
                            file.strip_prefix(&site_packages)
                                .unwrap_or(&file)
                                .components()
                                .next()
                                .and_then(|c| c.as_os_str().to_str())
                                .unwrap_or(""),
                        );
                        files.push((file, is_stdlib));
                    }
                }
                return Ok(files);
            }
        }
    }

    Ok(vec![])
}
