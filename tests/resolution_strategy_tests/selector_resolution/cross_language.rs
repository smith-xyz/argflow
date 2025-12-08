//! Cross-language selector resolution consistency tests
//!
//! These tests verify consistent behavior across all supported languages for common
//! selector/attribute/property access patterns.

use super::test_utils::*;

// =============================================================================
// Simple Field Access Consistency
// =============================================================================

#[test]
fn test_cross_lang_field_access_unresolved() {
    // Go
    let go_source = r#"
package main
import "golang.org/x/crypto/pbkdf2"
func test() { pbkdf2.Key(p, s, cfg.Iterations, 32, h) }
"#;
    let go_result = scan_go(go_source);
    assert!(!go_result.calls.is_empty());
    assert!(is_arg_unresolved(&go_result, 2));
    assert_eq!(
        get_arg_expression(&go_result, 2),
        Some("cfg.Iterations".to_string())
    );

    // Python
    let py_source = r#"
from hashlib import pbkdf2_hmac
result = pbkdf2_hmac('sha256', p, s, cfg.iterations)
"#;
    let py_result = scan_python(py_source);
    assert!(!py_result.calls.is_empty());
    assert!(is_arg_unresolved(&py_result, 3));
    assert_eq!(
        get_arg_expression(&py_result, 3),
        Some("cfg.iterations".to_string())
    );

    // Rust
    let rs_source = r#"
fn test() {
    pbkdf2::derive(p, s, cfg.iterations, 32);
}
"#;
    let rs_result = scan_rust(rs_source);
    assert!(!rs_result.calls.is_empty());
    assert!(is_arg_unresolved(&rs_result, 2));
    assert_eq!(
        get_arg_expression(&rs_result, 2),
        Some("cfg.iterations".to_string())
    );

    // JavaScript
    let js_source = r#"
crypto.pbkdf2Sync(p, s, cfg.iterations, 32, 'sha256');
"#;
    let js_result = scan_javascript(js_source);
    assert!(!js_result.calls.is_empty());
    assert!(is_arg_unresolved(&js_result, 2));
    assert_eq!(
        get_arg_expression(&js_result, 2),
        Some("cfg.iterations".to_string())
    );
}

// =============================================================================
// Chained Selector Consistency
// =============================================================================

#[test]
fn test_cross_lang_chained_selector() {
    // Go
    let go_source = r#"
package main
import "golang.org/x/crypto/pbkdf2"
func test() { pbkdf2.Key(p, s, app.cfg.iterations, 32, h) }
"#;
    let go_result = scan_go(go_source);
    assert!(!go_result.calls.is_empty());
    assert!(is_arg_unresolved(&go_result, 2));
    assert_eq!(
        get_arg_expression(&go_result, 2),
        Some("app.cfg.iterations".to_string())
    );

    // Python
    let py_source = r#"
from hashlib import pbkdf2_hmac
result = pbkdf2_hmac('sha256', p, s, app.cfg.iterations)
"#;
    let py_result = scan_python(py_source);
    assert!(!py_result.calls.is_empty());
    assert!(is_arg_unresolved(&py_result, 3));
    assert_eq!(
        get_arg_expression(&py_result, 3),
        Some("app.cfg.iterations".to_string())
    );

    // Rust
    let rs_source = r#"
fn test() {
    pbkdf2::derive(p, s, app.cfg.iterations, 32);
}
"#;
    let rs_result = scan_rust(rs_source);
    assert!(!rs_result.calls.is_empty());
    assert!(is_arg_unresolved(&rs_result, 2));
    assert_eq!(
        get_arg_expression(&rs_result, 2),
        Some("app.cfg.iterations".to_string())
    );

    // JavaScript
    let js_source = r#"
crypto.pbkdf2Sync(p, s, app.cfg.iterations, 32, 'sha256');
"#;
    let js_result = scan_javascript(js_source);
    assert!(!js_result.calls.is_empty());
    assert!(is_arg_unresolved(&js_result, 2));
    assert_eq!(
        get_arg_expression(&js_result, 2),
        Some("app.cfg.iterations".to_string())
    );
}

// =============================================================================
// Self/This Receiver Consistency
// =============================================================================

#[test]
fn test_cross_lang_self_receiver() {
    // Go (method receiver is not `self`, use `c`)
    let go_source = r#"
package main
import "golang.org/x/crypto/pbkdf2"
type Crypto struct { iterations int }
func (c *Crypto) derive() { pbkdf2.Key(p, s, c.iterations, 32, h) }
"#;
    let go_result = scan_go(go_source);
    assert!(!go_result.calls.is_empty());
    assert!(is_arg_unresolved(&go_result, 2));
    assert_eq!(
        get_arg_expression(&go_result, 2),
        Some("c.iterations".to_string())
    );

    // Python (self)
    let py_source = r#"
from hashlib import pbkdf2_hmac
class Crypto:
    def derive(self):
        return pbkdf2_hmac('sha256', p, s, self.iterations)
"#;
    let py_result = scan_python(py_source);
    assert!(!py_result.calls.is_empty());
    assert!(is_arg_unresolved(&py_result, 3));
    assert_eq!(
        get_arg_expression(&py_result, 3),
        Some("self.iterations".to_string())
    );

    // Rust (self)
    let rs_source = r#"
impl Crypto {
    fn derive(&self) {
        pbkdf2::derive(p, s, self.iterations, 32);
    }
}
"#;
    let rs_result = scan_rust(rs_source);
    assert!(!rs_result.calls.is_empty());
    assert!(is_arg_unresolved(&rs_result, 2));
    assert_eq!(
        get_arg_expression(&rs_result, 2),
        Some("self.iterations".to_string())
    );

    // JavaScript (this)
    let js_source = r#"
class Crypto {
    derive() {
        return crypto.pbkdf2Sync(p, s, this.iterations, 32, 'sha256');
    }
}
"#;
    let js_result = scan_javascript(js_source);
    assert!(!js_result.calls.is_empty());
    assert!(is_arg_unresolved(&js_result, 2));
    assert_eq!(
        get_arg_expression(&js_result, 2),
        Some("this.iterations".to_string())
    );
}

// =============================================================================
// Mixed Resolved/Unresolved Consistency
// =============================================================================

#[test]
fn test_cross_lang_mixed_resolution() {
    // Go: literal resolves, selector doesn't
    let go_source = r#"
package main
import "golang.org/x/crypto/pbkdf2"
func test() { pbkdf2.Key(p, s, 10000, cfg.KeyLen, h) }
"#;
    let go_result = scan_go(go_source);
    assert!(!go_result.calls.is_empty());
    assert!(is_arg_resolved(&go_result, 2));
    assert_eq!(get_first_arg_int(&go_result, 2), Some(10000));
    assert!(is_arg_unresolved(&go_result, 3));

    // Python: literal resolves, attribute doesn't
    let py_source = r#"
from hashlib import pbkdf2_hmac
result = pbkdf2_hmac('sha256', p, s, 10000, cfg.key_len)
"#;
    let py_result = scan_python(py_source);
    assert!(!py_result.calls.is_empty());
    assert!(is_arg_resolved(&py_result, 3));
    assert_eq!(get_first_arg_int(&py_result, 3), Some(10000));
    assert!(is_arg_unresolved(&py_result, 4));

    // Rust: literal resolves, field doesn't
    let rs_source = r#"
fn test() {
    pbkdf2::derive(p, s, 10000, cfg.key_len);
}
"#;
    let rs_result = scan_rust(rs_source);
    assert!(!rs_result.calls.is_empty());
    assert!(is_arg_resolved(&rs_result, 2));
    assert_eq!(get_first_arg_int(&rs_result, 2), Some(10000));
    assert!(is_arg_unresolved(&rs_result, 3));

    // JavaScript: literal resolves, property doesn't
    let js_source = r#"
crypto.pbkdf2Sync(p, s, 10000, cfg.keyLen, 'sha256');
"#;
    let js_result = scan_javascript(js_source);
    assert!(!js_result.calls.is_empty());
    assert!(is_arg_resolved(&js_result, 2));
    assert_eq!(get_first_arg_int(&js_result, 2), Some(10000));
    assert!(is_arg_unresolved(&js_result, 3));
}

// =============================================================================
// Binary with Selector Consistency
// =============================================================================

#[test]
fn test_cross_lang_binary_with_selector() {
    // Go
    let go_source = r#"
package main
import "golang.org/x/crypto/pbkdf2"
const BASE = 100000
func test() { pbkdf2.Key(p, s, cfg.extra + BASE, 32, h) }
"#;
    let go_result = scan_go(go_source);
    assert!(!go_result.calls.is_empty());
    // Binary expression containing selector should be unresolved
    assert!(is_arg_unresolved(&go_result, 2));

    // Python
    let py_source = r#"
from hashlib import pbkdf2_hmac
BASE = 100000
result = pbkdf2_hmac('sha256', p, s, cfg.extra + BASE)
"#;
    let py_result = scan_python(py_source);
    assert!(!py_result.calls.is_empty());
    assert!(is_arg_unresolved(&py_result, 3));

    // Rust
    let rs_source = r#"
const BASE: u32 = 100000;
fn test() {
    pbkdf2::derive(p, s, cfg.extra + BASE, 32);
}
"#;
    let rs_result = scan_rust(rs_source);
    assert!(!rs_result.calls.is_empty());
    assert!(is_arg_unresolved(&rs_result, 2));

    // JavaScript
    let js_source = r#"
const BASE = 100000;
crypto.pbkdf2Sync(p, s, cfg.extra + BASE, 32, 'sha256');
"#;
    let js_result = scan_javascript(js_source);
    assert!(!js_result.calls.is_empty());
    assert!(is_arg_unresolved(&js_result, 2));
}

// =============================================================================
// Local Variable Should Still Resolve
// =============================================================================

#[test]
fn test_cross_lang_local_var_resolves() {
    // Go
    let go_source = r#"
package main
import "golang.org/x/crypto/pbkdf2"
func test() {
    iterations := 10000
    pbkdf2.Key(p, s, iterations, 32, h)
}
"#;
    let go_result = scan_go(go_source);
    assert!(!go_result.calls.is_empty());
    assert!(is_arg_resolved(&go_result, 2));
    assert_eq!(get_first_arg_int(&go_result, 2), Some(10000));

    // Python
    let py_source = r#"
from hashlib import pbkdf2_hmac
def test():
    iterations = 10000
    return pbkdf2_hmac('sha256', p, s, iterations)
"#;
    let py_result = scan_python(py_source);
    assert!(!py_result.calls.is_empty());
    assert!(is_arg_resolved(&py_result, 3));
    assert_eq!(get_first_arg_int(&py_result, 3), Some(10000));

    // Rust
    let rs_source = r#"
fn test() {
    let iterations = 10000;
    pbkdf2::derive(p, s, iterations, 32);
}
"#;
    let rs_result = scan_rust(rs_source);
    assert!(!rs_result.calls.is_empty());
    assert!(is_arg_resolved(&rs_result, 2));
    assert_eq!(get_first_arg_int(&rs_result, 2), Some(10000));

    // JavaScript
    let js_source = r#"
function test() {
    const iterations = 10000;
    crypto.pbkdf2Sync(p, s, iterations, 32, 'sha256');
}
"#;
    let js_result = scan_javascript(js_source);
    assert!(!js_result.calls.is_empty());
    assert!(is_arg_resolved(&js_result, 2));
    assert_eq!(get_first_arg_int(&js_result, 2), Some(10000));
}

// =============================================================================
// Deeply Nested Selector (3+ levels)
// =============================================================================

#[test]
fn test_cross_lang_deeply_nested() {
    // Go
    let go_source = r#"
package main
import "golang.org/x/crypto/pbkdf2"
func test() { pbkdf2.Key(p, s, a.b.c.d, 32, h) }
"#;
    let go_result = scan_go(go_source);
    assert!(!go_result.calls.is_empty());
    assert!(is_arg_unresolved(&go_result, 2));
    assert_eq!(
        get_arg_expression(&go_result, 2),
        Some("a.b.c.d".to_string())
    );

    // Python
    let py_source = r#"
from hashlib import pbkdf2_hmac
result = pbkdf2_hmac('sha256', p, s, a.b.c.d)
"#;
    let py_result = scan_python(py_source);
    assert!(!py_result.calls.is_empty());
    assert!(is_arg_unresolved(&py_result, 3));
    assert_eq!(
        get_arg_expression(&py_result, 3),
        Some("a.b.c.d".to_string())
    );

    // Rust
    let rs_source = r#"
fn test() {
    pbkdf2::derive(p, s, a.b.c.d, 32);
}
"#;
    let rs_result = scan_rust(rs_source);
    assert!(!rs_result.calls.is_empty());
    assert!(is_arg_unresolved(&rs_result, 2));
    assert_eq!(
        get_arg_expression(&rs_result, 2),
        Some("a.b.c.d".to_string())
    );

    // JavaScript
    let js_source = r#"
crypto.pbkdf2Sync(p, s, a.b.c.d, 32, 'sha256');
"#;
    let js_result = scan_javascript(js_source);
    assert!(!js_result.calls.is_empty());
    assert!(is_arg_unresolved(&js_result, 2));
    assert_eq!(
        get_arg_expression(&js_result, 2),
        Some("a.b.c.d".to_string())
    );
}
