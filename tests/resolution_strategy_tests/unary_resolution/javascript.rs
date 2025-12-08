//! JavaScript unary resolution tests

use super::test_utils::{get_first_arg_int, is_arg_unresolved, scan_javascript};

// =============================================================================
// Negation Operator
// =============================================================================

#[test]
fn test_negative_integer() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, -1, 32, 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(-1), "Negative one");
}

#[test]
fn test_negative_large_integer() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, -100000, 32, 'sha256');
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(-100000),
        "Negative large integer"
    );
}

#[test]
fn test_double_negative() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, --42, 32, 'sha256');
"#,
    );
    // In JS, --42 is a decrement operation which has different parsing
    // The test verifies the scanner handles this case
    assert_eq!(result.call_count(), 1);
}

#[test]
fn test_negative_parenthesized() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, -(-100), 32, 'sha256');
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(100),
        "Negative parenthesized"
    );
}

// =============================================================================
// Positive Operator
// =============================================================================

#[test]
fn test_positive_integer() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, +100000, 32, 'sha256');
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(100000),
        "Positive operator"
    );
}

// =============================================================================
// Bitwise NOT Operator
// =============================================================================

#[test]
fn test_bitwise_not_zero() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, ~0, 32, 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(-1), "~0 = -1");
}

#[test]
fn test_bitwise_not_255() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, ~255, 32, 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(-256), "~255 = -256");
}

// =============================================================================
// Logical NOT Operator
// =============================================================================

#[test]
fn test_not_true() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, 100000, 32, !true);
"#,
    );
    // !true = false = 0
    assert_eq!(get_first_arg_int(&result, 4), Some(0), "!true = 0");
}

#[test]
fn test_not_false() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, 100000, 32, !false);
"#,
    );
    // !false = true = 1
    assert_eq!(get_first_arg_int(&result, 4), Some(1), "!false = 1");
}

#[test]
fn test_double_not() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, 100000, 32, !!true);
"#,
    );
    assert_eq!(get_first_arg_int(&result, 4), Some(1), "!!true = 1");
}

// =============================================================================
// Unresolved Operands
// =============================================================================

#[test]
fn test_negative_variable() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const iterations = 100000;
crypto.pbkdf2Sync(password, salt, -iterations, 32, 'sha256');
"#,
    );
    assert!(
        is_arg_unresolved(&result, 2),
        "-variable needs identifier resolution"
    );
}

#[test]
fn test_not_variable() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const flag = true;
crypto.pbkdf2Sync(password, salt, 100000, 32, !flag);
"#,
    );
    assert!(
        is_arg_unresolved(&result, 4),
        "!variable needs identifier resolution"
    );
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_negative_zero() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, -0, 32, 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(0), "-0 = 0");
}

#[test]
fn test_negative_hex() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, -0xFF, 32, 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(-255), "-0xFF = -255");
}

#[test]
fn test_triple_negative() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, - - -42, 32, 'sha256');
"#,
    );
    // With spaces, JS parses these as separate unary operators
    assert_eq!(get_first_arg_int(&result, 2), Some(-42), "- - -42 = -42");
}
