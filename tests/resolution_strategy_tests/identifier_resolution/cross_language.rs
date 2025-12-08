//! Cross-language identifier resolution consistency tests
//!
//! These tests verify that identifier resolution behaves consistently across
//! all supported languages for common patterns.

use super::test_utils::{
    get_arg_source, get_first_arg_int, get_first_arg_string, is_arg_resolved, scan_go,
    scan_javascript, scan_python, scan_rust,
};

// =============================================================================
// Local Variable Resolution Consistency
// =============================================================================

#[test]
fn test_local_variable_across_languages() {
    // Go
    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() {
    iterations := 100000
    pbkdf2.Key(p, s, iterations, 32, h)
}
"#,
    );
    assert_eq!(
        get_first_arg_int(&go_result, 2),
        Some(100000),
        "Go local variable"
    );

    // Python
    let py_result = scan_python(
        r#"
import hashlib

def derive():
    iterations = 100000
    hashlib.pbkdf2_hmac('sha256', p, s, iterations)
"#,
    );
    assert_eq!(
        get_first_arg_int(&py_result, 3),
        Some(100000),
        "Python local variable"
    );

    // Rust
    let rs_result = scan_rust(
        r#"
fn derive() {
    let iterations = 100000;
    pbkdf2::derive(iterations);
}
"#,
    );
    assert_eq!(
        get_first_arg_int(&rs_result, 0),
        Some(100000),
        "Rust local variable"
    );

    // JavaScript
    let js_result = scan_javascript(
        r#"
function derive() {
    const iterations = 100000;
    crypto.pbkdf2Sync(p, s, iterations, 32, 'sha256');
}
"#,
    );
    assert_eq!(
        get_first_arg_int(&js_result, 2),
        Some(100000),
        "JavaScript local variable"
    );
}

// =============================================================================
// File-Level Constant Consistency
// =============================================================================

#[test]
fn test_file_level_constant_across_languages() {
    // Go
    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
const DEFAULT_ITERATIONS = 100000
func main() {
    pbkdf2.Key(p, s, DEFAULT_ITERATIONS, 32, h)
}
"#,
    );
    assert_eq!(
        get_first_arg_int(&go_result, 2),
        Some(100000),
        "Go file-level constant"
    );

    // Python
    let py_result = scan_python(
        r#"
import hashlib

DEFAULT_ITERATIONS = 100000

def derive():
    hashlib.pbkdf2_hmac('sha256', p, s, DEFAULT_ITERATIONS)
"#,
    );
    assert_eq!(
        get_first_arg_int(&py_result, 3),
        Some(100000),
        "Python file-level constant"
    );

    // Rust
    let rs_result = scan_rust(
        r#"
const DEFAULT_ITERATIONS: u32 = 100000;

fn derive() {
    pbkdf2::derive(DEFAULT_ITERATIONS);
}
"#,
    );
    assert_eq!(
        get_first_arg_int(&rs_result, 0),
        Some(100000),
        "Rust file-level constant"
    );

    // JavaScript
    let js_result = scan_javascript(
        r#"
const DEFAULT_ITERATIONS = 100000;

function derive() {
    crypto.pbkdf2Sync(p, s, DEFAULT_ITERATIONS, 32, 'sha256');
}
"#,
    );
    assert_eq!(
        get_first_arg_int(&js_result, 2),
        Some(100000),
        "JavaScript file-level constant"
    );
}

// =============================================================================
// Function Parameter Consistency
// =============================================================================

#[test]
fn test_function_parameter_across_languages() {
    // Go
    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func DeriveKey(iterations int) {
    pbkdf2.Key(p, s, iterations, 32, h)
}
"#,
    );
    assert!(!is_arg_resolved(&go_result, 2), "Go function parameter");
    assert_eq!(
        get_arg_source(&go_result, 2),
        Some("function_parameter".to_string())
    );

    // Python
    let py_result = scan_python(
        r#"
import hashlib

def derive(iterations):
    hashlib.pbkdf2_hmac('sha256', p, s, iterations)
"#,
    );
    assert!(!is_arg_resolved(&py_result, 3), "Python function parameter");
    assert_eq!(
        get_arg_source(&py_result, 3),
        Some("function_parameter".to_string())
    );

    // Rust
    let rs_result = scan_rust(
        r#"
fn derive(iterations: u32) {
    pbkdf2::derive(iterations);
}
"#,
    );
    assert!(!is_arg_resolved(&rs_result, 0), "Rust function parameter");
    assert_eq!(
        get_arg_source(&rs_result, 0),
        Some("function_parameter".to_string())
    );

    // JavaScript
    let js_result = scan_javascript(
        r#"
function derive(iterations) {
    crypto.pbkdf2Sync(p, s, iterations, 32, 'sha256');
}
"#,
    );
    assert!(
        !is_arg_resolved(&js_result, 2),
        "JavaScript function parameter"
    );
    assert_eq!(
        get_arg_source(&js_result, 2),
        Some("function_parameter".to_string())
    );
}

// =============================================================================
// String Variable Consistency
// =============================================================================

#[test]
fn test_string_variable_across_languages() {
    // Go
    let go_result = scan_go(
        r#"
package main
import "crypto/sha256"
func main() {
    algorithm := "sha256"
    sha256.New(algorithm)
}
"#,
    );
    assert_eq!(
        get_first_arg_string(&go_result, 0),
        Some("sha256".to_string()),
        "Go string variable"
    );

    // Python
    let py_result = scan_python(
        r#"
import hashlib

def hash():
    algorithm = "sha256"
    hashlib.new(algorithm)
"#,
    );
    assert_eq!(
        get_first_arg_string(&py_result, 0),
        Some("sha256".to_string()),
        "Python string variable"
    );

    // Rust
    let rs_result = scan_rust(
        r#"
fn hash() {
    let algorithm = "sha256";
    hash::new(algorithm);
}
"#,
    );
    assert_eq!(
        get_first_arg_string(&rs_result, 0),
        Some("sha256".to_string()),
        "Rust string variable"
    );

    // JavaScript
    let js_result = scan_javascript(
        r#"
function hash() {
    const algorithm = "sha256";
    crypto.createHash(algorithm);
}
"#,
    );
    assert_eq!(
        get_first_arg_string(&js_result, 0),
        Some("sha256".to_string()),
        "JavaScript string variable"
    );
}
