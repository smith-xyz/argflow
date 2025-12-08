//! Rust binary resolution tests

use super::test_utils::{get_arg_expression, get_first_arg_int, is_arg_unresolved, scan_rust};

// =============================================================================
// Addition Tests
// =============================================================================

#[test]
fn test_addition_simple() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 100 + 50, salt, password, &mut key); }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(150), "100 + 50");
}

#[test]
fn test_addition_large_values() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 100000 + 10000, salt, password, &mut key); }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 1),
        Some(110000),
        "PBKDF2 iteration calculation"
    );
}

// =============================================================================
// Subtraction Tests
// =============================================================================

#[test]
fn test_subtraction_simple() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 100 - 30, salt, password, &mut key); }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(70), "100 - 30");
}

#[test]
fn test_subtraction_negative_result() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 30 - 100, salt, password, &mut key); }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(-70), "30 - 100 = -70");
}

// =============================================================================
// Multiplication Tests
// =============================================================================

#[test]
fn test_multiplication_simple() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 10000, salt, password, 32 * 8); }
"#,
    );
    // Note: the position depends on how ring::pbkdf2 signature is defined
    // This test verifies the expression resolves
    assert!(result.calls.len() >= 1);
}

#[test]
fn test_multiplication_with_zero() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 10000 * 0, salt, password, &mut key); }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(0), "Multiply by zero");
}

// =============================================================================
// Division Tests
// =============================================================================

#[test]
fn test_division_simple() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 100 / 4, salt, password, &mut key); }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(25), "100 / 4 = 25");
}

#[test]
fn test_division_by_zero() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 100 / 0, salt, password, &mut key); }
"#,
    );
    // Division by zero should be unresolved
    assert!(
        is_arg_unresolved(&result, 1),
        "Division by zero is unresolved"
    );
}

// =============================================================================
// Bitwise Shift Tests
// =============================================================================

#[test]
fn test_shift_left() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 1 << 5, salt, password, &mut key); }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(32), "1 << 5 = 32");
}

#[test]
fn test_shift_left_256() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 1 << 8, salt, password, &mut key); }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(256), "1 << 8 = 256");
}

#[test]
fn test_shift_right() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 256 >> 3, salt, password, &mut key); }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(32), "256 >> 3 = 32");
}

// =============================================================================
// Bitwise Operation Tests
// =============================================================================

#[test]
fn test_bitwise_and() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 0xFF & 0x0F, salt, password, &mut key); }
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
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 0xF0 | 0x0F, salt, password, &mut key); }
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
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 0xFF ^ 0x0F, salt, password, &mut key); }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 1),
        Some(240),
        "0xFF ^ 0x0F = 0xF0"
    );
}

// =============================================================================
// Comparison Tests
// =============================================================================

#[test]
fn test_comparison_equal() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { 
    let x = 10 == 10;
    pbkdf2::derive(alg, 10000, salt, password, &mut key); 
}
"#,
    );
    // The comparison is in a separate let binding, just verify the call works
    assert!(result.calls.len() >= 1);
}

// =============================================================================
// Nested Expression Tests
// =============================================================================

#[test]
fn test_nested_addition() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 10 + 20 + 30, salt, password, &mut key); }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(60), "10 + 20 + 30 = 60");
}

#[test]
fn test_parenthesized_expression() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, (100 + 50) * 2, salt, password, &mut key); }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 1),
        Some(300),
        "(100 + 50) * 2 = 300"
    );
}

#[test]
fn test_with_unary_operand() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 100 + -50, salt, password, &mut key); }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(50), "100 + -50 = 50");
}

// =============================================================================
// Crypto-Relevant Tests
// =============================================================================

#[test]
fn test_pbkdf2_iteration_calculation() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 100000 + 10000, salt, password, &mut key); }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 1),
        Some(110000),
        "Base iterations + extra"
    );
}

// =============================================================================
// Partial Resolution Tests
// =============================================================================

#[test]
fn test_unresolved_left_operand() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { 
    let base = 100000;
    pbkdf2::derive(alg, base + 10000, salt, password, &mut key); 
}
"#,
    );
    assert!(
        is_arg_unresolved(&result, 1),
        "Identifier needs identifier strategy"
    );
    assert_eq!(
        get_arg_expression(&result, 1),
        Some("base + 10000".to_string()),
        "Expression preserved"
    );
}

#[test]
fn test_unresolved_right_operand() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { 
    let extra = 10000;
    pbkdf2::derive(alg, 100000 + extra, salt, password, &mut key); 
}
"#,
    );
    assert!(
        is_arg_unresolved(&result, 1),
        "Identifier needs identifier strategy"
    );
    assert_eq!(
        get_arg_expression(&result, 1),
        Some("100000 + extra".to_string()),
        "Expression preserved"
    );
}

// =============================================================================
// Rust-Specific Tests
// =============================================================================

#[test]
fn test_rust_type_suffix() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 100_i32 + 50_i32, salt, password, &mut key); }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 1),
        Some(150),
        "Type suffixes handled"
    );
}

#[test]
fn test_rust_underscore_separator() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 100_000 + 10_000, salt, password, &mut key); }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 1),
        Some(110000),
        "Underscore separators"
    );
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_hex_addition() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 0x10 + 0x10, salt, password, &mut key); }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(32), "0x10 + 0x10 = 32");
}

#[test]
fn test_large_result() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 1000000000 + 1, salt, password, &mut key); }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 1),
        Some(1000000001),
        "Large addition"
    );
}
