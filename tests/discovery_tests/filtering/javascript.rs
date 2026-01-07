//! JavaScript-specific crypto filtering tests

use super::test_utils::*;
use crate::discovery_tests::user_code::test_utils::{
    assert_file_found, assert_file_not_found, get_file_names,
};
use argflow::discovery::languages::javascript::{JavaScriptImportFilter, JavaScriptPackageLoader};
use argflow::discovery::loader::PackageLoader;

#[test]
fn test_javascript_crypto_filter() {
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::File::create(root.join("src/crypto.js"))
        .unwrap()
        .write_all(b"const crypto = require('crypto');")
        .unwrap();
    fs::File::create(root.join("src/utils.js"))
        .unwrap()
        .write_all(b"export function helper() {}")
        .unwrap();

    let loader = JavaScriptPackageLoader;
    let filter = JavaScriptImportFilter::from_bundled().expect("Failed to create filter");

    let all_files = loader
        .load_user_code(root)
        .expect("Failed to load user code");

    let crypto_files = filter_matching_files(all_files, &filter);

    assert!(!crypto_files.is_empty(), "Should find crypto files");

    let file_names = get_file_names(&crypto_files, root);

    assert_file_found(&file_names, "crypto.js");
    assert_file_not_found(&file_names, "utils.js");
}
