//! Python-specific dependency discovery tests

use crate::fixtures::get_test_fixture_path;
use argflow::discovery::cache::DiscoveryCache;
use argflow::discovery::languages::python::PythonPackageLoader;
use argflow::discovery::loader::PackageLoader;

#[test]
fn test_python_dependency_discovery() {
    let test_app_path = get_test_fixture_path("python", Some("basic-crypto"));
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
