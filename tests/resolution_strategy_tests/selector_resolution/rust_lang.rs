//! Rust field expression resolution tests
//!
//! Tests field access in Rust code:
//! - Struct field access (s.field)
//! - Module path access (module::Constant)
//! - Chained field access (a.b.c)
//! - Tuple field access (t.0)

use super::test_utils::*;

// =============================================================================
// Struct Field Access
// =============================================================================

#[test]
fn test_rust_struct_field() {
    let source = r#"
struct Config {
    iterations: u32,
}

fn test() {
    let cfg = Config { iterations: 10000 };
    pbkdf2::derive(password, salt, cfg.iterations, 32);
}
"#;
    let result = scan_rust(source);
    assert!(!result.calls.is_empty());

    // cfg.iterations should be unresolved
    assert!(is_arg_unresolved(&result, 2));
    let expr = get_arg_expression(&result, 2);
    assert_eq!(expr, Some("cfg.iterations".to_string()));
}

#[test]
fn test_rust_self_field() {
    let source = r#"
struct Crypto {
    iterations: u32,
}

impl Crypto {
    fn derive(&self, password: &[u8], salt: &[u8]) {
        pbkdf2::derive(password, salt, self.iterations, 32);
    }
}
"#;
    let result = scan_rust(source);
    assert!(!result.calls.is_empty());

    // self.iterations - method receiver field
    assert!(is_arg_unresolved(&result, 2));
    let expr = get_arg_expression(&result, 2);
    assert_eq!(expr, Some("self.iterations".to_string()));
}

// =============================================================================
// Chained Field Access
// =============================================================================

#[test]
fn test_rust_chained_field_two() {
    let source = r#"
fn test() {
    pbkdf2::derive(password, salt, app.config.iterations, 32);
}
"#;
    let result = scan_rust(source);
    assert!(!result.calls.is_empty());

    // app.config.iterations
    assert!(is_arg_unresolved(&result, 2));
    let expr = get_arg_expression(&result, 2);
    assert_eq!(expr, Some("app.config.iterations".to_string()));
}

#[test]
fn test_rust_chained_field_three() {
    let source = r#"
fn test() {
    pbkdf2::derive(password, salt, a.b.c.d, 32);
}
"#;
    let result = scan_rust(source);
    assert!(!result.calls.is_empty());

    // a.b.c.d
    assert!(is_arg_unresolved(&result, 2));
    let expr = get_arg_expression(&result, 2);
    assert_eq!(expr, Some("a.b.c.d".to_string()));
}

// =============================================================================
// Tuple Field Access
// =============================================================================

#[test]
fn test_rust_tuple_field() {
    let source = r#"
fn test() {
    let cfg = (10000, 32);
    pbkdf2::derive(password, salt, cfg.0, cfg.1);
}
"#;
    let result = scan_rust(source);
    assert!(!result.calls.is_empty());

    // Tuple field access resolves through the strategy chain:
    // 1. SelectorStrategy handles cfg.0
    // 2. Resolves cfg via IdentifierStrategy -> finds (10000, 32)
    // 3. CompositeStrategy resolves tuple to [10000, 32]
    // 4. SelectorStrategy extracts element 0 -> 10000
    assert!(is_arg_resolved(&result, 2));
    assert_eq!(get_first_arg_int(&result, 2), Some(10000));
    assert!(is_arg_resolved(&result, 3));
    assert_eq!(get_first_arg_int(&result, 3), Some(32));
}

// =============================================================================
// Field in Binary Expression
// =============================================================================

#[test]
fn test_rust_field_binary_add() {
    let source = r#"
const BASE: u32 = 100000;
fn test() {
    pbkdf2::derive(password, salt, config.extra + BASE, 32);
}
"#;
    let result = scan_rust(source);
    assert!(!result.calls.is_empty());

    // config.extra + BASE - contains field, should be unresolved
    assert!(is_arg_unresolved(&result, 2));
}

#[test]
fn test_rust_field_binary_mul() {
    let source = r#"
fn test() {
    pbkdf2::derive(password, salt, cfg.base * 2, 32);
}
"#;
    let result = scan_rust(source);
    assert!(!result.calls.is_empty());

    // cfg.base * 2
    assert!(is_arg_unresolved(&result, 2));
}

// =============================================================================
// Mixed Resolved and Unresolved
// =============================================================================

#[test]
fn test_rust_literal_with_field() {
    let source = r#"
fn test() {
    pbkdf2::derive(password, salt, 10000, cfg.key_length);
}
"#;
    let result = scan_rust(source);
    assert!(!result.calls.is_empty());

    // 10000 should resolve
    assert!(is_arg_resolved(&result, 2));
    assert_eq!(get_first_arg_int(&result, 2), Some(10000));

    // cfg.key_length should be unresolved
    assert!(is_arg_unresolved(&result, 3));
}

// =============================================================================
// Not Field Access (Control Cases)
// =============================================================================

#[test]
fn test_rust_not_field_literal() {
    let source = r#"
fn test() {
    pbkdf2::derive(password, salt, 10000, 32);
}
"#;
    let result = scan_rust(source);
    assert!(!result.calls.is_empty());

    assert!(is_arg_resolved(&result, 2));
    assert_eq!(get_first_arg_int(&result, 2), Some(10000));
}

#[test]
fn test_rust_not_field_local_var() {
    let source = r#"
fn test() {
    let iterations = 10000;
    pbkdf2::derive(password, salt, iterations, 32);
}
"#;
    let result = scan_rust(source);
    assert!(!result.calls.is_empty());

    // iterations is a local variable, should resolve
    assert!(is_arg_resolved(&result, 2));
    assert_eq!(get_first_arg_int(&result, 2), Some(10000));
}

#[test]
fn test_rust_not_field_const() {
    let source = r#"
const ITERATIONS: u32 = 100000;

fn test() {
    pbkdf2::derive(password, salt, ITERATIONS, 32);
}
"#;
    let result = scan_rust(source);
    assert!(!result.calls.is_empty());

    // ITERATIONS is a file-level constant, should resolve
    assert!(is_arg_resolved(&result, 2));
    assert_eq!(get_first_arg_int(&result, 2), Some(100000));
}

// =============================================================================
// Multiple Fields in Same Call
// =============================================================================

#[test]
fn test_rust_multiple_fields() {
    let source = r#"
fn test() {
    pbkdf2::derive(cfg.password, cfg.salt, cfg.iterations, cfg.key_len);
}
"#;
    let result = scan_rust(source);
    assert!(!result.calls.is_empty());

    // All arguments should be unresolved fields
    for i in 0..4 {
        assert!(
            is_arg_unresolved(&result, i),
            "Arg {} should be unresolved",
            i
        );
        let expr = get_arg_expression(&result, i);
        assert!(expr.is_some(), "Arg {} should have expression", i);
    }
}

// =============================================================================
// Reference Field Access
// =============================================================================

#[test]
fn test_rust_ref_field() {
    let source = r#"
fn test(cfg: &Config) {
    pbkdf2::derive(password, salt, cfg.iterations, 32);
}
"#;
    let result = scan_rust(source);
    assert!(!result.calls.is_empty());

    // cfg.iterations through reference
    assert!(is_arg_unresolved(&result, 2));
    let expr = get_arg_expression(&result, 2);
    assert_eq!(expr, Some("cfg.iterations".to_string()));
}
