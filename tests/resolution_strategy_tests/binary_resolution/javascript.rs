//! JavaScript binary resolution tests

use super::test_utils::{get_first_arg_int, is_arg_unresolved, scan_javascript};

// =============================================================================
// Addition Tests
// =============================================================================

#[test]
fn test_addition_simple() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 100 + 50, 32, 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(150), "100 + 50");
}

#[test]
fn test_addition_large_values() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 100000 + 10000, 32, 'sha256');
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(110000),
        "PBKDF2 iteration calculation"
    );
}

// =============================================================================
// Subtraction Tests
// =============================================================================

#[test]
fn test_subtraction_simple() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 100 - 30, 32, 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(70), "100 - 30");
}

#[test]
fn test_subtraction_negative_result() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 30 - 100, 32, 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(-70), "30 - 100 = -70");
}

// =============================================================================
// Multiplication Tests
// =============================================================================

#[test]
fn test_multiplication_simple() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 10000, 32 * 8, 'sha256');
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(256),
        "32 bytes * 8 = 256 bits"
    );
}

#[test]
fn test_multiplication_with_zero() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 10000 * 0, 32, 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(0), "Multiply by zero");
}

// =============================================================================
// Division Tests
// =============================================================================

#[test]
fn test_division_simple() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 10000, 256 / 8, 'sha256');
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(32),
        "256 bits / 8 = 32 bytes"
    );
}

#[test]
fn test_division_by_zero() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 100 / 0, 32, 'sha256');
"#,
    );
    // Division by zero should be unresolved
    assert!(
        is_arg_unresolved(&result, 2),
        "Division by zero is unresolved"
    );
}

// =============================================================================
// Bitwise Shift Tests
// =============================================================================

#[test]
fn test_shift_left() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 10000, 1 << 5, 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(32), "1 << 5 = 32");
}

#[test]
fn test_shift_right() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 10000, 256 >> 3, 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(32), "256 >> 3 = 32");
}

// =============================================================================
// Bitwise Operation Tests
// =============================================================================

#[test]
fn test_bitwise_and() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 10000, 0xFF & 0x0F, 'sha256');
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(15),
        "0xFF & 0x0F = 0x0F"
    );
}

#[test]
fn test_bitwise_or() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 10000, 0xF0 | 0x0F, 'sha256');
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(255),
        "0xF0 | 0x0F = 0xFF"
    );
}

#[test]
fn test_bitwise_xor() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 10000, 0xFF ^ 0x0F, 'sha256');
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(240),
        "0xFF ^ 0x0F = 0xF0"
    );
}

// =============================================================================
// Comparison Tests
// =============================================================================

#[test]
fn test_comparison_equal() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 10000, 32, 10 == 10);
"#,
    );
    // Note: JavaScript == is different from ===, but we handle both
    assert!(!result.calls.is_empty());
}

#[test]
fn test_comparison_strict_equal() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 10000, 32, 10 === 10);
"#,
    );
    // === might not be in our operator list, but the call should still be detected
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Nested Expression Tests
// =============================================================================

#[test]
fn test_nested_addition() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 10 + 20 + 30, 32, 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(60), "10 + 20 + 30 = 60");
}

#[test]
fn test_parenthesized_expression() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, (100 + 50) * 2, 32, 'sha256');
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(300),
        "(100 + 50) * 2 = 300"
    );
}

#[test]
fn test_with_unary_operand() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 100 + -50, 32, 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(50), "100 + -50 = 50");
}

// =============================================================================
// Crypto-Relevant Tests
// =============================================================================

#[test]
fn test_pbkdf2_iteration_calculation() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 100000 + 10000, 32, 'sha256');
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(110000),
        "Base iterations + extra"
    );
}

#[test]
fn test_key_size_bytes_to_bits() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 10000, 32 * 8, 'sha256');
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(256),
        "32 bytes = 256 bits"
    );
}

// =============================================================================
// Binary Expression with Identifier Resolution
// Binary strategy now properly resolves identifiers via the Resolver
// =============================================================================

#[test]
fn test_identifier_left_operand_resolved() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const base = 100000;
crypto.pbkdf2Sync(p, s, base + 10000, 32, 'sha256');
"#,
    );
    // File-level const resolves via identifier strategy
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(110000),
        "base (100000) + 10000 = 110000"
    );
}

#[test]
fn test_identifier_right_operand_resolved() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const extra = 10000;
crypto.pbkdf2Sync(p, s, 100000 + extra, 32, 'sha256');
"#,
    );
    // File-level const resolves via identifier strategy
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(110000),
        "100000 + extra (10000) = 110000"
    );
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_hex_addition() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 10000, 0x10 + 0x10, 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(32), "0x10 + 0x10 = 32");
}

#[test]
fn test_large_result() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 1000000000 + 1, 32, 'sha256');
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(1000000001),
        "Large addition"
    );
}
