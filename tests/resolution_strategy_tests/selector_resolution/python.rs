//! Python attribute (selector) resolution tests
//!
//! Tests attribute access in Python code:
//! - Module attribute access (module.constant)
//! - Object attribute access (obj.attr)
//! - Chained attributes (a.b.c)
//! - Class attribute access

use super::test_utils::*;

// =============================================================================
// Module Attribute Access
// =============================================================================

#[test]
fn test_python_hashlib_attribute() {
    let source = r#"
import hashlib
result = hashlib.sha256(data)
"#;
    let result = scan_python(source);
    assert!(!result.calls.is_empty());
    // hashlib.sha256 is recognized as a call
}

// =============================================================================
// Object Attribute Access
// =============================================================================

#[test]
fn test_python_object_attribute() {
    let source = r#"
from hashlib import pbkdf2_hmac
class Config:
    iterations = 10000

cfg = Config()
pbkdf2_hmac('sha256', password, salt, cfg.iterations)
"#;
    let result = scan_python(source);
    assert!(!result.calls.is_empty());

    // cfg.iterations should be unresolved
    assert!(is_arg_unresolved(&result, 3));
    let expr = get_arg_expression(&result, 3);
    assert_eq!(expr, Some("cfg.iterations".to_string()));
}

#[test]
fn test_python_self_attribute() {
    let source = r#"
from hashlib import pbkdf2_hmac
class Crypto:
    def __init__(self):
        self.iterations = 10000
    
    def derive(self, password, salt):
        return pbkdf2_hmac('sha256', password, salt, self.iterations)
"#;
    let result = scan_python(source);
    assert!(!result.calls.is_empty());

    // self.iterations - method receiver attribute
    assert!(is_arg_unresolved(&result, 3));
    let expr = get_arg_expression(&result, 3);
    assert_eq!(expr, Some("self.iterations".to_string()));
}

// =============================================================================
// Chained Attributes
// =============================================================================

#[test]
fn test_python_chained_attribute_two() {
    let source = r#"
from hashlib import pbkdf2_hmac
result = pbkdf2_hmac('sha256', password, salt, app.config.iterations)
"#;
    let result = scan_python(source);
    assert!(!result.calls.is_empty());

    // app.config.iterations
    assert!(is_arg_unresolved(&result, 3));
    let expr = get_arg_expression(&result, 3);
    assert_eq!(expr, Some("app.config.iterations".to_string()));
}

#[test]
fn test_python_chained_attribute_three() {
    let source = r#"
from hashlib import pbkdf2_hmac
result = pbkdf2_hmac('sha256', password, salt, a.b.c.d)
"#;
    let result = scan_python(source);
    assert!(!result.calls.is_empty());

    // a.b.c.d - deeply chained
    assert!(is_arg_unresolved(&result, 3));
    let expr = get_arg_expression(&result, 3);
    assert_eq!(expr, Some("a.b.c.d".to_string()));
}

// =============================================================================
// Mixed Resolved and Unresolved
// =============================================================================

#[test]
fn test_python_literal_with_attribute() {
    let source = r#"
from hashlib import pbkdf2_hmac
result = pbkdf2_hmac('sha256', password, salt, 10000, cfg.key_length)
"#;
    let result = scan_python(source);
    assert!(!result.calls.is_empty());

    // 'sha256' should resolve
    assert!(is_arg_resolved(&result, 0));
    assert_eq!(get_first_arg_string(&result, 0), Some("sha256".to_string()));

    // 10000 should resolve
    assert!(is_arg_resolved(&result, 3));
    assert_eq!(get_first_arg_int(&result, 3), Some(10000));

    // cfg.key_length should be unresolved
    assert!(is_arg_unresolved(&result, 4));
}

// =============================================================================
// Attribute in Binary Expression
// =============================================================================

#[test]
fn test_python_attribute_binary_add() {
    let source = r#"
from hashlib import pbkdf2_hmac
BASE = 100000
result = pbkdf2_hmac('sha256', password, salt, config.extra + BASE)
"#;
    let result = scan_python(source);
    assert!(!result.calls.is_empty());

    // config.extra + BASE - contains selector, should be unresolved
    assert!(is_arg_unresolved(&result, 3));
}

#[test]
fn test_python_attribute_binary_mul() {
    let source = r#"
from hashlib import pbkdf2_hmac
result = pbkdf2_hmac('sha256', password, salt, cfg.base * 2)
"#;
    let result = scan_python(source);
    assert!(!result.calls.is_empty());

    // cfg.base * 2
    assert!(is_arg_unresolved(&result, 3));
}

// =============================================================================
// Class Constants
// =============================================================================

#[test]
fn test_python_class_constant() {
    let source = r#"
from hashlib import pbkdf2_hmac
class Crypto:
    DEFAULT_ITERATIONS = 100000

result = pbkdf2_hmac('sha256', password, salt, Crypto.DEFAULT_ITERATIONS)
"#;
    let result = scan_python(source);
    assert!(!result.calls.is_empty());

    // Crypto.DEFAULT_ITERATIONS - class attribute
    assert!(is_arg_unresolved(&result, 3));
    let expr = get_arg_expression(&result, 3);
    assert_eq!(expr, Some("Crypto.DEFAULT_ITERATIONS".to_string()));
}

// =============================================================================
// Not Attributes (Control Cases)
// =============================================================================

#[test]
fn test_python_not_attribute_literal() {
    let source = r#"
from hashlib import pbkdf2_hmac
result = pbkdf2_hmac('sha256', password, salt, 10000)
"#;
    let result = scan_python(source);
    assert!(!result.calls.is_empty());

    // 10000 is a literal
    assert!(is_arg_resolved(&result, 3));
    assert_eq!(get_first_arg_int(&result, 3), Some(10000));
}

#[test]
fn test_python_not_attribute_local_var() {
    let source = r#"
from hashlib import pbkdf2_hmac
def test():
    iterations = 10000
    result = pbkdf2_hmac('sha256', password, salt, iterations)
"#;
    let result = scan_python(source);
    assert!(!result.calls.is_empty());

    // iterations is a local variable, should resolve
    assert!(is_arg_resolved(&result, 3));
    assert_eq!(get_first_arg_int(&result, 3), Some(10000));
}

// =============================================================================
// Multiple Attributes in Same Call
// =============================================================================

#[test]
fn test_python_multiple_attributes() {
    let source = r#"
from hashlib import pbkdf2_hmac
result = pbkdf2_hmac(cfg.algorithm, cfg.password, cfg.salt, cfg.iterations)
"#;
    let result = scan_python(source);
    assert!(!result.calls.is_empty());

    // All arguments should be unresolved attributes
    for i in 0..4 {
        assert!(
            is_arg_unresolved(&result, i),
            "Arg {i} should be unresolved"
        );
        let expr = get_arg_expression(&result, i);
        assert!(expr.is_some(), "Arg {i} should have expression");
    }
}
