//! JavaScript-specific integration tests

use super::test_utils::combine_user_and_dependencies;
use crate::discovery_tests::filtering::test_utils::filter_crypto_files;
use crypto_extractor_core::discovery::cache::DiscoveryCache;
use crypto_extractor_core::discovery::languages::javascript::{
    JavaScriptCryptoFilter, JavaScriptPackageLoader,
};
use crypto_extractor_core::discovery::loader::PackageLoader;

#[test]
fn test_javascript_user_and_dependencies() {
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

    let loader = JavaScriptPackageLoader;
    let filter = JavaScriptCryptoFilter;

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

    assert!(!crypto_files.is_empty(), "Should find crypto files");

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

    assert!(
        user_crypto_count >= 1,
        "Should find at least 1 user code crypto file"
    );
}
