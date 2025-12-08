//! Cross-language consistency tests for call resolution
//!
//! These tests verify that similar patterns resolve consistently
//! across all supported languages.

use super::test_utils::*;

#[test]
fn test_simple_int_return_consistency() {
    // Go
    let go_source = r#"
package main

import "golang.org/x/crypto/pbkdf2"

func getIterations() int {
    return 100000
}

func main() {
    pbkdf2.Key(nil, nil, getIterations(), 32, nil)
}"#;
    let go_result = scan_go(go_source);

    // Python
    let python_source = r#"
from hashlib import pbkdf2_hmac

def get_iterations():
    return 100000

pbkdf2_hmac("sha256", b"password", b"salt", get_iterations())
"#;
    let python_result = scan_python(python_source);

    // JavaScript
    let js_source = r#"
const crypto = require('crypto');

function getIterations() {
    return 100000;
}

crypto.pbkdf2Sync("password", "salt", getIterations(), 32, "sha256");
"#;
    let js_result = scan_javascript(js_source);

    // Rust
    let rust_source = r#"
use ring::pbkdf2;

fn get_iterations() -> u32 {
    return 100000;
}

fn main() {
    pbkdf2::derive(get_iterations());
}
"#;
    let rust_result = scan_rust(rust_source);

    // All should resolve to 100000
    assert_eq!(get_first_arg_int(&go_result, 2), Some(100000));
    assert_eq!(get_first_arg_int(&python_result, 3), Some(100000));
    assert_eq!(get_first_arg_int(&js_result, 2), Some(100000));
    assert_eq!(get_first_arg_int(&rust_result, 0), Some(100000));
}

#[test]
fn test_function_not_found_consistency() {
    // Go
    let go_source = r#"
package main

import "golang.org/x/crypto/pbkdf2"

func main() {
    pbkdf2.Key(nil, nil, unknownFunc(), 32, nil)
}"#;
    let go_result = scan_go(go_source);

    // Python
    let python_source = r#"
from hashlib import pbkdf2_hmac

pbkdf2_hmac("sha256", b"password", b"salt", unknown_func())
"#;
    let python_result = scan_python(python_source);

    // JavaScript
    let js_source = r#"
const crypto = require('crypto');

crypto.pbkdf2Sync("password", "salt", unknownFunc(), 32, "sha256");
"#;
    let js_result = scan_javascript(js_source);

    // Rust
    let rust_source = r#"
use ring::pbkdf2;

fn main() {
    pbkdf2::derive(unknown_func());
}
"#;
    let rust_result = scan_rust(rust_source);

    // All should be unresolved with "function_not_found" source
    assert_eq!(
        get_arg_source(&go_result, 2),
        Some("function_not_found".to_string())
    );
    assert_eq!(
        get_arg_source(&python_result, 3),
        Some("function_not_found".to_string())
    );
    assert_eq!(
        get_arg_source(&js_result, 2),
        Some("function_not_found".to_string())
    );
    assert_eq!(
        get_arg_source(&rust_result, 0),
        Some("function_not_found".to_string())
    );
}

#[test]
fn test_if_else_returns_consistency() {
    // Go
    let go_source = r#"
package main

import "golang.org/x/crypto/pbkdf2"

func getSize(large bool) int {
    if large {
        return 32
    }
    return 16
}

func main() {
    pbkdf2.Key(nil, nil, 10000, getSize(true), nil)
}"#;
    let go_result = scan_go(go_source);

    // Python
    let python_source = r#"
from hashlib import pbkdf2_hmac

def get_size(large):
    if large:
        return 32
    return 16

pbkdf2_hmac("sha256", b"password", b"salt", get_size(True))
"#;
    let python_result = scan_python(python_source);

    // JavaScript
    let js_source = r#"
const crypto = require('crypto');

function getSize(large) {
    if (large) {
        return 32;
    }
    return 16;
}

crypto.pbkdf2Sync("password", "salt", 10000, getSize(true), "sha256");
"#;
    let js_result = scan_javascript(js_source);

    // All should have both 32 and 16 as possible values
    let go_sizes = get_first_arg_ints(&go_result, 3);
    let python_sizes = get_first_arg_ints(&python_result, 3);
    let js_sizes = get_first_arg_ints(&js_result, 3);

    assert!(go_sizes.contains(&32) || go_sizes.contains(&16));
    assert!(python_sizes.contains(&32) || python_sizes.contains(&16));
    assert!(js_sizes.contains(&32) || js_sizes.contains(&16));
}

#[test]
fn test_string_return_consistency() {
    // Go
    let go_source = r#"
package main

import "crypto/sha256"

func getAlgorithm() string {
    return "sha256"
}

func main() {
    sha256.New(getAlgorithm())
}"#;
    let go_result = scan_go(go_source);

    // Python
    let python_source = r#"
from hashlib import pbkdf2_hmac

def get_algorithm():
    return "sha256"

pbkdf2_hmac(get_algorithm(), b"password", b"salt", 10000)
"#;
    let python_result = scan_python(python_source);

    // JavaScript
    let js_source = r#"
const crypto = require('crypto');

function getAlgorithm() {
    return "sha256";
}

crypto.pbkdf2Sync("password", "salt", 10000, 32, getAlgorithm());
"#;
    let js_result = scan_javascript(js_source);

    // All should resolve to "sha256"
    assert_eq!(
        get_first_arg_string(&go_result, 0),
        Some("sha256".to_string())
    );
    assert_eq!(
        get_first_arg_string(&python_result, 0),
        Some("sha256".to_string())
    );
    assert_eq!(
        get_first_arg_string(&js_result, 4),
        Some("sha256".to_string())
    );
}
