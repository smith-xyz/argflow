//! Shared test utilities for user code discovery tests

use crypto_extractor_core::discovery::SourceFile;
use std::path::Path;

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
