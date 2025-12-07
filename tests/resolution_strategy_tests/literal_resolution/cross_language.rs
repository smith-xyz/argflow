//! Cross-language consistency tests
//!
//! Verifies that the same literal values produce consistent results
//! across all supported languages.

use super::test_utils::{scan_go, scan_javascript, scan_python, scan_rust};

#[test]
fn test_pbkdf2_values_consistent() {
    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 100000, 32, h) }
"#,
    );

    let py_result = scan_python(
        r#"
import hashlib
hashlib.pbkdf2_hmac('sha256', password, salt, 100000, 32)
"#,
    );

    let js_result = scan_javascript(
        r#"
const crypto = require('crypto');
const key = crypto.pbkdf2Sync(password, salt, 100000, 32, 'sha256');
"#,
    );

    let rs_result = scan_rust(
        r#"
fn pbkdf2(password: &[u8], salt: &[u8], iterations: u32, key: &mut [u8]) {}
fn main() {
    pbkdf2(password, salt, 100000, &mut key);
}
"#,
    );

    assert_eq!(go_result.call_count(), 1, "Go call");
    assert_eq!(py_result.call_count(), 1, "Python call");
    assert_eq!(js_result.call_count(), 1, "JS call");
    assert_eq!(rs_result.call_count(), 1, "Rust call");

    let go_iter = go_result.calls[0].arguments[2].int_values.first().copied();
    let py_iter = py_result.calls[0].arguments[3].int_values.first().copied();
    let js_iter = js_result.calls[0].arguments[2].int_values.first().copied();
    let rs_iter = rs_result.calls[0].arguments[2].int_values.first().copied();

    assert_eq!(go_iter, Some(100000), "Go iterations");
    assert_eq!(py_iter, Some(100000), "Python iterations");
    assert_eq!(js_iter, Some(100000), "JS iterations");
    assert_eq!(rs_iter, Some(100000), "Rust iterations");
}

#[test]
fn test_hex_literals_consistent() {
    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 0xFF, 32, h) }
"#,
    );

    let py_result = scan_python(
        r#"
import hashlib
hashlib.pbkdf2_hmac('sha256', password, salt, 0xFF, 32)
"#,
    );

    let js_result = scan_javascript(
        r#"
const crypto = require('crypto');
const key = crypto.pbkdf2Sync(password, salt, 0xFF, 32, 'sha256');
"#,
    );

    let go_val = go_result.calls[0].arguments[2].int_values.first().copied();
    let py_val = py_result.calls[0].arguments[3].int_values.first().copied();
    let js_val = js_result.calls[0].arguments[2].int_values.first().copied();

    assert_eq!(go_val, Some(255), "Go 0xFF");
    assert_eq!(py_val, Some(255), "Python 0xFF");
    assert_eq!(js_val, Some(255), "JS 0xFF");
}

#[test]
fn test_algorithm_strings_detected() {
    let py_result = scan_python(
        r#"
import hashlib
hashlib.pbkdf2_hmac('sha256', password, salt, 10000)
"#,
    );

    let js_result = scan_javascript(
        r#"
const crypto = require('crypto');
const key = crypto.pbkdf2Sync(password, salt, 10000, 32, 'sha256');
"#,
    );

    let py_algo = py_result.calls[0].arguments[0]
        .string_values
        .first()
        .cloned();
    let js_algo = js_result.calls[0].arguments[4]
        .string_values
        .first()
        .cloned();

    assert_eq!(py_algo, Some("sha256".to_string()), "Python algorithm");
    assert_eq!(js_algo, Some("sha256".to_string()), "JS algorithm");
}

#[test]
fn test_zero_consistent() {
    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 0, 0, h) }
"#,
    );

    let py_result = scan_python(
        r#"
import hashlib
hashlib.pbkdf2_hmac('sha256', password, salt, 0, 0)
"#,
    );

    let go_zero = go_result.calls[0].arguments[2].int_values.first().copied();
    let py_zero = py_result.calls[0].arguments[3].int_values.first().copied();

    assert_eq!(go_zero, Some(0), "Go zero");
    assert_eq!(py_zero, Some(0), "Python zero");
}
