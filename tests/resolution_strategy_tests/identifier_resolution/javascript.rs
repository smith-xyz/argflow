//! JavaScript identifier resolution tests

use super::test_utils::{
    get_arg_source, get_first_arg_int, get_first_arg_string, is_arg_resolved, scan_javascript,
};

// =============================================================================
// Local Variable Resolution
// =============================================================================

#[test]
fn test_const_variable() {
    let result = scan_javascript(
        r#"
function deriveKey() {
    const iterations = 100000;
    crypto.pbkdf2Sync(password, salt, iterations, 32, 'sha256');
}
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(100000),
        "Const variable resolution"
    );
}

#[test]
fn test_let_variable() {
    let result = scan_javascript(
        r#"
function encrypt() {
    let keySize = 256;
    crypto.createCipher('aes', keySize);
}
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 1),
        Some(256),
        "Let variable resolution"
    );
}

#[test]
fn test_multiple_variables() {
    let result = scan_javascript(
        r#"
function deriveKey() {
    const iterations = 100000;
    const keyLen = 32;
    crypto.pbkdf2Sync(password, salt, iterations, keyLen, 'sha256');
}
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(100000), "iterations");
    assert_eq!(get_first_arg_int(&result, 3), Some(32), "keyLen");
}

// =============================================================================
// File-Level Constants
// =============================================================================

#[test]
fn test_file_level_const() {
    let result = scan_javascript(
        r#"
const DEFAULT_ITERATIONS = 100000;

function deriveKey() {
    crypto.pbkdf2Sync(password, salt, DEFAULT_ITERATIONS, 32, 'sha256');
}
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(100000),
        "File-level const"
    );
}

// =============================================================================
// Function Parameters (Unresolved)
// =============================================================================

#[test]
fn test_function_parameter_unresolved() {
    let result = scan_javascript(
        r#"
function deriveKey(iterations) {
    crypto.pbkdf2Sync(password, salt, iterations, 32, 'sha256');
}
"#,
    );
    assert!(
        !is_arg_resolved(&result, 2),
        "Function parameter should not be resolved"
    );
    assert_eq!(
        get_arg_source(&result, 2),
        Some("function_parameter".to_string()),
        "Should be marked as function_parameter"
    );
}

// =============================================================================
// Variable Shadowing
// =============================================================================

#[test]
fn test_local_shadows_global() {
    let result = scan_javascript(
        r#"
const ITERATIONS = 10000;

function deriveKey() {
    const ITERATIONS = 100000;
    crypto.pbkdf2Sync(password, salt, ITERATIONS, 32, 'sha256');
}
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(100000),
        "Local should shadow global"
    );
}

// =============================================================================
// String Variables
// =============================================================================

#[test]
fn test_string_variable() {
    let result = scan_javascript(
        r#"
function hash() {
    const algorithm = "sha256";
    crypto.createHash(algorithm);
}
"#,
    );
    assert_eq!(
        get_first_arg_string(&result, 0),
        Some("sha256".to_string()),
        "String variable resolution"
    );
}

#[test]
fn test_string_const() {
    let result = scan_javascript(
        r#"
const ALGORITHM = "sha256";

function hash() {
    crypto.createHash(ALGORITHM);
}
"#,
    );
    assert_eq!(
        get_first_arg_string(&result, 0),
        Some("sha256".to_string()),
        "String const resolution"
    );
}

// =============================================================================
// Identifier Not Found
// =============================================================================

#[test]
fn test_identifier_not_found() {
    let result = scan_javascript(
        r#"
function deriveKey() {
    crypto.pbkdf2Sync(password, salt, undeclaredVar, 32, 'sha256');
}
"#,
    );
    assert!(
        !is_arg_resolved(&result, 2),
        "Undeclared variable should not be resolved"
    );
    assert_eq!(
        get_arg_source(&result, 2),
        Some("identifier_not_found".to_string()),
        "Should be marked as identifier_not_found"
    );
}
