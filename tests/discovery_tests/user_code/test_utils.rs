//! Shared test utilities for user code discovery tests

use crypto_extractor_core::discovery::SourceFile;
use std::path::{Path, PathBuf};

pub fn get_go_fixture_path(fixture_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("go")
        .join(fixture_name)
}

pub fn get_python_fixture_path(fixture_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("python")
        .join(fixture_name)
}

pub fn get_javascript_fixture_path(fixture_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("javascript")
        .join(fixture_name)
}

pub fn get_rust_fixture_path(fixture_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("rust")
        .join(fixture_name)
}

pub fn get_file_names(files: &[SourceFile], base_path: &Path) -> Vec<String> {
    files
        .iter()
        .map(|f| {
            f.path
                .strip_prefix(base_path)
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect()
}

pub fn assert_file_found(file_names: &[String], expected: &str) {
    assert!(
        file_names.iter().any(|f| f.ends_with(expected)),
        "Should find {expected}"
    );
}

pub fn assert_file_not_found(file_names: &[String], not_expected: &str) {
    assert!(
        !file_names.iter().any(|f| f.contains(not_expected)),
        "Should NOT find {not_expected}"
    );
}
