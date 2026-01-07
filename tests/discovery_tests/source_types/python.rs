//! Python-specific source type tagging tests

use super::test_utils::*;
use crate::fixtures::get_test_fixture_path;
use argflow::discovery::cache::DiscoveryCache;
use argflow::discovery::languages::python::PythonPackageLoader;
use argflow::discovery::loader::PackageLoader;

#[test]
fn test_python_source_type_tagging() {
    let test_app_path = get_test_fixture_path("python", Some("basic-crypto"));
    let loader = PythonPackageLoader;
    let mut cache = DiscoveryCache::default();

    let user_files = loader
        .load_user_code(&test_app_path)
        .expect("Failed to load user code");

    let dep_files = loader
        .load_dependencies(&test_app_path, &mut cache)
        .expect("Failed to load dependencies");

    assert_all_user_code_tagged(&user_files);
    assert_all_dependencies_tagged(&dep_files);

    for file in &user_files {
        assert_eq!(
            file.language,
            argflow::cli::Language::Python,
            "All files should be tagged with correct language"
        );
    }
}
