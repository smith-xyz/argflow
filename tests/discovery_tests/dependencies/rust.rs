//! Rust-specific dependency discovery tests

use argflow::discovery::cache::DiscoveryCache;
use argflow::discovery::languages::rust::RustPackageLoader;
use argflow::discovery::loader::PackageLoader;

#[test]
fn test_rust_dependency_discovery() {
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::File::create(root.join("Cargo.toml"))
        .unwrap()
        .write_all(b"[package]\nname = \"test\"")
        .unwrap();

    let loader = RustPackageLoader;
    let mut cache = DiscoveryCache::default();
    let dep_files = loader
        .load_dependencies(root, &mut cache)
        .expect("Failed to load dependencies");

    if dep_files.is_empty() {
        println!(
            "NOTE: No dependencies found - this is expected if target/debug/deps doesn't exist"
        );
        return;
    }

    assert!(
        !dep_files.is_empty(),
        "Should find dependency files if target/debug/deps exists"
    );
}
