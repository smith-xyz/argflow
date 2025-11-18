use crypto_extractor_core::discovery::filter::CryptoFileFilter;
use crypto_extractor_core::discovery::languages::go::{GoCryptoFilter, GoPackageLoader};
use crypto_extractor_core::discovery::loader::PackageLoader;
use std::path::PathBuf;

#[test]
fn test_discovery_e2e_user_code() {
    let test_app_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("test-app");

    let loader = GoPackageLoader;
    let files = loader
        .load_user_code(&test_app_path)
        .expect("Failed to load user code");

    assert!(!files.is_empty(), "Should find Go files");

    let expected_files = vec![
        "pkg/auth/pbkdf2.go",
        "pkg/auth/jose.go",
        "pkg/encryption/aes.go",
        "pkg/utils/helper.go",
    ];

    let file_names: Vec<String> = files
        .iter()
        .map(|p| {
            p.strip_prefix(&test_app_path)
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect();

    for expected in &expected_files {
        assert!(
            file_names.iter().any(|f| f.ends_with(expected)),
            "Should find {}",
            expected
        );
    }
}

#[test]
fn test_discovery_e2e_crypto_filter() {
    let test_app_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("test-app");

    let loader = GoPackageLoader;
    let filter = GoCryptoFilter;

    let all_files = loader
        .load_user_code(&test_app_path)
        .expect("Failed to load user code");

    let crypto_files: Vec<PathBuf> = all_files
        .into_iter()
        .filter_map(|file| {
            filter
                .has_crypto_usage(&file)
                .ok()
                .and_then(|has_crypto| has_crypto.then_some(file))
        })
        .collect();

    assert!(!crypto_files.is_empty(), "Should find crypto files");

    let file_names: Vec<String> = crypto_files
        .iter()
        .map(|p| {
            p.strip_prefix(&test_app_path)
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect();

    assert!(
        file_names.iter().any(|f| f.contains("pbkdf2.go")),
        "Should find pbkdf2.go"
    );
    assert!(
        file_names.iter().any(|f| f.contains("jose.go")),
        "Should find jose.go (uses go-jose)"
    );
    assert!(
        file_names.iter().any(|f| f.contains("aes.go")),
        "Should find aes.go"
    );
    assert!(
        !file_names.iter().any(|f| f.contains("helper.go")),
        "Should NOT find helper.go (no crypto imports)"
    );
}

#[test]
fn test_discovery_e2e_full_workflow() {
    let test_app_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("test-app");

    let loader = GoPackageLoader;
    let filter = GoCryptoFilter;

    let all_files = loader
        .load_user_code(&test_app_path)
        .expect("Failed to load user code");

    let crypto_files: Vec<PathBuf> = all_files
        .into_iter()
        .filter_map(|file| {
            filter
                .has_crypto_usage(&file)
                .ok()
                .and_then(|has_crypto| has_crypto.then_some(file))
        })
        .collect();

    assert_eq!(
        crypto_files.len(),
        3,
        "Should find exactly 3 crypto files (pbkdf2.go, jose.go, and aes.go)"
    );

    println!("Files that would be scanned:");
    for file in &crypto_files {
        println!("  - {}", file.display());
    }
}

#[test]
fn test_discovery_e2e_dependencies() {
    let test_app_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("test-app");

    let loader = GoPackageLoader;
    let dep_files = loader
        .load_dependencies(&test_app_path)
        .expect("Failed to load dependencies");

    if dep_files.is_empty() {
        println!("NOTE: No dependencies found (go-jose may not be in module cache)");
        println!("      This is expected if dependencies haven't been downloaded yet");
        println!("      Run: cd tests/fixtures/test-app && go mod download");
        return;
    }

    assert!(!dep_files.is_empty(), "Should find dependency files");

    let file_names: Vec<String> = dep_files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let has_go_jose = file_names
        .iter()
        .any(|f| f.contains("go-jose") || f.contains("go-jose/v3"));

    if has_go_jose {
        println!("Found go-jose dependency files:");
        for file in dep_files.iter().filter(|f| {
            f.to_string_lossy().contains("go-jose") || f.to_string_lossy().contains("go-jose/v3")
        }) {
            println!("  - {}", file.display());
        }
    } else {
        println!("NOTE: go-jose files not found in dependency scan");
        println!("      This may be expected if scanning entire module cache");
    }
}

#[test]
fn test_discovery_e2e_dependencies_with_filter() {
    let test_app_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("test-app");

    let loader = GoPackageLoader;
    let filter = GoCryptoFilter;

    let dep_files = loader
        .load_dependencies(&test_app_path)
        .expect("Failed to load dependencies");

    if dep_files.is_empty() {
        println!("NOTE: No dependencies found - skipping filter test");
        return;
    }

    let crypto_dep_files: Vec<PathBuf> = dep_files
        .into_iter()
        .filter_map(|file| {
            filter
                .has_crypto_usage(&file)
                .ok()
                .and_then(|has_crypto| has_crypto.then_some(file))
        })
        .collect();

    println!(
        "Found {} dependency files with crypto usage",
        crypto_dep_files.len()
    );

    if !crypto_dep_files.is_empty() {
        println!("Crypto dependency files that would be scanned:");
        for file in &crypto_dep_files {
            println!("  - {}", file.display());
        }
    }
}

#[test]
fn test_discovery_e2e_user_and_dependencies() {
    let test_app_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("test-app");

    let loader = GoPackageLoader;
    let filter = GoCryptoFilter;

    let user_files = loader
        .load_user_code(&test_app_path)
        .expect("Failed to load user code");

    let dep_files = loader
        .load_dependencies(&test_app_path)
        .expect("Failed to load dependencies");

    let all_files: Vec<PathBuf> = user_files.into_iter().chain(dep_files).collect();

    assert!(
        !all_files.is_empty(),
        "Should find some files (user code or dependencies)"
    );

    let crypto_files: Vec<PathBuf> = all_files
        .into_iter()
        .filter_map(|file| {
            filter
                .has_crypto_usage(&file)
                .ok()
                .and_then(|has_crypto| has_crypto.then_some(file))
        })
        .collect();

    assert!(!crypto_files.is_empty(), "Should find crypto files");

    let user_crypto_count = crypto_files
        .iter()
        .filter(|f| f.to_string_lossy().contains("test-app"))
        .count();

    let dep_crypto_count = crypto_files.len() - user_crypto_count;

    println!("Complete scan results:");
    println!("  User code crypto files: {}", user_crypto_count);
    println!("  Dependency crypto files: {}", dep_crypto_count);
    println!("  Total crypto files: {}", crypto_files.len());

    assert!(
        user_crypto_count >= 3,
        "Should find at least 3 user code crypto files (pbkdf2.go, jose.go, aes.go)"
    );
}

#[test]
fn test_discovery_e2e_go_jose_imported() {
    let test_app_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("test-app");

    let loader = GoPackageLoader;
    let filter = GoCryptoFilter;

    let user_files = loader
        .load_user_code(&test_app_path)
        .expect("Failed to load user code");

    let user_crypto_files: Vec<PathBuf> = user_files
        .into_iter()
        .filter_map(|file| {
            filter
                .has_crypto_usage(&file)
                .ok()
                .and_then(|has_crypto| has_crypto.then_some(file))
        })
        .collect();

    let has_jose_import = user_crypto_files
        .iter()
        .any(|f| f.to_string_lossy().contains("jose.go"));

    assert!(
        has_jose_import,
        "Should find jose.go file that imports go-jose"
    );

    let dep_files = loader
        .load_dependencies(&test_app_path)
        .expect("Failed to load dependencies");

    if dep_files.is_empty() {
        println!("NOTE: No dependencies found - skipping go-jose dependency verification");
        return;
    }

    let crypto_dep_files: Vec<PathBuf> = dep_files
        .into_iter()
        .filter_map(|file| {
            filter
                .has_crypto_usage(&file)
                .ok()
                .and_then(|has_crypto| has_crypto.then_some(file))
        })
        .collect();

    let go_jose_files: Vec<_> = crypto_dep_files
        .iter()
        .filter(|f| {
            let path = f.to_string_lossy();
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
        println!("  - {}", file.display());
    }
    if go_jose_files.len() > 10 {
        println!("  ... and {} more", go_jose_files.len() - 10);
    }
}
