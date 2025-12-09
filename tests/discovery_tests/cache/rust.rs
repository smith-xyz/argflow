//! Rust-specific cache tests

use super::test_utils::*;
use crypto_extractor_core::discovery::cache::DiscoveryCache;
use crypto_extractor_core::discovery::languages::rust::RustPackageLoader;
use crypto_extractor_core::discovery::loader::PackageLoader;

#[test]
fn test_rust_cache_functionality() {
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
    let mut cache1 = DiscoveryCache::default();
    let mut cache2 = DiscoveryCache::default();

    let files1 = loader
        .load_dependencies(root, &mut cache1)
        .expect("Failed to load dependencies");

    if files1.is_empty() {
        println!("NOTE: No dependencies found - skipping cache test");
        return;
    }

    let files2 = loader
        .load_dependencies(root, &mut cache2)
        .expect("Failed to load dependencies");

    assert_cache_consistency(&files1, &files2);
}
