//! Python-specific source type tagging tests

use super::test_utils::*;
use crate::discovery_tests::user_code::test_utils::get_python_fixture_path;
use crypto_extractor_core::discovery::cache::DiscoveryCache;
use crypto_extractor_core::discovery::languages::python::PythonPackageLoader;
use crypto_extractor_core::discovery::loader::PackageLoader;

#[test]
fn test_python_source_type_tagging() {
    let test_app_path = get_python_fixture_path("basic-crypto");
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
            crypto_extractor_core::cli::Language::Python,
            "All files should be tagged with correct language"
        );
    }
}
