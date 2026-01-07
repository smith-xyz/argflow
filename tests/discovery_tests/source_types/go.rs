//! Go-specific source type tagging tests

use super::test_utils::*;
use crate::discovery_tests::dependencies::test_utils::{
    get_dependency_files, get_stdlib_files, get_user_code_files,
};
use crate::fixtures::get_test_fixture_path;
use argflow::discovery::cache::DiscoveryCache;
use argflow::discovery::languages::go::GoPackageLoader;
use argflow::discovery::loader::PackageLoader;

#[test]
fn test_go_source_type_tagging() {
    let test_app_path = get_test_fixture_path("go", Some("discovery-test-app"));
    let loader = GoPackageLoader;
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
            argflow::cli::Language::Go,
            "All files should be tagged with correct language"
        );
    }
}

#[test]
fn test_go_source_type_counts() {
    let test_app_path = get_test_fixture_path("go", Some("discovery-test-app"));
    let loader = GoPackageLoader;
    let mut cache = DiscoveryCache::default();

    let user_files = loader
        .load_user_code(&test_app_path)
        .expect("Failed to load user code");

    let dep_files = loader
        .load_dependencies(&test_app_path, &mut cache)
        .expect("Failed to load dependencies");

    assert!(!user_files.is_empty(), "Should find user code files");

    let user_file_count = user_files.len();
    let dep_file_count = dep_files.len();

    let all_files: Vec<_> = user_files.into_iter().chain(dep_files).collect();

    let user_code_files = get_user_code_files(&all_files);
    let dependency_files = get_dependency_files(&all_files);
    let stdlib_files = get_stdlib_files(&all_files);

    assert_eq!(
        user_code_files.len(),
        user_file_count,
        "User code files should be tagged as UserCode"
    );
    assert_eq!(
        dependency_files.len() + stdlib_files.len(),
        dep_file_count,
        "Dependency and stdlib files should equal total dependency file count"
    );
}
