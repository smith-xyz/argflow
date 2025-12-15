//! Python-specific user code discovery tests

use super::test_utils::*;
use crate::fixtures::get_test_fixture_path;
use crypto_extractor_core::discovery::languages::python::PythonPackageLoader;
use crypto_extractor_core::discovery::loader::PackageLoader;

#[test]
fn test_python_user_code_discovery() {
    let test_app_path = get_test_fixture_path("python", Some("basic-crypto"));
    let loader = PythonPackageLoader;
    let files = loader
        .load_user_code(&test_app_path)
        .expect("Failed to load user code");

    assert!(!files.is_empty(), "Should find Python files");

    let expected_files = vec![
        "cipher/aes.py",
        "hash/sha.py",
        "kdf/pbkdf2.py",
        "utils/helpers.py",
    ];

    let file_names = get_file_names(&files, &test_app_path);

    for expected in &expected_files {
        assert_file_found(&file_names, expected);
    }
}

#[test]
fn test_python_excluded_directories() {
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("pkg")).unwrap();
    fs::create_dir_all(root.join("testdata")).unwrap();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join("__pycache__")).unwrap();

    fs::File::create(root.join("main.py"))
        .unwrap()
        .write_all(b"import os")
        .unwrap();
    fs::File::create(root.join("pkg/helper.py"))
        .unwrap()
        .write_all(b"def helper(): pass")
        .unwrap();
    fs::File::create(root.join("testdata/test.py"))
        .unwrap()
        .write_all(b"def test(): pass")
        .unwrap();
    fs::File::create(root.join(".git/config.py"))
        .unwrap()
        .write_all(b"def config(): pass")
        .unwrap();
    fs::File::create(root.join("__pycache__/cache.py"))
        .unwrap()
        .write_all(b"def cache(): pass")
        .unwrap();

    let loader = PythonPackageLoader;
    let user_files = loader
        .load_user_code(root)
        .expect("Failed to load user code");

    let file_names = get_file_names(&user_files, root);

    assert_file_found(&file_names, "main.py");
    assert_file_found(&file_names, "helper.py");
    assert_file_not_found(&file_names, "testdata");
    assert_file_not_found(&file_names, ".git");
    assert_file_not_found(&file_names, "__pycache__");
}
