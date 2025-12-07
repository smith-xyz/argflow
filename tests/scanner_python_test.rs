//! Python-specific scanner e2e tests
//!
//! Tests crypto detection and parameter resolution for Python code.
//! Fixtures: tests/fixtures/python/

use crypto_extractor_core::scanner::Scanner;
use std::path::PathBuf;

fn python_fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("python")
}

fn parse_python(source: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .unwrap();
    parser.parse(source, None).unwrap()
}

fn scan_python_file(project: &str, file_path: &str) -> crypto_extractor_core::scanner::ScanResult {
    let full_path = python_fixtures_path().join(project).join(file_path);
    let source = std::fs::read_to_string(&full_path)
        .unwrap_or_else(|_| panic!("Failed to read: {}/{}", project, file_path));
    let tree = parse_python(&source);
    let scanner = Scanner::new();
    scanner.scan_tree(
        &tree,
        source.as_bytes(),
        &full_path.to_string_lossy(),
        "python",
    )
}

fn scan_python_inline(source: &str) -> crypto_extractor_core::scanner::ScanResult {
    let tree = parse_python(source);
    let scanner = Scanner::new();
    scanner.scan_tree(&tree, source.as_bytes(), "inline.py", "python")
}

// =============================================================================
// basic-crypto project tests
// =============================================================================

#[test]
fn test_python_basic_crypto_pbkdf2() {
    let result = scan_python_file("basic-crypto", "kdf/pbkdf2.py");

    // Python: hashlib.pbkdf2_hmac(hash_name, password, salt, iterations, dklen)
    let pbkdf2_calls: Vec<_> = result
        .calls
        .iter()
        .filter(|c| c.function_name == "pbkdf2_hmac")
        .collect();

    assert_eq!(pbkdf2_calls.len(), 3, "Should find 3 pbkdf2_hmac calls");

    // Find the literal call (derive_key_literal) - has resolved iterations
    let literal_call = pbkdf2_calls
        .iter()
        .find(|c| c.arguments.get(3).map(|a| a.is_resolved).unwrap_or(false))
        .expect("Should find a call with resolved iterations");

    // Python pbkdf2_hmac: arg[3] = iterations, arg[4] = dklen
    assert_eq!(literal_call.arguments[3].int_values, vec![10000]);
    assert_eq!(literal_call.arguments[4].int_values, vec![32]);
}

#[test]
fn test_python_basic_crypto_aes() {
    let result = scan_python_file("basic-crypto", "cipher/aes.py");

    // Python cryptography: Cipher(algorithms.AES(key), modes.CBC(iv))
    let cipher_calls: Vec<_> = result
        .calls
        .iter()
        .filter(|c| c.function_name == "Cipher")
        .collect();

    assert_eq!(cipher_calls.len(), 3, "Should find 3 Cipher calls");

    // Should also find AES() calls
    let aes_calls: Vec<_> = result
        .calls
        .iter()
        .filter(|c| c.function_name == "AES")
        .collect();

    assert_eq!(aes_calls.len(), 3, "Should find 3 AES calls");
}

#[test]
fn test_python_basic_crypto_hash() {
    let result = scan_python_file("basic-crypto", "hash/sha.py");

    // Python: hashlib.sha256(), hashlib.sha512()
    let sha256_calls: Vec<_> = result
        .calls
        .iter()
        .filter(|c| c.function_name == "sha256")
        .collect();

    let sha512_calls: Vec<_> = result
        .calls
        .iter()
        .filter(|c| c.function_name == "sha512")
        .collect();

    assert!(sha256_calls.len() >= 2, "Should find sha256 calls");
    assert!(sha512_calls.len() >= 2, "Should find sha512 calls");
}

#[test]
fn test_python_basic_crypto_no_false_positives() {
    let result = scan_python_file("basic-crypto", "utils/helpers.py");
    assert_eq!(
        result.call_count(),
        0,
        "Should find NO crypto calls in helpers.py"
    );
}

// =============================================================================
// Inline tests for Python-specific resolution behaviors
// =============================================================================

#[test]
fn test_python_inline_literal_integers() {
    let result = scan_python_inline(
        r#"
import hashlib

def derive():
    return hashlib.pbkdf2_hmac('sha256', password, salt, 100000, 32)
"#,
    );

    assert_eq!(result.call_count(), 1);
    let call = &result.calls[0];

    // Python pbkdf2_hmac: arg[3] = iterations, arg[4] = dklen
    assert!(call.arguments[3].is_resolved);
    assert_eq!(call.arguments[3].int_values, vec![100000]);
    assert!(call.arguments[4].is_resolved);
    assert_eq!(call.arguments[4].int_values, vec![32]);
}

#[test]
fn test_python_inline_string_algorithm() {
    let result = scan_python_inline(
        r#"
import hashlib

def hash_data():
    return hashlib.pbkdf2_hmac('sha256', b'password', b'salt', 10000)
"#,
    );

    assert_eq!(result.call_count(), 1);
    let call = &result.calls[0];

    // First arg is the hash algorithm string
    assert!(call.arguments[0].is_resolved);
    assert_eq!(call.arguments[0].string_values, vec!["sha256"]);
}

#[test]
fn test_python_inline_multiple_crypto_calls() {
    let result = scan_python_inline(
        r#"
import hashlib
from cryptography.hazmat.primitives.ciphers import Cipher, algorithms

def crypto_ops():
    h = hashlib.sha256()
    key = hashlib.pbkdf2_hmac('sha256', b'pass', b'salt', 10000, 32)
    cipher = Cipher(algorithms.AES(key), None)
"#,
    );

    assert!(
        result.call_count() >= 3,
        "Should find multiple crypto calls"
    );

    let names: Vec<_> = result
        .calls
        .iter()
        .map(|c| c.function_name.as_str())
        .collect();

    assert!(names.contains(&"sha256"));
    assert!(names.contains(&"pbkdf2_hmac"));
}
