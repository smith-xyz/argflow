//! Python literal resolution tests

use super::test_utils::{get_first_arg_int, get_first_arg_string, scan_python};

// =============================================================================
// Integer Literals
// =============================================================================

#[test]
fn test_decimal_integer() {
    let result = scan_python(
        r#"
import hashlib
hashlib.pbkdf2_hmac('sha256', password, salt, 100000, 32)
"#,
    );
    // Python pbkdf2_hmac: arg[3] = iterations, arg[4] = dklen
    assert_eq!(get_first_arg_int(&result, 3), Some(100000), "Iterations");
    assert_eq!(get_first_arg_int(&result, 4), Some(32), "Key length");
}

#[test]
fn test_hex_integer() {
    let result = scan_python(
        r#"
import hashlib
hashlib.pbkdf2_hmac('sha256', password, salt, 0x186A0, 0x20)
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(100000),
        "Hex iterations"
    );
    assert_eq!(get_first_arg_int(&result, 4), Some(32), "Hex key length");
}

#[test]
fn test_octal_integer() {
    let result = scan_python(
        r#"
import hashlib
hashlib.pbkdf2_hmac('sha256', password, salt, 0o303240, 0o40)
"#,
    );
    // 0o303240 = 100000, 0o40 = 32
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(100000),
        "Octal iterations"
    );
    assert_eq!(get_first_arg_int(&result, 4), Some(32), "Octal key length");
}

#[test]
fn test_binary_integer() {
    let result = scan_python(
        r#"
import hashlib
hashlib.pbkdf2_hmac('sha256', password, salt, 10000, 0b100000)
"#,
    );
    // 0b100000 = 32
    assert_eq!(get_first_arg_int(&result, 4), Some(32), "Binary key length");
}

#[test]
fn test_underscore_separator() {
    let result = scan_python(
        r#"
import hashlib
hashlib.pbkdf2_hmac('sha256', password, salt, 100_000, 32)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(100000));
}

#[test]
fn test_zero() {
    let result = scan_python(
        r#"
import hashlib
hashlib.pbkdf2_hmac('sha256', password, salt, 0, 0)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(0));
    assert_eq!(get_first_arg_int(&result, 4), Some(0));
}

#[test]
fn test_large_integer() {
    let result = scan_python(
        r#"
import hashlib
hashlib.pbkdf2_hmac('sha256', password, salt, 1000000000, 64)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(1000000000));
}

// =============================================================================
// String Literals
// =============================================================================

#[test]
fn test_single_quoted_string() {
    let result = scan_python(
        r#"
import hashlib
hashlib.pbkdf2_hmac('sha256', password, salt, 10000)
"#,
    );
    assert_eq!(get_first_arg_string(&result, 0), Some("sha256".to_string()));
}

#[test]
fn test_double_quoted_string() {
    let result = scan_python(
        r#"
import hashlib
hashlib.pbkdf2_hmac("sha512", password, salt, 10000)
"#,
    );
    assert_eq!(get_first_arg_string(&result, 0), Some("sha512".to_string()));
}

#[test]
fn test_algorithm_strings() {
    for algo in ["sha256", "sha512", "sha384", "sha1", "md5"] {
        let source = format!(
            r#"
import hashlib
hashlib.pbkdf2_hmac('{algo}', password, salt, 10000)
"#
        );
        let result = scan_python(&source);
        assert_eq!(
            get_first_arg_string(&result, 0),
            Some(algo.to_string()),
            "Algorithm: {algo}"
        );
    }
}

// =============================================================================
// Boolean and None Literals
// =============================================================================

#[test]
fn test_boolean_in_context() {
    let result = scan_python(
        r#"
import hashlib
h = hashlib.sha256()
"#,
    );
    assert_eq!(result.call_count(), 1);
}

#[test]
fn test_none_literal() {
    let result = scan_python(
        r#"
import hashlib
hashlib.pbkdf2_hmac('sha256', password, salt, 10000, None)
"#,
    );
    assert_eq!(result.call_count(), 1);
}
