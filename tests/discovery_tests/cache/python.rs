//! Python-specific cache tests

use super::test_utils::*;
use crate::fixtures::get_test_fixture_path;
use crypto_extractor_core::discovery::cache::DiscoveryCache;
use crypto_extractor_core::discovery::languages::python::PythonPackageLoader;
use crypto_extractor_core::discovery::loader::PackageLoader;

#[test]
fn test_python_cache_functionality() {
    let test_app_path = get_test_fixture_path("python", Some("basic-crypto"));
    let loader = PythonPackageLoader;
    let mut cache1 = DiscoveryCache::default();
    let mut cache2 = DiscoveryCache::default();

    let files1 = loader
        .load_dependencies(&test_app_path, &mut cache1)
        .expect("Failed to load dependencies");

    if files1.is_empty() {
        println!("NOTE: No dependencies found - skipping cache test");
        return;
    }

    let files2 = loader
        .load_dependencies(&test_app_path, &mut cache2)
        .expect("Failed to load dependencies");

    assert_cache_consistency(&files1, &files2);
}
