//! JavaScript-specific user code discovery tests

use super::test_utils::*;
use argflow::discovery::languages::javascript::JavaScriptPackageLoader;
use argflow::discovery::loader::PackageLoader;

#[test]
fn test_javascript_user_code_discovery() {
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
    fs::File::create(root.join("src/utils.ts"))
        .unwrap()
        .write_all(b"export function helper() {}")
        .unwrap();

    let loader = JavaScriptPackageLoader;
    let files = loader
        .load_user_code(root)
        .expect("Failed to load user code");

    assert!(!files.is_empty(), "Should find JavaScript/TypeScript files");

    let file_names = get_file_names(&files, root);

    assert_file_found(&file_names, "crypto.js");
    assert_file_found(&file_names, "utils.ts");
}

#[test]
fn test_javascript_excluded_directories() {
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("testdata")).unwrap();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join("node_modules/dep")).unwrap();
    fs::create_dir_all(root.join("dist")).unwrap();

    fs::File::create(root.join("src/main.js"))
        .unwrap()
        .write_all(b"console.log('hello');")
        .unwrap();
    fs::File::create(root.join("testdata/test.js"))
        .unwrap()
        .write_all(b"describe('test');")
        .unwrap();
    fs::File::create(root.join(".git/config.js"))
        .unwrap()
        .write_all(b"module.exports = {};")
        .unwrap();
    fs::File::create(root.join("node_modules/dep/index.js"))
        .unwrap()
        .write_all(b"module.exports = {};")
        .unwrap();
    fs::File::create(root.join("dist/bundle.js"))
        .unwrap()
        .write_all(b"// bundle")
        .unwrap();

    let loader = JavaScriptPackageLoader;
    let user_files = loader
        .load_user_code(root)
        .expect("Failed to load user code");

    let file_names = get_file_names(&user_files, root);

    assert_file_found(&file_names, "main.js");
    assert_file_not_found(&file_names, "testdata");
    assert_file_not_found(&file_names, ".git");
    assert_file_not_found(&file_names, "node_modules");
    assert_file_not_found(&file_names, "dist");
}
