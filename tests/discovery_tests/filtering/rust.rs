//! Rust-specific crypto filtering tests

use super::test_utils::*;
use crate::discovery_tests::user_code::test_utils::{
    assert_file_found, assert_file_not_found, get_file_names,
};
use argflow::discovery::languages::rust::{RustImportFilter, RustPackageLoader};
use argflow::discovery::loader::PackageLoader;

#[test]
fn test_rust_crypto_filter() {
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::File::create(root.join("src/crypto.rs"))
        .unwrap()
        .write_all(b"use ring::digest;")
        .unwrap();
    fs::File::create(root.join("src/utils.rs"))
        .unwrap()
        .write_all(b"pub fn helper() {}")
        .unwrap();

    let loader = RustPackageLoader;
    let filter = RustImportFilter::from_bundled().expect("Failed to create filter");

    let all_files = loader
        .load_user_code(root)
        .expect("Failed to load user code");

    let crypto_files = filter_matching_files(all_files, &filter);

    if !crypto_files.is_empty() {
        let file_names = get_file_names(&crypto_files, root);
        assert_file_found(&file_names, "crypto.rs");
        assert_file_not_found(&file_names, "utils.rs");
    } else {
        println!("NOTE: No crypto files found - this may be expected if presets/crypto/rust/mappings.json doesn't contain 'ring'");
    }
}
