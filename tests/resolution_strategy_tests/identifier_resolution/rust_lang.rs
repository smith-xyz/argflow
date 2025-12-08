//! Rust identifier resolution tests

use super::test_utils::{
    get_arg_source, get_first_arg_int, get_first_arg_string, is_arg_resolved, scan_rust,
};

// =============================================================================
// Local Variable Resolution
// =============================================================================

#[test]
fn test_local_let_variable() {
    let result = scan_rust(
        r#"
fn encrypt() {
    let key_size = 32;
    aes::new_cipher(key_size);
}
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 0),
        Some(32),
        "Local let variable resolution"
    );
}

#[test]
fn test_multiple_variables() {
    let result = scan_rust(
        r#"
fn derive_key() {
    let iterations = 100000;
    let key_len = 32;
    pbkdf2::derive(iterations, key_len);
}
"#,
    );
    assert_eq!(get_first_arg_int(&result, 0), Some(100000), "iterations");
    assert_eq!(get_first_arg_int(&result, 1), Some(32), "key_len");
}

// =============================================================================
// File-Level Constants
// =============================================================================

#[test]
fn test_file_level_const() {
    let result = scan_rust(
        r#"
const DEFAULT_ITERATIONS: u32 = 100000;

fn derive_key() {
    pbkdf2::derive(DEFAULT_ITERATIONS);
}
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 0),
        Some(100000),
        "File-level const"
    );
}

#[test]
fn test_static_variable() {
    let result = scan_rust(
        r#"
static KEY_SIZE: u32 = 256;

fn encrypt() {
    aes::new_cipher(KEY_SIZE);
}
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 0),
        Some(256),
        "Static variable resolution"
    );
}

// =============================================================================
// Function Parameters (Unresolved)
// =============================================================================

#[test]
fn test_function_parameter_unresolved() {
    let result = scan_rust(
        r#"
fn derive_key(iterations: u32) {
    pbkdf2::derive(iterations);
}
"#,
    );
    assert!(
        !is_arg_resolved(&result, 0),
        "Function parameter should not be resolved"
    );
    assert_eq!(
        get_arg_source(&result, 0),
        Some("function_parameter".to_string()),
        "Should be marked as function_parameter"
    );
}

// =============================================================================
// String Variables
// =============================================================================

#[test]
fn test_string_variable() {
    let result = scan_rust(
        r#"
fn hash_data() {
    let algorithm = "sha256";
    hash::new(algorithm);
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
    let result = scan_rust(
        r#"
const ALGORITHM: &str = "sha256";

fn hash_data() {
    hash::new(ALGORITHM);
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
    let result = scan_rust(
        r#"
fn derive_key() {
    pbkdf2::derive(undeclared_var);
}
"#,
    );
    assert!(
        !is_arg_resolved(&result, 0),
        "Undeclared variable should not be resolved"
    );
    assert_eq!(
        get_arg_source(&result, 0),
        Some("identifier_not_found".to_string()),
        "Should be marked as identifier_not_found"
    );
}
