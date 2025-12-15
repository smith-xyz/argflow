//! Go-specific crypto filtering tests

use super::test_utils::*;
use crate::discovery_tests::user_code::test_utils::{
    assert_file_found, assert_file_not_found, get_file_names,
};
use crate::fixtures::get_test_fixture_path;
use crypto_extractor_core::discovery::languages::go::{GoCryptoFilter, GoPackageLoader};
use crypto_extractor_core::discovery::loader::PackageLoader;

#[test]
fn test_go_crypto_filter() {
    let test_app_path = get_test_fixture_path("go", Some("discovery-test-app"));
    let loader = GoPackageLoader;
    let filter = GoCryptoFilter;

    let all_files = loader
        .load_user_code(&test_app_path)
        .expect("Failed to load user code");

    let crypto_files = filter_crypto_files(all_files, &filter);

    assert!(!crypto_files.is_empty(), "Should find crypto files");

    let file_names = get_file_names(&crypto_files, &test_app_path);

    assert_file_found(&file_names, "pbkdf2.go");
    assert_file_found(&file_names, "jose.go");
    assert_file_found(&file_names, "aes.go");
    assert_file_not_found(&file_names, "helper.go");
}

#[test]
fn test_go_crypto_filter_exact_count() {
    let test_app_path = get_test_fixture_path("go", Some("discovery-test-app"));
    let loader = GoPackageLoader;
    let filter = GoCryptoFilter;

    let all_files = loader
        .load_user_code(&test_app_path)
        .expect("Failed to load user code");

    let crypto_files = filter_crypto_files(all_files, &filter);

    assert_eq!(
        crypto_files.len(),
        3,
        "Should find exactly 3 crypto files (pbkdf2.go, jose.go, and aes.go)"
    );

    println!("Files that would be scanned:");
    for file in &crypto_files {
        println!("  - {}", file.path.display());
    }
}
