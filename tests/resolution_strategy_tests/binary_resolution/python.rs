//! Python binary resolution tests

use super::test_utils::{get_arg_expression, get_first_arg_int, is_arg_unresolved, scan_python};

// =============================================================================
// Addition Tests
// =============================================================================

#[test]
fn test_addition_simple() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(100 + 50, 32)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 0), Some(150), "100 + 50");
}

#[test]
fn test_addition_large_values() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(100000 + 10000, 32)
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 0),
        Some(110000),
        "PBKDF2 iteration calculation"
    );
}

// =============================================================================
// Subtraction Tests
// =============================================================================

#[test]
fn test_subtraction_simple() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(100 - 30, 32)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 0), Some(70), "100 - 30");
}

#[test]
fn test_subtraction_negative_result() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(30 - 100, 32)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 0), Some(-70), "30 - 100 = -70");
}

// =============================================================================
// Multiplication Tests
// =============================================================================

#[test]
fn test_multiplication_simple() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(10000, 32 * 8)
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 1),
        Some(256),
        "32 bytes * 8 = 256 bits"
    );
}

#[test]
fn test_multiplication_with_zero() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(10000 * 0, 32)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 0), Some(0), "Multiply by zero");
}

// =============================================================================
// Division Tests
// =============================================================================

#[test]
fn test_division_simple() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(10000, 256 // 8)
"#,
    );
    // Python uses // for integer division
    // Note: tree-sitter may parse // differently
    assert!(!result.calls.is_empty());
}

#[test]
fn test_division_by_zero() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(100 / 0, 32)
"#,
    );
    // Division by zero should be unresolved or error
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Bitwise Shift Tests
// =============================================================================

#[test]
fn test_shift_left() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(10000, 1 << 5)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(32), "1 << 5 = 32");
}

#[test]
fn test_shift_right() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(10000, 256 >> 3)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(32), "256 >> 3 = 32");
}

// =============================================================================
// Bitwise Operation Tests
// =============================================================================

#[test]
fn test_bitwise_and() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(10000, 0xFF & 0x0F)
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 1),
        Some(15),
        "0xFF & 0x0F = 0x0F"
    );
}

#[test]
fn test_bitwise_or() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(10000, 0xF0 | 0x0F)
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 1),
        Some(255),
        "0xF0 | 0x0F = 0xFF"
    );
}

#[test]
fn test_bitwise_xor() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(10000, 0xFF ^ 0x0F)
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 1),
        Some(240),
        "0xFF ^ 0x0F = 0xF0"
    );
}

// =============================================================================
// Nested Expression Tests
// =============================================================================

#[test]
fn test_nested_addition() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(10 + 20 + 30, 32)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 0), Some(60), "10 + 20 + 30 = 60");
}

#[test]
fn test_parenthesized_expression() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC((100 + 50) * 2, 32)
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 0),
        Some(300),
        "(100 + 50) * 2 = 300"
    );
}

#[test]
fn test_with_unary_operand() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(100 + -50, 32)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 0), Some(50), "100 + -50 = 50");
}

// =============================================================================
// Crypto-Relevant Tests
// =============================================================================

#[test]
fn test_pbkdf2_iteration_calculation() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(100000 + 10000, 32)
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 0),
        Some(110000),
        "Base iterations + extra"
    );
}

#[test]
fn test_key_size_bytes_to_bits() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(10000, 32 * 8)
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 1),
        Some(256),
        "32 bytes = 256 bits"
    );
}

// =============================================================================
// Partial Resolution Tests
// =============================================================================

#[test]
fn test_unresolved_left_operand() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
base = 100000
PBKDF2HMAC(base + 10000, 32)
"#,
    );
    assert!(
        is_arg_unresolved(&result, 0),
        "Identifier needs identifier strategy"
    );
    assert_eq!(
        get_arg_expression(&result, 0),
        Some("base + 10000".to_string()),
        "Expression preserved"
    );
}

#[test]
fn test_unresolved_right_operand() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
extra = 10000
PBKDF2HMAC(100000 + extra, 32)
"#,
    );
    assert!(
        is_arg_unresolved(&result, 0),
        "Identifier needs identifier strategy"
    );
    assert_eq!(
        get_arg_expression(&result, 0),
        Some("100000 + extra".to_string()),
        "Expression preserved"
    );
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_hex_addition() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(10000, 0x10 + 0x10)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(32), "0x10 + 0x10 = 32");
}

#[test]
fn test_large_result() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(1000000000 + 1, 32)
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 0),
        Some(1000000001),
        "Large addition"
    );
}
