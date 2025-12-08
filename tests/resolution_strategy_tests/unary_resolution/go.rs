//! Go unary resolution tests

use super::test_utils::{get_arg_expression, get_first_arg_int, is_arg_unresolved, scan_go};

// =============================================================================
// Negation Operator
// =============================================================================

#[test]
fn test_negative_integer() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, -1, 32, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(-1), "Negative one");
}

#[test]
fn test_negative_large_integer() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, -100000, 32, h) }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(-100000),
        "Negative large integer"
    );
}

#[test]
fn test_negative_hex() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, -0xFF, 32, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(-255), "Negative hex");
}

#[test]
fn test_double_negative() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, --42, 32, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(42), "Double negative");
}

#[test]
fn test_negative_parenthesized() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, -(-100), 32, h) }
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
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, +100000, 32, h) }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(100000),
        "Positive operator"
    );
}

#[test]
fn test_positive_then_negative() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, +-42, 32, h) }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(-42),
        "Positive then negative"
    );
}

// =============================================================================
// Bitwise NOT Operator
// =============================================================================

#[test]
fn test_bitwise_not_zero() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, ^0, 32, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(-1), "^0 = -1");
}

#[test]
fn test_bitwise_not_255() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, ^255, 32, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(-256), "^255 = -256");
}

#[test]
fn test_bitwise_not_hex() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, ^0xFF, 32, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(-256), "^0xFF = -256");
}

// =============================================================================
// Logical NOT Operator
// =============================================================================

#[test]
fn test_logical_not_true() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 32, !true) }
"#,
    );
    // !true resolves to 0 (false)
    assert_eq!(get_first_arg_int(&result, 4), Some(0), "!true = 0");
}

#[test]
fn test_logical_not_false() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 32, !false) }
"#,
    );
    // !false resolves to 1 (true)
    assert_eq!(get_first_arg_int(&result, 4), Some(1), "!false = 1");
}

#[test]
fn test_double_logical_not() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 32, !!true) }
"#,
    );
    // !!true = !0 = 1
    assert_eq!(get_first_arg_int(&result, 4), Some(1), "!!true = 1");
}

// =============================================================================
// Reference/Address-of Operator
// =============================================================================

#[test]
fn test_address_of_variable() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { 
    var x int
    pbkdf2.Key(p, s, &x, 32, h) 
}
"#,
    );
    // Address-of produces a partial expression
    assert!(is_arg_unresolved(&result, 2), "&x is unresolved");
    assert_eq!(
        get_arg_expression(&result, 2),
        Some("&x".to_string()),
        "Expression preserved"
    );
}

// =============================================================================
// Dereference Operator
// =============================================================================

#[test]
fn test_dereference_pointer() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { 
    var x *int
    pbkdf2.Key(p, s, *x, 32, h) 
}
"#,
    );
    // Dereference produces a partial expression
    assert!(is_arg_unresolved(&result, 2), "*x is unresolved");
    assert_eq!(
        get_arg_expression(&result, 2),
        Some("*x".to_string()),
        "Expression preserved"
    );
}

// =============================================================================
// Unresolved Operands
// =============================================================================

#[test]
fn test_negative_variable() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
var someVar = 10000
func main() { pbkdf2.Key(p, s, -someVar, 32, h) }
"#,
    );
    // Unary operation on unresolved identifier produces partial expression
    assert!(
        is_arg_unresolved(&result, 2),
        "-someVar needs identifier resolution"
    );
}

#[test]
fn test_not_variable() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
var flag = true
func main() { pbkdf2.Key(p, s, 10000, 32, !flag) }
"#,
    );
    // Unary operation on unresolved identifier produces partial expression
    assert!(
        is_arg_unresolved(&result, 4),
        "!flag needs identifier resolution"
    );
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_negative_zero() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, -0, 32, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(0), "-0 = 0");
}

#[test]
fn test_triple_negative() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, ---42, 32, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(-42), "---42 = -42");
}
