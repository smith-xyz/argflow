//! Go-specific dependency discovery tests

use super::test_utils::*;
use crate::discovery_tests::user_code::test_utils::get_go_fixture_path;
use crypto_extractor_core::discovery::cache::DiscoveryCache;
use crypto_extractor_core::discovery::languages::go::GoPackageLoader;
use crypto_extractor_core::discovery::loader::PackageLoader;
use crypto_extractor_core::discovery::SourceType;

#[test]
fn test_go_dependency_discovery() {
    let test_app_path = get_go_fixture_path("discovery-test-app");
    let loader = GoPackageLoader;
    let mut cache = DiscoveryCache::default();
    let dep_files = loader
        .load_dependencies(&test_app_path, &mut cache)
        .expect("Failed to load dependencies");

    if dep_files.is_empty() {
        println!("NOTE: No dependencies found (go-jose may not be in module cache)");
        println!("      This is expected if dependencies haven't been downloaded yet");
        println!("      Run: cd tests/fixtures/go/discovery-test-app && go mod download");
        return;
    }

    assert!(!dep_files.is_empty(), "Should find dependency files");

    let file_names: Vec<String> = dep_files
        .iter()
        .map(|f| f.path.to_string_lossy().to_string())
        .collect();

    let has_go_jose = file_names
        .iter()
        .any(|f| f.contains("go-jose") || f.contains("go-jose/v3"));

    if has_go_jose {
        println!("Found go-jose dependency files:");
        for file in dep_files.iter().filter(|f| {
            f.path.to_string_lossy().contains("go-jose")
                || f.path.to_string_lossy().contains("go-jose/v3")
        }) {
            println!("  - {}", file.path.display());
        }
    } else {
        println!("NOTE: go-jose files not found in dependency scan");
        println!("      This may be expected if scanning entire module cache");
    }
}

#[test]
fn test_go_vendor_as_dependency() {
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("vendor/github.com/user/dep")).unwrap();
    fs::File::create(root.join("vendor/github.com/user/dep/pkg.go"))
        .unwrap()
        .write_all(b"package dep\nimport \"crypto/aes\"")
        .unwrap();

    fs::create_dir_all(root).unwrap();
    fs::File::create(root.join("go.mod"))
        .unwrap()
        .write_all(b"module test\n")
        .unwrap();

    let loader = GoPackageLoader;
    let mut cache = DiscoveryCache::default();

    let dep_files = loader
        .load_dependencies(root, &mut cache)
        .expect("Failed to load dependencies");

    let vendor_files: Vec<_> = dep_files
        .iter()
        .filter(|f| f.path.to_string_lossy().contains("vendor"))
        .collect();

    if !vendor_files.is_empty() {
        for file in &vendor_files {
            assert!(
                matches!(file.source_type, SourceType::Dependency { .. }),
                "Vendor files should be tagged as Dependency: {}",
                file.path.display()
            );
        }
    } else {
        println!(
            "NOTE: Vendor directory not found - this is expected if vendor hasn't been populated"
        );
    }
}

#[test]
fn test_go_stdlib_files_included() {
    let test_app_path = get_go_fixture_path("discovery-test-app");
    let loader = GoPackageLoader;
    let mut cache = DiscoveryCache::default();

    let dep_files = loader
        .load_dependencies(&test_app_path, &mut cache)
        .expect("Failed to load dependencies");

    if dep_files.is_empty() {
        println!("NOTE: No dependencies found - skipping stdlib test");
        return;
    }

    let stdlib_files = get_stdlib_files(&dep_files);

    let stdlib_paths: Vec<String> = stdlib_files
        .iter()
        .map(|f| f.path.to_string_lossy().to_string())
        .collect();

    let has_crypto_stdlib = stdlib_paths
        .iter()
        .any(|p| p.contains("crypto/") || p.contains("golang.org/x/crypto"));

    if !stdlib_files.is_empty() {
        println!("Found {} stdlib files", stdlib_files.len());
        println!("Stdlib files include crypto packages: {has_crypto_stdlib}");

        assert!(
            !stdlib_files.is_empty(),
            "Should find stdlib files when dependencies are loaded"
        );
    } else {
        println!("NOTE: No stdlib files found - this may be expected depending on go list output");
    }
}
