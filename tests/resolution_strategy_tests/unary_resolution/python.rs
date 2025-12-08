//! Python unary resolution tests

use super::test_utils::{get_first_arg_int, is_arg_unresolved, scan_python};

// =============================================================================
// Negation Operator
// =============================================================================

#[test]
fn test_negative_integer() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm, length, salt, -1)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(-1), "Negative one");
}

#[test]
fn test_negative_large_integer() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm, length, salt, -100000)
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(-100000),
        "Negative large"
    );
}

#[test]
fn test_double_negative() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm, length, salt, --42)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(42), "Double negative");
}

#[test]
fn test_negative_parenthesized() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm, length, salt, -(-100))
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(100),
        "Negative parenthesized"
    );
}

// =============================================================================
// Positive Operator
// =============================================================================

#[test]
fn test_positive_integer() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm, length, salt, +100000)
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(100000),
        "Positive operator"
    );
}

// =============================================================================
// Bitwise NOT Operator
// =============================================================================

#[test]
fn test_bitwise_not_zero() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm, length, salt, ~0)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(-1), "~0 = -1");
}

#[test]
fn test_bitwise_not_255() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm, length, salt, ~255)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(-256), "~255 = -256");
}

// =============================================================================
// Logical NOT Operator
// =============================================================================

#[test]
fn test_not_true() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm, length, salt, not True)
"#,
    );
    // not True = False = 0
    assert_eq!(get_first_arg_int(&result, 3), Some(0), "not True = 0");
}

#[test]
fn test_not_false() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm, length, salt, not False)
"#,
    );
    // not False = True = 1
    assert_eq!(get_first_arg_int(&result, 3), Some(1), "not False = 1");
}

#[test]
fn test_double_not() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm, length, salt, not not True)
"#,
    );
    // not not True = not False = True = 1
    assert_eq!(get_first_arg_int(&result, 3), Some(1), "not not True = 1");
}

// =============================================================================
// Unresolved Operands
// =============================================================================

#[test]
fn test_negative_variable() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
iterations = 100000
kdf = PBKDF2HMAC(algorithm, length, salt, -iterations)
"#,
    );
    assert!(
        is_arg_unresolved(&result, 3),
        "-variable needs identifier resolution"
    );
}

#[test]
fn test_not_variable() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
flag = True
kdf = PBKDF2HMAC(algorithm, length, salt, not flag)
"#,
    );
    assert!(
        is_arg_unresolved(&result, 3),
        "not variable needs identifier resolution"
    );
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_negative_zero() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm, length, salt, -0)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(0), "-0 = 0");
}

#[test]
fn test_negative_hex() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm, length, salt, -0xFF)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(-255), "-0xFF = -255");
}
