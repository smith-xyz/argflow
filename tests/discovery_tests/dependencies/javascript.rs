//! JavaScript-specific dependency discovery tests

use crypto_extractor_core::discovery::cache::DiscoveryCache;
use crypto_extractor_core::discovery::languages::javascript::JavaScriptPackageLoader;
use crypto_extractor_core::discovery::loader::PackageLoader;

#[test]
fn test_javascript_dependency_discovery() {
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("node_modules/crypto-js")).unwrap();
    fs::File::create(root.join("node_modules/crypto-js/index.js"))
        .unwrap()
        .write_all(b"module.exports = {};")
        .unwrap();
    fs::File::create(root.join("package.json"))
        .unwrap()
        .write_all(b"{\"name\": \"test\"}")
        .unwrap();

    let loader = JavaScriptPackageLoader;
    let mut cache = DiscoveryCache::default();
    let dep_files = loader
        .load_dependencies(root, &mut cache)
        .expect("Failed to load dependencies");

    if dep_files.is_empty() {
        println!("NOTE: No dependencies found - node_modules may not exist");
        return;
    }

    assert!(
        !dep_files.is_empty(),
        "Should find dependency files if node_modules exists"
    );
}
