//! Python-specific crypto filtering tests

use super::test_utils::*;
use crate::discovery_tests::user_code::test_utils::{
    assert_file_found, assert_file_not_found, get_file_names, get_python_fixture_path,
};
use crypto_extractor_core::discovery::languages::python::{
    PythonCryptoFilter, PythonPackageLoader,
};
use crypto_extractor_core::discovery::loader::PackageLoader;

#[test]
fn test_python_crypto_filter() {
    let test_app_path = get_python_fixture_path("basic-crypto");
    let loader = PythonPackageLoader;
    let filter = PythonCryptoFilter;

    let all_files = loader
        .load_user_code(&test_app_path)
        .expect("Failed to load user code");

    let crypto_files = filter_crypto_files(all_files, &filter);

    assert!(!crypto_files.is_empty(), "Should find crypto files");

    let file_names = get_file_names(&crypto_files, &test_app_path);

    assert_file_found(&file_names, "aes.py");
    assert_file_found(&file_names, "sha.py");
    assert_file_found(&file_names, "pbkdf2.py");
    assert_file_not_found(&file_names, "helpers.py");
}
