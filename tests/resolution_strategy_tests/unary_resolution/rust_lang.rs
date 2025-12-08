//! Rust unary resolution tests

use super::test_utils::{get_arg_expression, get_first_arg_int, is_arg_unresolved, scan_rust};

// =============================================================================
// Negation Operator
// =============================================================================

#[test]
fn test_negative_integer() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(algorithm, -1, salt, password, &mut out);
}
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(-1), "Negative one");
}

#[test]
fn test_negative_large_integer() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(algorithm, -100000, salt, password, &mut out);
}
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 1),
        Some(-100000),
        "Negative large integer"
    );
}

#[test]
fn test_double_negative() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(algorithm, --42, salt, password, &mut out);
}
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(42), "Double negative");
}

#[test]
fn test_negative_parenthesized() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(algorithm, -(-100), salt, password, &mut out);
}
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 1),
        Some(100),
        "Negative parenthesized"
    );
}

// =============================================================================
// Logical NOT Operator
// =============================================================================

#[test]
fn test_not_true() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(algorithm, 100000, salt, !true, &mut out);
}
"#,
    );
    // !true = false = 0
    assert_eq!(get_first_arg_int(&result, 3), Some(0), "!true = 0");
}

#[test]
fn test_not_false() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(algorithm, 100000, salt, !false, &mut out);
}
"#,
    );
    // !false = true = 1
    assert_eq!(get_first_arg_int(&result, 3), Some(1), "!false = 1");
}

#[test]
fn test_double_not() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(algorithm, 100000, salt, !!true, &mut out);
}
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(1), "!!true = 1");
}

// =============================================================================
// Reference Operator
// =============================================================================

#[test]
fn test_reference_variable() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    let x = 42;
    pbkdf2::derive(algorithm, 100000, salt, &x, &mut out);
}
"#,
    );
    // Reference produces partial expression
    assert!(is_arg_unresolved(&result, 3), "&x is unresolved");
    assert_eq!(
        get_arg_expression(&result, 3),
        Some("&x".to_string()),
        "Expression preserved"
    );
}

#[test]
fn test_mutable_reference() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    let mut out = [0u8; 32];
    pbkdf2::derive(algorithm, 100000, salt, password, &mut out);
}
"#,
    );
    // &mut reference produces partial expression
    assert!(is_arg_unresolved(&result, 4), "&mut out is unresolved");
}

// =============================================================================
// Dereference Operator
// =============================================================================

#[test]
fn test_dereference_pointer() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    let x: &i32 = &42;
    pbkdf2::derive(algorithm, *x, salt, password, &mut out);
}
"#,
    );
    // Dereference produces partial expression
    assert!(is_arg_unresolved(&result, 1), "*x is unresolved");
    assert_eq!(
        get_arg_expression(&result, 1),
        Some("*x".to_string()),
        "Expression preserved"
    );
}

// =============================================================================
// Unresolved Operands
// =============================================================================

#[test]
fn test_negative_variable() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    let iterations = 100000;
    pbkdf2::derive(algorithm, -iterations, salt, password, &mut out);
}
"#,
    );
    assert!(
        is_arg_unresolved(&result, 1),
        "-variable needs identifier resolution"
    );
}

#[test]
fn test_not_variable() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    let flag = true;
    pbkdf2::derive(algorithm, 100000, salt, !flag, &mut out);
}
"#,
    );
    assert!(
        is_arg_unresolved(&result, 3),
        "!variable needs identifier resolution"
    );
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_negative_zero() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(algorithm, -0, salt, password, &mut out);
}
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(0), "-0 = 0");
}

#[test]
fn test_negative_hex() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(algorithm, -0xFF, salt, password, &mut out);
}
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(-255), "-0xFF = -255");
}

#[test]
fn test_triple_negative() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(algorithm, ---42, salt, password, &mut out);
}
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(-42), "---42 = -42");
}
