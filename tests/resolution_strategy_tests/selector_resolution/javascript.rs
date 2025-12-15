//! JavaScript member expression resolution tests
//!
//! Tests property access in JavaScript code:
//! - Object property access (obj.prop)
//! - Module property access (module.func)
//! - Chained property access (a.b.c)
//! - this property access (this.prop)

use super::test_utils::*;

// =============================================================================
// Object Property Access
// =============================================================================

#[test]
fn test_js_object_property() {
    let source = r#"
const cfg = { iterations: 10000 };
crypto.pbkdf2Sync(password, salt, cfg.iterations, 32, 'sha256');
"#;
    let result = scan_javascript(source);
    assert!(!result.calls.is_empty());

    // cfg.iterations should be unresolved
    assert!(is_arg_unresolved(&result, 2));
    let expr = get_arg_expression(&result, 2);
    assert_eq!(expr, Some("cfg.iterations".to_string()));
}

#[test]
fn test_js_this_property() {
    let source = r#"
class Crypto {
    constructor() {
        this.iterations = 10000;
    }
    
    derive(password, salt) {
        return crypto.pbkdf2Sync(password, salt, this.iterations, 32, 'sha256');
    }
}
"#;
    let result = scan_javascript(source);
    assert!(!result.calls.is_empty());

    // this.iterations
    assert!(is_arg_unresolved(&result, 2));
    let expr = get_arg_expression(&result, 2);
    assert_eq!(expr, Some("this.iterations".to_string()));
}

// =============================================================================
// Module Property Access
// =============================================================================

#[test]
fn test_js_module_function() {
    let source = r#"
const crypto = require('crypto');
crypto.createHash('sha256');
"#;
    let result = scan_javascript(source);
    assert!(!result.calls.is_empty());
    // crypto.createHash is recognized as a call
}

#[test]
fn test_js_module_constant() {
    let source = r#"
const config = require('./config');
crypto.pbkdf2Sync(password, salt, config.DEFAULT_ITERATIONS, 32, 'sha256');
"#;
    let result = scan_javascript(source);
    assert!(!result.calls.is_empty());

    // config.DEFAULT_ITERATIONS - module constant
    assert!(is_arg_unresolved(&result, 2));
    let expr = get_arg_expression(&result, 2);
    assert_eq!(expr, Some("config.DEFAULT_ITERATIONS".to_string()));
}

// =============================================================================
// Chained Property Access
// =============================================================================

#[test]
fn test_js_chained_property_two() {
    let source = r#"
crypto.pbkdf2Sync(password, salt, app.config.iterations, 32, 'sha256');
"#;
    let result = scan_javascript(source);
    assert!(!result.calls.is_empty());

    // app.config.iterations
    assert!(is_arg_unresolved(&result, 2));
    let expr = get_arg_expression(&result, 2);
    assert_eq!(expr, Some("app.config.iterations".to_string()));
}

#[test]
fn test_js_chained_property_three() {
    let source = r#"
crypto.pbkdf2Sync(password, salt, a.b.c.d, 32, 'sha256');
"#;
    let result = scan_javascript(source);
    assert!(!result.calls.is_empty());

    // a.b.c.d
    assert!(is_arg_unresolved(&result, 2));
    let expr = get_arg_expression(&result, 2);
    assert_eq!(expr, Some("a.b.c.d".to_string()));
}

// =============================================================================
// Property in Binary Expression
// =============================================================================

#[test]
fn test_js_property_binary_add() {
    let source = r#"
const BASE = 100000;
crypto.pbkdf2Sync(password, salt, config.extra + BASE, 32, 'sha256');
"#;
    let result = scan_javascript(source);
    assert!(!result.calls.is_empty());

    // config.extra + BASE - contains property, should be unresolved
    assert!(is_arg_unresolved(&result, 2));
}

#[test]
fn test_js_property_binary_mul() {
    let source = r#"
crypto.pbkdf2Sync(password, salt, cfg.base * 2, 32, 'sha256');
"#;
    let result = scan_javascript(source);
    assert!(!result.calls.is_empty());

    // cfg.base * 2
    assert!(is_arg_unresolved(&result, 2));
}

// =============================================================================
// Mixed Resolved and Unresolved
// =============================================================================

#[test]
fn test_js_literal_with_property() {
    let source = r#"
crypto.pbkdf2Sync(password, salt, 10000, cfg.keyLength, 'sha256');
"#;
    let result = scan_javascript(source);
    assert!(!result.calls.is_empty());

    // 10000 should resolve
    assert!(is_arg_resolved(&result, 2));
    assert_eq!(get_first_arg_int(&result, 2), Some(10000));

    // cfg.keyLength should be unresolved
    assert!(is_arg_unresolved(&result, 3));
}

#[test]
fn test_js_string_literal_with_property() {
    let source = r#"
crypto.pbkdf2Sync(password, salt, cfg.iterations, 32, 'sha256');
"#;
    let result = scan_javascript(source);
    assert!(!result.calls.is_empty());

    // 'sha256' should resolve
    assert!(is_arg_resolved(&result, 4));
    assert_eq!(get_first_arg_string(&result, 4), Some("sha256".to_string()));

    // cfg.iterations should be unresolved
    assert!(is_arg_unresolved(&result, 2));
}

// =============================================================================
// Class Properties
// =============================================================================

#[test]
fn test_js_class_static_property() {
    let source = r#"
class Crypto {
    static DEFAULT_ITERATIONS = 100000;
}

crypto.pbkdf2Sync(password, salt, Crypto.DEFAULT_ITERATIONS, 32, 'sha256');
"#;
    let result = scan_javascript(source);
    assert!(!result.calls.is_empty());

    // Crypto.DEFAULT_ITERATIONS - static class property
    assert!(is_arg_unresolved(&result, 2));
    let expr = get_arg_expression(&result, 2);
    assert_eq!(expr, Some("Crypto.DEFAULT_ITERATIONS".to_string()));
}

// =============================================================================
// Not Property Access (Control Cases)
// =============================================================================

#[test]
fn test_js_not_property_literal() {
    let source = r#"
crypto.pbkdf2Sync(password, salt, 10000, 32, 'sha256');
"#;
    let result = scan_javascript(source);
    assert!(!result.calls.is_empty());

    assert!(is_arg_resolved(&result, 2));
    assert_eq!(get_first_arg_int(&result, 2), Some(10000));
}

#[test]
fn test_js_not_property_local_const() {
    let source = r#"
function test() {
    const iterations = 10000;
    crypto.pbkdf2Sync(password, salt, iterations, 32, 'sha256');
}
"#;
    let result = scan_javascript(source);
    assert!(!result.calls.is_empty());

    // iterations is a local constant, should resolve
    assert!(is_arg_resolved(&result, 2));
    assert_eq!(get_first_arg_int(&result, 2), Some(10000));
}

#[test]
fn test_js_not_property_file_const() {
    let source = r#"
const ITERATIONS = 100000;

function test() {
    crypto.pbkdf2Sync(password, salt, ITERATIONS, 32, 'sha256');
}
"#;
    let result = scan_javascript(source);
    assert!(!result.calls.is_empty());

    // ITERATIONS is a file-level constant, should resolve
    assert!(is_arg_resolved(&result, 2));
    assert_eq!(get_first_arg_int(&result, 2), Some(100000));
}

// =============================================================================
// Multiple Properties in Same Call
// =============================================================================

#[test]
fn test_js_multiple_properties() {
    let source = r#"
crypto.pbkdf2Sync(cfg.password, cfg.salt, cfg.iterations, cfg.keyLength, cfg.hash);
"#;
    let result = scan_javascript(source);
    assert!(!result.calls.is_empty());

    // All arguments should be unresolved properties
    for i in 0..5 {
        assert!(
            is_arg_unresolved(&result, i),
            "Arg {i} should be unresolved"
        );
        let expr = get_arg_expression(&result, i);
        assert!(expr.is_some(), "Arg {i} should have expression");
    }
}

// =============================================================================
// Arrow Functions with Properties
// =============================================================================

#[test]
fn test_js_arrow_function_property() {
    let source = r#"
const derive = (config) => {
    return crypto.pbkdf2Sync(password, salt, config.iterations, 32, 'sha256');
};
"#;
    let result = scan_javascript(source);
    assert!(!result.calls.is_empty());

    // config.iterations in arrow function
    assert!(is_arg_unresolved(&result, 2));
    let expr = get_arg_expression(&result, 2);
    assert_eq!(expr, Some("config.iterations".to_string()));
}

// =============================================================================
// Prototype Property
// =============================================================================

#[test]
fn test_js_prototype_property() {
    let source = r#"
Crypto.prototype.iterations = 10000;
crypto.pbkdf2Sync(password, salt, Crypto.prototype.iterations, 32, 'sha256');
"#;
    let result = scan_javascript(source);
    assert!(!result.calls.is_empty());

    // Crypto.prototype.iterations - chained property
    assert!(is_arg_unresolved(&result, 2));
}
