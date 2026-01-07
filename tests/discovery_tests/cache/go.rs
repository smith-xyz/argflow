//! Go-specific cache tests

use crate::fixtures::get_test_fixture_path;

use super::test_utils::*;
use argflow::discovery::cache::DiscoveryCache;
use argflow::discovery::languages::go::GoPackageLoader;
use argflow::discovery::loader::PackageLoader;

#[test]
fn test_go_cache_functionality() {
    let test_app_path = get_test_fixture_path("go", Some("discovery-test-app"));
    let loader = GoPackageLoader;
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
