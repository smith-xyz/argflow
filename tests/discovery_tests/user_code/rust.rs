//! Rust-specific user code discovery tests

use super::test_utils::*;
use argflow::discovery::languages::rust::RustPackageLoader;
use argflow::discovery::loader::PackageLoader;

#[test]
fn test_rust_user_code_discovery() {
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::File::create(root.join("src/main.rs"))
        .unwrap()
        .write_all(b"fn main() {}")
        .unwrap();
    fs::File::create(root.join("src/lib.rs"))
        .unwrap()
        .write_all(b"pub fn lib() {}")
        .unwrap();
    fs::File::create(root.join("Cargo.toml"))
        .unwrap()
        .write_all(b"[package]\nname = \"test\"")
        .unwrap();

    let loader = RustPackageLoader;
    let files = loader
        .load_user_code(root)
        .expect("Failed to load user code");

    assert!(!files.is_empty(), "Should find Rust files");

    let file_names = get_file_names(&files, root);

    assert_file_found(&file_names, "main.rs");
    assert_file_found(&file_names, "lib.rs");
}

#[test]
fn test_rust_excluded_directories() {
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("testdata")).unwrap();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join("target/debug")).unwrap();

    fs::File::create(root.join("src/main.rs"))
        .unwrap()
        .write_all(b"fn main() {}")
        .unwrap();
    fs::File::create(root.join("testdata/test.rs"))
        .unwrap()
        .write_all(b"#[test] fn test() {}")
        .unwrap();
    fs::File::create(root.join(".git/config.rs"))
        .unwrap()
        .write_all(b"fn config() {}")
        .unwrap();
    fs::File::create(root.join("target/debug/main.rs"))
        .unwrap()
        .write_all(b"fn main() {}")
        .unwrap();

    let loader = RustPackageLoader;
    let user_files = loader
        .load_user_code(root)
        .expect("Failed to load user code");

    let file_names = get_file_names(&user_files, root);

    assert_file_found(&file_names, "main.rs");
    assert_file_not_found(&file_names, "testdata");
    assert_file_not_found(&file_names, ".git");
    assert_file_not_found(&file_names, "target");
}
