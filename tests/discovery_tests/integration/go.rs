//! Go-specific integration tests

use super::test_utils::*;
use crate::discovery_tests::dependencies::test_utils::{
    get_dependency_files, get_stdlib_files, get_user_code_files,
};
use crate::discovery_tests::filtering::test_utils::filter_crypto_files;
use crate::discovery_tests::user_code::test_utils::get_go_fixture_path;
use crypto_extractor_core::discovery::cache::DiscoveryCache;
use crypto_extractor_core::discovery::languages::go::{GoCryptoFilter, GoPackageLoader};
use crypto_extractor_core::discovery::loader::PackageLoader;

#[test]
fn test_go_user_and_dependencies() {
    let test_app_path = get_go_fixture_path("discovery-test-app");
    let loader = GoPackageLoader;
    let filter = GoCryptoFilter;

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

    let crypto_files = filter_crypto_files(all_files, &filter);

    assert!(!crypto_files.is_empty(), "Should find crypto files");

    let user_crypto_count = crypto_files
        .iter()
        .filter(|f| {
            matches!(
                f.source_type,
                crypto_extractor_core::discovery::SourceType::UserCode
            )
        })
        .count();

    let dep_crypto_count = crypto_files.len() - user_crypto_count;

    println!("Complete scan results:");
    println!("  User code crypto files: {user_crypto_count}");
    println!("  Dependency crypto files: {dep_crypto_count}");
    println!("  Total crypto files: {}", crypto_files.len());

    assert!(
        user_crypto_count >= 3,
        "Should find at least 3 user code crypto files (pbkdf2.go, jose.go, aes.go)"
    );
}

#[test]
fn test_go_go_jose_imported() {
    let test_app_path = get_go_fixture_path("discovery-test-app");
    let loader = GoPackageLoader;
    let filter = GoCryptoFilter;

    let user_files = loader
        .load_user_code(&test_app_path)
        .expect("Failed to load user code");

    let user_crypto_files = filter_crypto_files(user_files, &filter);

    let has_jose_import = user_crypto_files
        .iter()
        .any(|f| f.path.to_string_lossy().contains("jose.go"));

    assert!(
        has_jose_import,
        "Should find jose.go file that imports go-jose"
    );

    let mut cache = DiscoveryCache::default();
    let dep_files = loader
        .load_dependencies(&test_app_path, &mut cache)
        .expect("Failed to load dependencies");

    if dep_files.is_empty() {
        println!("NOTE: No dependencies found - skipping go-jose dependency verification");
        return;
    }

    let crypto_dep_files = filter_crypto_files(dep_files, &filter);

    let go_jose_files: Vec<_> = crypto_dep_files
        .iter()
        .filter(|f| {
            let path = f.path.to_string_lossy();
            path.contains("go-jose") || path.contains("go-jose/v3")
        })
        .collect();

    assert!(
        !go_jose_files.is_empty(),
        "Should find go-jose dependency files when go-jose is imported in user code"
    );

    println!(
        "Found {} go-jose files with crypto usage:",
        go_jose_files.len()
    );
    for file in go_jose_files.iter().take(10) {
        println!("  - {}", file.path.display());
    }
    if go_jose_files.len() > 10 {
        println!("  ... and {} more", go_jose_files.len() - 10);
    }
}

#[test]
fn test_go_dependencies_included_in_scan() {
    let test_app_path = get_go_fixture_path("discovery-test-app");
    let loader = GoPackageLoader;
    let filter = GoCryptoFilter;
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

    println!("Discovery results:");
    println!("  User code files: {user_file_count}");
    println!("  Dependency files: {dep_file_count}");

    let all_files = combine_user_and_dependencies(user_files, dep_files);
    assert!(
        !all_files.is_empty(),
        "Should find files from both user code and dependencies"
    );

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

    println!("  Stdlib files: {}", stdlib_files.len());
    println!("  Third-party dependency files: {}", dependency_files.len());

    let crypto_files = filter_crypto_files(all_files, &filter);

    assert!(
        !crypto_files.is_empty(),
        "Should find crypto files in combined scan"
    );

    let user_crypto = get_user_code_files(&crypto_files);
    let dep_crypto = get_dependency_files(&crypto_files);
    let stdlib_crypto = get_stdlib_files(&crypto_files);

    println!("Crypto files found:");
    println!("  User code crypto files: {}", user_crypto.len());
    println!(
        "  Third-party dependency crypto files: {}",
        dep_crypto.len()
    );
    println!("  Stdlib crypto files: {}", stdlib_crypto.len());
    println!("  Total crypto files: {}", crypto_files.len());

    assert!(
        user_crypto.len() >= 3,
        "Should find at least 3 user code crypto files (pbkdf2.go, jose.go, aes.go)"
    );

    if dep_file_count > 0 {
        assert!(
            !dep_crypto.is_empty() || !stdlib_crypto.is_empty(),
            "When dependencies are found, should find crypto usage in dependencies or stdlib"
        );

        if !dep_crypto.is_empty() {
            println!("\nThird-party dependency crypto files found:");
            for file in dep_crypto.iter().take(5) {
                println!("  - {}", file.path.display());
            }
            if dep_crypto.len() > 5 {
                println!(
                    "  ... and {} more dependency crypto files",
                    dep_crypto.len() - 5
                );
            }
        }

        if !stdlib_crypto.is_empty() {
            println!("\nStdlib crypto files found:");
            for file in stdlib_crypto.iter().take(5) {
                println!("  - {}", file.path.display());
            }
            if stdlib_crypto.len() > 5 {
                println!(
                    "  ... and {} more stdlib crypto files",
                    stdlib_crypto.len() - 5
                );
            }
        }
    } else {
        println!(
            "\nNOTE: No dependencies found - this is expected if go mod download hasn't been run"
        );
        println!("      To test dependency scanning, run: cd tests/fixtures/go/discovery-test-app && go mod download");
    }

    assert!(
        crypto_files.len() >= user_crypto.len(),
        "Total crypto files should include both user code and dependencies when dependencies are available"
    );
}

#[test]
fn test_go_no_unnecessary_skipping() {
    let test_app_path = get_go_fixture_path("discovery-test-app");
    let loader = GoPackageLoader;
    let filter = GoCryptoFilter;
    let mut cache = DiscoveryCache::default();

    let user_files = loader
        .load_user_code(&test_app_path)
        .expect("Failed to load user code");

    let dep_files = loader
        .load_dependencies(&test_app_path, &mut cache)
        .expect("Failed to load dependencies");

    let all_files = combine_user_and_dependencies(user_files, dep_files);

    assert!(
        !all_files.is_empty(),
        "Should find files - nothing should be unnecessarily skipped"
    );

    let crypto_files = filter_crypto_files(all_files, &filter);

    assert!(
        !crypto_files.is_empty(),
        "Should find crypto files - crypto filtering should not skip valid files"
    );

    let user_crypto_count = crypto_files
        .iter()
        .filter(|f| {
            matches!(
                f.source_type,
                crypto_extractor_core::discovery::SourceType::UserCode
            )
        })
        .count();

    assert!(
        user_crypto_count >= 3,
        "Should find at least 3 user code crypto files - files should not be skipped"
    );
}
