//! Python identifier resolution tests

use super::test_utils::{
    get_arg_source, get_first_arg_int, get_first_arg_string, is_arg_resolved, scan_python,
};

// =============================================================================
// Local Variable Resolution
// =============================================================================

#[test]
fn test_local_variable() {
    let result = scan_python(
        r#"
import hashlib

def derive_key():
    iterations = 100000
    hashlib.pbkdf2_hmac('sha256', password, salt, iterations)
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(100000),
        "Local variable resolution"
    );
}

#[test]
fn test_multiple_variables() {
    let result = scan_python(
        r#"
import hashlib

def derive_key():
    iterations = 100000
    dk_len = 32
    hashlib.pbkdf2_hmac('sha256', password, salt, iterations, dk_len)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(100000), "iterations");
    assert_eq!(get_first_arg_int(&result, 4), Some(32), "dk_len");
}

// =============================================================================
// File-Level Constants
// =============================================================================

#[test]
fn test_file_level_constant() {
    let result = scan_python(
        r#"
import hashlib

DEFAULT_ITERATIONS = 100000

def derive_key():
    hashlib.pbkdf2_hmac('sha256', password, salt, DEFAULT_ITERATIONS)
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(100000),
        "File-level constant"
    );
}

// =============================================================================
// Function Parameters (Unresolved)
// =============================================================================

#[test]
fn test_function_parameter_unresolved() {
    let result = scan_python(
        r#"
import hashlib

def derive_key(iterations):
    hashlib.pbkdf2_hmac('sha256', password, salt, iterations)
"#,
    );
    assert!(
        !is_arg_resolved(&result, 3),
        "Function parameter should not be resolved"
    );
    assert_eq!(
        get_arg_source(&result, 3),
        Some("function_parameter".to_string()),
        "Should be marked as function_parameter"
    );
}

// =============================================================================
// Variable Shadowing
// =============================================================================

#[test]
fn test_local_shadows_global() {
    let result = scan_python(
        r#"
import hashlib

ITERATIONS = 10000

def derive_key():
    ITERATIONS = 100000
    hashlib.pbkdf2_hmac('sha256', password, salt, ITERATIONS)
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(100000),
        "Local should shadow global"
    );
}

// =============================================================================
// String Variables
// =============================================================================

#[test]
fn test_string_variable() {
    let result = scan_python(
        r#"
import hashlib

def hash_data():
    algorithm = "sha256"
    hashlib.new(algorithm)
"#,
    );
    assert_eq!(
        get_first_arg_string(&result, 0),
        Some("sha256".to_string()),
        "String variable resolution"
    );
}

#[test]
fn test_string_constant() {
    let result = scan_python(
        r#"
import hashlib

ALGORITHM = "sha256"

def hash_data():
    hashlib.new(ALGORITHM)
"#,
    );
    assert_eq!(
        get_first_arg_string(&result, 0),
        Some("sha256".to_string()),
        "String constant resolution"
    );
}

// =============================================================================
// Identifier Not Found
// =============================================================================

#[test]
fn test_identifier_not_found() {
    let result = scan_python(
        r#"
import hashlib

def derive_key():
    hashlib.pbkdf2_hmac('sha256', password, salt, undeclared_var)
"#,
    );
    assert!(
        !is_arg_resolved(&result, 3),
        "Undeclared variable should not be resolved"
    );
    assert_eq!(
        get_arg_source(&result, 3),
        Some("identifier_not_found".to_string()),
        "Should be marked as identifier_not_found"
    );
}
