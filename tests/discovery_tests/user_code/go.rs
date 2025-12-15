//! Go-specific user code discovery tests

use crate::fixtures::get_test_fixture_path;

use super::test_utils::*;
use crypto_extractor_core::discovery::languages::go::GoPackageLoader;
use crypto_extractor_core::discovery::loader::PackageLoader;

#[test]
fn test_go_user_code_discovery() {
    let test_app_path = get_test_fixture_path("go", Some("discovery-test-app"));
    let loader = GoPackageLoader;
    let files = loader
        .load_user_code(&test_app_path)
        .expect("Failed to load user code");

    assert!(!files.is_empty(), "Should find Go files");

    let expected_files = vec![
        "pkg/auth/pbkdf2.go",
        "pkg/auth/jose.go",
        "pkg/encryption/aes.go",
        "pkg/utils/helper.go",
    ];

    let file_names = get_file_names(&files, &test_app_path);

    for expected in &expected_files {
        assert_file_found(&file_names, expected);
    }
}

#[test]
fn test_go_excluded_directories() {
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("pkg")).unwrap();
    fs::create_dir_all(root.join("testdata")).unwrap();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join("vendor/dep")).unwrap();

    fs::File::create(root.join("main.go"))
        .unwrap()
        .write_all(b"package main")
        .unwrap();
    fs::File::create(root.join("pkg/helper.go"))
        .unwrap()
        .write_all(b"package pkg")
        .unwrap();
    fs::File::create(root.join("testdata/test.go"))
        .unwrap()
        .write_all(b"package testdata")
        .unwrap();
    fs::File::create(root.join(".git/config.go"))
        .unwrap()
        .write_all(b"package git")
        .unwrap();
    fs::File::create(root.join("vendor/dep/pkg.go"))
        .unwrap()
        .write_all(b"package dep")
        .unwrap();

    let loader = GoPackageLoader;
    let user_files = loader
        .load_user_code(root)
        .expect("Failed to load user code");

    let file_names = get_file_names(&user_files, root);

    assert_file_found(&file_names, "main.go");
    assert_file_found(&file_names, "helper.go");
    assert_file_not_found(&file_names, "testdata");
    assert_file_not_found(&file_names, ".git");

    let vendor_in_user_code: Vec<_> = file_names.iter().filter(|f| f.contains("vendor")).collect();

    if !vendor_in_user_code.is_empty() {
        println!(
            "NOTE: Vendor files found in user code scan (vendor is not excluded from user code)"
        );
        println!("      Vendor files: {vendor_in_user_code:?}");
    }
}
