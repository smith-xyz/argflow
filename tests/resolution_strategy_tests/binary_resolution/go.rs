//! Go binary resolution tests

use super::test_utils::{get_arg_expression, get_first_arg_int, is_arg_unresolved, scan_go};

// =============================================================================
// Addition Tests
// =============================================================================

#[test]
fn test_addition_simple() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 100 + 50, 32, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(150), "100 + 50");
}

#[test]
fn test_addition_large_values() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 100000 + 10000, 32, h) }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(110000),
        "PBKDF2 iteration calculation"
    );
}

#[test]
fn test_addition_with_zero() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000 + 0, 32, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(10000), "Adding zero");
}

// =============================================================================
// Subtraction Tests
// =============================================================================

#[test]
fn test_subtraction_simple() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 100 - 30, 32, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(70), "100 - 30");
}

#[test]
fn test_subtraction_resulting_zero() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 50 - 50, 32, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(0), "50 - 50 = 0");
}

#[test]
fn test_subtraction_negative_result() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 30 - 100, 32, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(-70), "30 - 100 = -70");
}

// =============================================================================
// Multiplication Tests
// =============================================================================

#[test]
fn test_multiplication_simple() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 32 * 8, h) }
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
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000 * 0, 32, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(0), "Multiply by zero");
}

#[test]
fn test_multiplication_with_one() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000 * 1, 32, h) }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(10000),
        "Multiply by one"
    );
}

// =============================================================================
// Division Tests
// =============================================================================

#[test]
fn test_division_simple() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 256 / 8, h) }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(32),
        "256 bits / 8 = 32 bytes"
    );
}

#[test]
fn test_division_integer_truncation() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10 / 3, 32, h) }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(3),
        "Integer division truncates"
    );
}

#[test]
fn test_division_by_zero() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 100 / 0, 32, h) }
"#,
    );
    assert!(
        is_arg_unresolved(&result, 2),
        "Division by zero is unresolved"
    );
    assert_eq!(
        get_arg_expression(&result, 2),
        Some("100 / 0".to_string()),
        "Expression preserved"
    );
}

// =============================================================================
// Modulo Tests
// =============================================================================

#[test]
fn test_modulo_simple() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 100 % 30, 32, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(10), "100 % 30 = 10");
}

#[test]
fn test_modulo_by_zero() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 100 % 0, 32, h) }
"#,
    );
    assert!(
        is_arg_unresolved(&result, 2),
        "Modulo by zero is unresolved"
    );
}

// =============================================================================
// Bitwise Shift Tests
// =============================================================================

#[test]
fn test_shift_left() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 1 << 5, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(32), "1 << 5 = 32");
}

#[test]
fn test_shift_left_key_size() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 1 << 8, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(256), "1 << 8 = 256");
}

#[test]
fn test_shift_right() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 256 >> 3, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(32), "256 >> 3 = 32");
}

// =============================================================================
// Bitwise Operation Tests
// =============================================================================

#[test]
fn test_bitwise_and() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 0xFF & 0x0F, h) }
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
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 0xF0 | 0x0F, h) }
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
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 0xFF ^ 0x0F, h) }
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
fn test_comparison_equal_true() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 32, 10 == 10) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 4), Some(1), "10 == 10 is true");
}

#[test]
fn test_comparison_equal_false() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 32, 10 == 20) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 4), Some(0), "10 == 20 is false");
}

#[test]
fn test_comparison_not_equal() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 32, 10 != 20) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 4), Some(1), "10 != 20 is true");
}

#[test]
fn test_comparison_less_than() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 32, 5 < 10) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 4), Some(1), "5 < 10 is true");
}

#[test]
fn test_comparison_greater_than() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 32, 10 > 5) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 4), Some(1), "10 > 5 is true");
}

#[test]
fn test_comparison_less_equal() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 32, 10 <= 10) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 4), Some(1), "10 <= 10 is true");
}

#[test]
fn test_comparison_greater_equal() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 32, 10 >= 10) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 4), Some(1), "10 >= 10 is true");
}

// =============================================================================
// Logical Operator Tests
// =============================================================================

#[test]
fn test_logical_and_true() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 32, 1 != 0 && 2 != 0) }
"#,
    );
    // The && expression should resolve - outer is the logical and
    // We need to check that we're getting the right expression
    assert!(!result.calls.is_empty());
}

#[test]
fn test_logical_or_true() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 32, 0 == 0 || 1 == 0) }
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Nested Expression Tests
// =============================================================================

#[test]
fn test_nested_addition() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10 + 20 + 30, 32, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(60), "10 + 20 + 30 = 60");
}

#[test]
fn test_nested_multiplication() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 2 * 2 * 8, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(32), "2 * 2 * 8 = 32");
}

#[test]
fn test_parenthesized_expression() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, (100 + 50) * 2, 32, h) }
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
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 100 + -50, 32, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(50), "100 + -50 = 50");
}

// =============================================================================
// Crypto-Relevant Tests
// =============================================================================

#[test]
fn test_pbkdf2_iteration_calculation() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 100000 + 10000, 32, h) }
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
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 32 * 8, h) }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(256),
        "32 bytes = 256 bits"
    );
}

#[test]
fn test_aes_key_size_calculation() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 256 / 8, h) }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(32),
        "256 bits = 32 bytes"
    );
}

#[test]
fn test_key_size_with_shift() {
    let result = scan_go(
        r#"
package main
import "crypto/aes"
func main() { aes.NewCipher(make([]byte, 1 << 5)) }
"#,
    );
    // 1 << 5 = 32, but it's in make() so we just verify detection
    assert_eq!(result.call_count(), 1);
}

// =============================================================================
// Binary Expression with Identifier Resolution
// Binary strategy now properly resolves identifiers via the Resolver
// =============================================================================

#[test]
fn test_identifier_left_operand_resolved() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
var base = 100000
func main() { pbkdf2.Key(p, s, base + 10000, 32, h) }
"#,
    );
    // File-level var resolves via identifier strategy
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(110000),
        "base (100000) + 10000 = 110000"
    );
}

#[test]
fn test_identifier_right_operand_resolved() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
var extra = 10000
func main() { pbkdf2.Key(p, s, 100000 + extra, 32, h) }
"#,
    );
    // File-level var resolves via identifier strategy
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(110000),
        "100000 + extra (10000) = 110000"
    );
}

#[test]
fn test_both_identifier_operands_resolved() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
var base = 100000
var extra = 10000
func main() { pbkdf2.Key(p, s, base + extra, 32, h) }
"#,
    );
    // Both file-level vars resolve via identifier strategy
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(110000),
        "base (100000) + extra (10000) = 110000"
    );
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_hex_addition() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 0x10 + 0x10, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(32), "0x10 + 0x10 = 32");
}

#[test]
fn test_octal_multiplication() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 0o10 * 4, h) }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(32),
        "0o10 * 4 = 8 * 4 = 32"
    );
}

#[test]
fn test_large_result() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 1000000000 + 1, 32, h) }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(1000000001),
        "Large addition"
    );
}
