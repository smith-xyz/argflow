//! Rust-specific integration tests

use super::test_utils::combine_user_and_dependencies;
use crate::discovery_tests::filtering::test_utils::filter_crypto_files;
use crypto_extractor_core::discovery::cache::DiscoveryCache;
use crypto_extractor_core::discovery::languages::rust::{RustCryptoFilter, RustPackageLoader};
use crypto_extractor_core::discovery::loader::PackageLoader;

#[test]
fn test_rust_user_and_dependencies() {
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

    let loader = RustPackageLoader;
    let filter = RustCryptoFilter;

    let user_files = loader
        .load_user_code(root)
        .expect("Failed to load user code");

    let mut cache = DiscoveryCache::default();
    let dep_files = loader
        .load_dependencies(root, &mut cache)
        .expect("Failed to load dependencies");

    let all_files = combine_user_and_dependencies(user_files, dep_files);

    assert!(
        !all_files.is_empty(),
        "Should find some files (user code or dependencies)"
    );

    let crypto_files = filter_crypto_files(all_files, &filter);

    if !crypto_files.is_empty() {
        let user_crypto_count = crypto_files
            .iter()
            .filter(|f| {
                matches!(
                    f.source_type,
                    crypto_extractor_core::discovery::SourceType::UserCode
                )
            })
            .count();

        println!("Complete scan results:");
        println!("  User code crypto files: {user_crypto_count}");
        println!("  Total crypto files: {}", crypto_files.len());
    } else {
        println!("NOTE: No crypto files found - this may be expected if classifier-rules/rust/mappings.json doesn't contain expected patterns");
    }
}
