//! Go-specific scanner e2e tests
//!
//! Tests crypto detection and parameter resolution for Go code.
//! Fixtures: tests/fixtures/go/

use crypto_extractor_core::scanner::Scanner;
use std::path::PathBuf;

fn go_fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("go")
}

fn parse_go(source: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_go::LANGUAGE.into())
        .unwrap();
    parser.parse(source, None).unwrap()
}

fn scan_go_file(project: &str, file_path: &str) -> crypto_extractor_core::scanner::ScanResult {
    let full_path = go_fixtures_path().join(project).join(file_path);
    let source = std::fs::read_to_string(&full_path)
        .unwrap_or_else(|_| panic!("Failed to read: {}/{}", project, file_path));
    let tree = parse_go(&source);
    let scanner = Scanner::new();
    scanner.scan_tree(&tree, source.as_bytes(), &full_path.to_string_lossy(), "go")
}

fn scan_go_inline(source: &str) -> crypto_extractor_core::scanner::ScanResult {
    let tree = parse_go(source);
    let scanner = Scanner::new();
    scanner.scan_tree(&tree, source.as_bytes(), "inline.go", "go")
}

// =============================================================================
// basic-crypto project tests
// =============================================================================

#[test]
fn test_go_basic_crypto_pbkdf2() {
    let result = scan_go_file("basic-crypto", "pkg/kdf/pbkdf2.go");

    // Go: pbkdf2.Key(password, salt, iterations, keyLen, hashFunc)
    let pbkdf2_calls: Vec<_> = result
        .calls
        .iter()
        .filter(|c| c.function_name == "Key" && c.package.as_deref() == Some("pbkdf2"))
        .collect();

    assert_eq!(pbkdf2_calls.len(), 3, "Should find 3 pbkdf2.Key calls");

    // Find the literal call (DeriveKeyLiteral) - has resolved iterations
    let literal_call = pbkdf2_calls
        .iter()
        .find(|c| c.arguments.get(2).map(|a| a.is_resolved).unwrap_or(false))
        .expect("Should find a call with resolved iterations");

    // Go pbkdf2.Key: arg[2] = iterations, arg[3] = keyLen
    assert_eq!(literal_call.arguments[2].int_values, vec![10000]);
    assert_eq!(literal_call.arguments[3].int_values, vec![32]);
}

#[test]
fn test_go_basic_crypto_aes() {
    let result = scan_go_file("basic-crypto", "pkg/cipher/aes.go");

    // Go: aes.NewCipher(key)
    let aes_calls: Vec<_> = result
        .calls
        .iter()
        .filter(|c| c.function_name == "NewCipher" && c.package.as_deref() == Some("aes"))
        .collect();

    assert_eq!(aes_calls.len(), 2, "Should find 2 aes.NewCipher calls");

    // Go: cipher.NewGCM(block)
    let gcm_calls: Vec<_> = result
        .calls
        .iter()
        .filter(|c| c.function_name == "NewGCM")
        .collect();

    assert_eq!(gcm_calls.len(), 1, "Should find 1 cipher.NewGCM call");
}

#[test]
fn test_go_basic_crypto_hash() {
    let result = scan_go_file("basic-crypto", "pkg/hash/sha.go");

    // Go: sha256.New(), sha512.New()
    let sha256_calls: Vec<_> = result
        .calls
        .iter()
        .filter(|c| c.package.as_deref() == Some("sha256"))
        .collect();

    let sha512_calls: Vec<_> = result
        .calls
        .iter()
        .filter(|c| c.package.as_deref() == Some("sha512"))
        .collect();

    assert!(sha256_calls.len() >= 2, "Should find sha256 calls");
    assert!(sha512_calls.len() >= 2, "Should find sha512 calls");
}

#[test]
fn test_go_basic_crypto_no_false_positives() {
    let result = scan_go_file("basic-crypto", "pkg/utils/helpers.go");
    assert_eq!(
        result.call_count(),
        0,
        "Should find NO crypto calls in helpers.go"
    );
}

// =============================================================================
// cross-file-constants project tests
// =============================================================================

#[test]
fn test_go_cross_file_constants_config() {
    let result = scan_go_file("cross-file-constants", "config/constants.go");
    assert_eq!(result.call_count(), 0, "Constants file has no crypto calls");
}

#[test]
fn test_go_cross_file_constants_kdf() {
    let result = scan_go_file("cross-file-constants", "crypto/kdf.go");

    let pbkdf2_calls: Vec<_> = result
        .calls
        .iter()
        .filter(|c| c.function_name == "Key")
        .collect();

    assert_eq!(pbkdf2_calls.len(), 3, "Should find 3 pbkdf2.Key calls");
    // Note: Cross-file constant resolution requires Identifier strategy
}

// =============================================================================
// Inline tests for Go-specific resolution behaviors
// =============================================================================

#[test]
fn test_go_inline_literal_integers() {
    let result = scan_go_inline(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() {
    key := pbkdf2.Key(password, salt, 100000, 32, sha256.New)
}
"#,
    );

    assert_eq!(result.call_count(), 1);
    let call = &result.calls[0];

    // Go pbkdf2.Key: arg[2] = iterations, arg[3] = keyLen
    assert!(call.arguments[2].is_resolved);
    assert_eq!(call.arguments[2].int_values, vec![100000]);
    assert!(call.arguments[3].is_resolved);
    assert_eq!(call.arguments[3].int_values, vec![32]);
}

#[test]
fn test_go_inline_hex_literals() {
    let result = scan_go_inline(
        r#"
package main
import "crypto/aes"
func main() {
    key := make([]byte, 0x20)
    block, _ := aes.NewCipher(key)
    _ = block
}
"#,
    );

    let aes_calls: Vec<_> = result
        .calls
        .iter()
        .filter(|c| c.function_name == "NewCipher")
        .collect();

    assert_eq!(aes_calls.len(), 1, "Should find aes.NewCipher");
}

#[test]
fn test_go_inline_multiple_crypto_calls() {
    let result = scan_go_inline(
        r#"
package main
import (
    "crypto/aes"
    "crypto/sha256"
    "golang.org/x/crypto/pbkdf2"
)
func main() {
    h := sha256.New()
    key := pbkdf2.Key(pass, salt, 10000, 32, h)
    block, _ := aes.NewCipher(key)
    _ = block
}
"#,
    );

    assert_eq!(result.call_count(), 3, "Should find 3 crypto calls");

    let names: Vec<_> = result
        .calls
        .iter()
        .map(|c| c.function_name.as_str())
        .collect();

    assert!(names.contains(&"New"));
    assert!(names.contains(&"Key"));
    assert!(names.contains(&"NewCipher"));
}
