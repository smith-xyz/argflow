//! Python-specific integration tests

use super::test_utils::combine_user_and_dependencies;
use crate::discovery_tests::filtering::test_utils::filter_matching_files;
use crate::fixtures::get_test_fixture_path;
use argflow::discovery::cache::DiscoveryCache;
use argflow::discovery::languages::python::{PythonImportFilter, PythonPackageLoader};
use argflow::discovery::loader::PackageLoader;

#[test]
fn test_python_user_and_dependencies() {
    let test_app_path = get_test_fixture_path("python", Some("basic-crypto"));
    let loader = PythonPackageLoader;
    let filter = PythonImportFilter::from_bundled().expect("Failed to create filter");

    let user_files = loader
        .load_user_code(&test_app_path)
        .expect("Failed to load user code");

    let mut cache = DiscoveryCache::default();
    let dep_files = loader
        .load_dependencies(&test_app_path, &mut cache)
        .expect("Failed to load dependencies");

    let all_files = combine_user_and_dependencies(user_files, dep_files);

    assert!(
        !all_files.is_empty(),
        "Should find some files (user code or dependencies)"
    );

    let crypto_files = filter_matching_files(all_files, &filter);

    assert!(!crypto_files.is_empty(), "Should find crypto files");

    let user_crypto_count = crypto_files
        .iter()
        .filter(|f| matches!(f.source_type, argflow::discovery::SourceType::UserCode))
        .count();

    println!("Complete scan results:");
    println!("  User code crypto files: {user_crypto_count}");
    println!("  Total crypto files: {}", crypto_files.len());

    assert!(
        user_crypto_count >= 3,
        "Should find at least 3 user code crypto files (aes.py, sha.py, pbkdf2.py)"
    );
}
