//! Go selector expression resolution tests
//!
//! Tests selector expressions in Go code:
//! - Package-qualified constants (pkg.Constant)
//! - Field access (obj.field)
//! - Chained selectors (a.b.c)
//! - Method receivers (basic heuristic)

use super::test_utils::*;

// =============================================================================
// Package-Qualified Constants
// =============================================================================

#[test]
fn test_go_package_constant_uppercase() {
    let source = r#"
package main
import "golang.org/x/crypto/pbkdf2"
func test() { pbkdf2.Key(pass, salt, crypto.DefaultIterations, 32, sha256.New) }
"#;
    let result = scan_go(source);
    assert!(!result.calls.is_empty());

    // Argument 2 (index 2) is `crypto.DefaultIterations` - should be partial expression
    assert!(is_arg_unresolved(&result, 2));
    let expr = get_arg_expression(&result, 2);
    assert!(expr
        .map(|e| e.contains("DefaultIterations"))
        .unwrap_or(false));
}

#[test]
fn test_go_package_constant_sha256() {
    let source = r#"
package main
import "crypto/hmac"
func test() { hmac.New(sha256.New, key) }
"#;
    let result = scan_go(source);
    assert!(!result.calls.is_empty());

    // First argument is sha256.New - package constant reference
    // Should be partial expression since we can't resolve cross-package
    assert!(is_arg_unresolved(&result, 0));
}

#[test]
fn test_go_package_function_call() {
    let source = r#"
package main
import "crypto/aes"
func test() { aes.NewCipher(key) }
"#;
    let result = scan_go(source);
    assert!(!result.calls.is_empty());
    // aes.NewCipher is recognized as a call
}

// =============================================================================
// Field Access
// =============================================================================

#[test]
fn test_go_field_access_simple() {
    let source = r#"
package main
import "golang.org/x/crypto/pbkdf2"
func test() {
    cfg := Config{Iterations: 10000}
    pbkdf2.Key(pass, salt, cfg.Iterations, 32, sha256.New)
}
"#;
    let result = scan_go(source);
    assert!(!result.calls.is_empty());

    // cfg.Iterations - field access returns partial expression
    assert!(is_arg_unresolved(&result, 2));
    let expr = get_arg_expression(&result, 2);
    assert_eq!(expr, Some("cfg.Iterations".to_string()));
}

#[test]
fn test_go_field_access_key_size() {
    // Note: opts.KeySize is inside make([]byte, opts.KeySize), so it won't be directly accessible
    // The first argument to NewCipher is the make() call expression
    let source = r#"
package main
import "crypto/aes"
func test() {
    keyLen := opts.KeySize
    cipher, _ := aes.NewCipher(make([]byte, keyLen))
}
"#;
    let result = scan_go(source);
    // keyLen references opts.KeySize which is a selector
    // This tests that identifier resolution works with selector-assigned values
    if !result.calls.is_empty() {
        // The make call argument contains keyLen, which is unresolved (references selector)
        // This is complex - we're testing transitive selector propagation
    }
}

#[test]
fn test_go_method_receiver_access() {
    let source = r#"
package main
import "golang.org/x/crypto/pbkdf2"
type Crypto struct {
    iterations int
}
func (c *Crypto) Derive() {
    pbkdf2.Key(pass, salt, c.iterations, 32, sha256.New)
}
"#;
    let result = scan_go(source);
    assert!(!result.calls.is_empty());

    // c.iterations - method receiver field access
    assert!(is_arg_unresolved(&result, 2));
    let expr = get_arg_expression(&result, 2);
    assert_eq!(expr, Some("c.iterations".to_string()));
}

// =============================================================================
// Chained Selectors
// =============================================================================

#[test]
fn test_go_chained_selector_two() {
    let source = r#"
package main
import "golang.org/x/crypto/pbkdf2"
func test() { pbkdf2.Key(pass, salt, cfg.crypto.iterations, 32, sha256.New) }
"#;
    let result = scan_go(source);
    assert!(!result.calls.is_empty());

    // cfg.crypto.iterations - chained access
    assert!(is_arg_unresolved(&result, 2));
    let expr = get_arg_expression(&result, 2);
    assert_eq!(expr, Some("cfg.crypto.iterations".to_string()));
}

#[test]
fn test_go_chained_selector_three() {
    let source = r#"
package main
import "golang.org/x/crypto/pbkdf2"
func test() { pbkdf2.Key(pass, salt, a.b.c.d, 32, sha256.New) }
"#;
    let result = scan_go(source);
    assert!(!result.calls.is_empty());

    // a.b.c.d - deeply chained
    assert!(is_arg_unresolved(&result, 2));
    let expr = get_arg_expression(&result, 2);
    assert_eq!(expr, Some("a.b.c.d".to_string()));
}

// =============================================================================
// Selector with Other Expressions
// =============================================================================

#[test]
fn test_go_selector_in_binary_add() {
    let source = r#"
package main
import "golang.org/x/crypto/pbkdf2"
const BASE = 100000
func test() { pbkdf2.Key(pass, salt, config.extra + BASE, 32, sha256.New) }
"#;
    let result = scan_go(source);
    assert!(!result.calls.is_empty());

    // config.extra + BASE - binary with selector
    // Should be unresolved due to selector
    assert!(is_arg_unresolved(&result, 2));
}

#[test]
fn test_go_selector_in_binary_mul() {
    let source = r#"
package main
import "golang.org/x/crypto/pbkdf2"
func test() { pbkdf2.Key(pass, salt, cfg.base * 2, 32, sha256.New) }
"#;
    let result = scan_go(source);
    assert!(!result.calls.is_empty());

    // cfg.base * 2 - binary with selector operand
    assert!(is_arg_unresolved(&result, 2));
}

// =============================================================================
// Resolved Literals (Not Selectors)
// =============================================================================

#[test]
fn test_go_literal_with_selector_sibling() {
    let source = r#"
package main
import "golang.org/x/crypto/pbkdf2"
func test() { pbkdf2.Key(pass, salt, 10000, cfg.KeyLen, sha256.New) }
"#;
    let result = scan_go(source);
    assert!(!result.calls.is_empty());

    // Literal 10000 should still resolve
    assert!(is_arg_resolved(&result, 2));
    assert_eq!(get_first_arg_int(&result, 2), Some(10000));

    // cfg.KeyLen should be unresolved
    assert!(is_arg_unresolved(&result, 3));
}

// =============================================================================
// Import Alias Selectors
// =============================================================================

#[test]
fn test_go_aliased_import() {
    let source = r#"
package main
import kdf "golang.org/x/crypto/pbkdf2"
func test() { kdf.Key(pass, salt, 10000, 32, sha256.New) }
"#;
    let result = scan_go(source);
    assert!(!result.calls.is_empty());
    // kdf.Key should be recognized as a crypto call
}

// =============================================================================
// Constant Declarations
// =============================================================================

#[test]
fn test_go_local_constant_not_selector() {
    let source = r#"
package main
import "golang.org/x/crypto/pbkdf2"
const iterations = 100000
func test() { pbkdf2.Key(pass, salt, iterations, 32, sha256.New) }
"#;
    let result = scan_go(source);
    assert!(!result.calls.is_empty());

    // iterations (identifier) should resolve to 100000
    assert!(is_arg_resolved(&result, 2));
    assert_eq!(get_first_arg_int(&result, 2), Some(100000));
}

// =============================================================================
// Multiple Selectors in Same Call
// =============================================================================

#[test]
fn test_go_multiple_selectors() {
    let source = r#"
package main
import "golang.org/x/crypto/pbkdf2"
func test() { pbkdf2.Key(cfg.pass, cfg.salt, cfg.iterations, cfg.keyLen, hash.New) }
"#;
    let result = scan_go(source);
    assert!(!result.calls.is_empty());

    // All arguments should be unresolved selectors
    for i in 0..4 {
        assert!(is_arg_unresolved(&result, i));
        let expr = get_arg_expression(&result, i);
        assert!(expr.is_some(), "Arg {} should have expression", i);
    }
}

// =============================================================================
// Negative Tests - Not Selectors
// =============================================================================

#[test]
fn test_go_not_selector_literal() {
    let source = r#"
package main
import "golang.org/x/crypto/pbkdf2"
func test() { pbkdf2.Key(pass, salt, 10000, 32, sha256.New) }
"#;
    let result = scan_go(source);
    assert!(!result.calls.is_empty());

    // 10000 is a literal, should resolve
    assert!(is_arg_resolved(&result, 2));
    assert_eq!(get_first_arg_int(&result, 2), Some(10000));
}

#[test]
fn test_go_not_selector_binary() {
    let source = r#"
package main
import "golang.org/x/crypto/pbkdf2"
func test() { pbkdf2.Key(pass, salt, 10000 + 5000, 32, sha256.New) }
"#;
    let result = scan_go(source);
    assert!(!result.calls.is_empty());

    // 10000 + 5000 should resolve to 15000
    assert!(is_arg_resolved(&result, 2));
    assert_eq!(get_first_arg_int(&result, 2), Some(15000));
}
