//! JavaScript-specific source type tagging tests

use super::test_utils::*;
use argflow::discovery::cache::DiscoveryCache;
use argflow::discovery::languages::javascript::JavaScriptPackageLoader;
use argflow::discovery::loader::PackageLoader;

#[test]
fn test_javascript_source_type_tagging() {
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::File::create(root.join("src/main.js"))
        .unwrap()
        .write_all(b"console.log('hello');")
        .unwrap();

    let loader = JavaScriptPackageLoader;
    let mut cache = DiscoveryCache::default();

    let user_files = loader
        .load_user_code(root)
        .expect("Failed to load user code");

    let dep_files = loader
        .load_dependencies(root, &mut cache)
        .expect("Failed to load dependencies");

    assert_all_user_code_tagged(&user_files);
    assert_all_dependencies_tagged(&dep_files);

    for file in &user_files {
        assert_eq!(
            file.language,
            argflow::cli::Language::Javascript,
            "All files should be tagged with correct language"
        );
    }
}
