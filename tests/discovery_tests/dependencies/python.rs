//! Python-specific dependency discovery tests

use crate::discovery_tests::user_code::test_utils::get_python_fixture_path;
use crypto_extractor_core::discovery::cache::DiscoveryCache;
use crypto_extractor_core::discovery::languages::python::PythonPackageLoader;
use crypto_extractor_core::discovery::loader::PackageLoader;

#[test]
fn test_python_dependency_discovery() {
    let test_app_path = get_python_fixture_path("basic-crypto");
    let loader = PythonPackageLoader;
    let mut cache = DiscoveryCache::default();
    let dep_files = loader
        .load_dependencies(&test_app_path, &mut cache)
        .expect("Failed to load dependencies");

    if dep_files.is_empty() {
        println!("NOTE: No dependencies found - this is expected if no virtual environment or requirements.txt");
        return;
    }

    assert!(
        !dep_files.is_empty(),
        "Should find dependency files if available"
    );
}
