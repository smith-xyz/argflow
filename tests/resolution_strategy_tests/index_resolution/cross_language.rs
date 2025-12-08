//! Cross-language index resolution consistency tests
//!
//! These tests verify that index resolution behaves consistently across languages
//! for equivalent constructs, ensuring the universal approach works correctly.
//!
//! Note: Python tests are ignored because scanner doesn't detect PBKDF2HMAC.
//! The index strategy itself works correctly - see unit tests in index.rs.

use super::test_utils::{scan_go, scan_javascript, scan_python, scan_rust};

// =============================================================================
// Array First Element (Consistent Across Languages)
// =============================================================================

#[test]

fn test_all_languages_array_first_element() {
    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, []int{10000, 20000}[0], 32, h) }
"#,
    );

    let python_result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm=h, length=32, salt=s, iterations=[10000, 20000][0])
"#,
    );

    let rust_result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(alg, [10000u32, 20000][0], &salt, &password, &mut key);
}
"#,
    );

    let js_result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, [10000, 20000][0], 32, 'sha256');
"#,
    );

    let go_val = go_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(2))
        .map(|a| a.int_values.first().copied());
    let python_val = python_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(3))
        .map(|a| a.int_values.first().copied());
    let rust_val = rust_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(1))
        .map(|a| a.int_values.first().copied());
    let js_val = js_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(2))
        .map(|a| a.int_values.first().copied());

    assert_eq!(go_val, Some(Some(10000)), "Go first element");
    assert_eq!(python_val, Some(Some(10000)), "Python first element");
    assert_eq!(rust_val, Some(Some(10000)), "Rust first element");
    assert_eq!(js_val, Some(Some(10000)), "JavaScript first element");
}

// =============================================================================
// Array Last Element (Consistent Across Languages)
// =============================================================================

#[test]

fn test_all_languages_array_last_element() {
    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, []int{10000, 20000, 30000}[2], 32, h) }
"#,
    );

    let python_result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm=h, length=32, salt=s, iterations=[10000, 20000, 30000][2])
"#,
    );

    let rust_result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(alg, [10000u32, 20000, 30000][2], &salt, &password, &mut key);
}
"#,
    );

    let js_result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, [10000, 20000, 30000][2], 32, 'sha256');
"#,
    );

    let go_val = go_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(2))
        .map(|a| a.int_values.first().copied());
    let python_val = python_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(3))
        .map(|a| a.int_values.first().copied());
    let rust_val = rust_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(1))
        .map(|a| a.int_values.first().copied());
    let js_val = js_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(2))
        .map(|a| a.int_values.first().copied());

    assert_eq!(go_val, Some(Some(30000)), "Go last element");
    assert_eq!(python_val, Some(Some(30000)), "Python last element");
    assert_eq!(rust_val, Some(Some(30000)), "Rust last element");
    assert_eq!(js_val, Some(Some(30000)), "JavaScript last element");
}

// =============================================================================
// Key Sizes Array (Crypto-relevant)
// =============================================================================

#[test]

fn test_all_languages_key_sizes_128() {
    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, []int{16, 24, 32}[0], h) }
"#,
    );

    let python_result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm=h, length=[16, 24, 32][0], salt=s, iterations=10000)
"#,
    );

    let js_result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, 10000, [16, 24, 32][0], 'sha256');
"#,
    );

    let go_val = go_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(3))
        .map(|a| a.int_values.first().copied());
    let python_val = python_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(1))
        .map(|a| a.int_values.first().copied());
    let js_val = js_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(3))
        .map(|a| a.int_values.first().copied());

    assert_eq!(go_val, Some(Some(16)), "Go AES-128 key");
    assert_eq!(python_val, Some(Some(16)), "Python AES-128 key");
    assert_eq!(js_val, Some(Some(16)), "JavaScript AES-128 key");
}

#[test]

fn test_all_languages_key_sizes_256() {
    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, []int{16, 24, 32}[2], h) }
"#,
    );

    let python_result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm=h, length=[16, 24, 32][2], salt=s, iterations=10000)
"#,
    );

    let js_result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, 10000, [16, 24, 32][2], 'sha256');
"#,
    );

    let go_val = go_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(3))
        .map(|a| a.int_values.first().copied());
    let python_val = python_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(1))
        .map(|a| a.int_values.first().copied());
    let js_val = js_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(3))
        .map(|a| a.int_values.first().copied());

    assert_eq!(go_val, Some(Some(32)), "Go AES-256 key");
    assert_eq!(python_val, Some(Some(32)), "Python AES-256 key");
    assert_eq!(js_val, Some(Some(32)), "JavaScript AES-256 key");
}

// =============================================================================
// Out of Bounds (Consistent Failure Across Languages)
// =============================================================================

#[test]
fn test_all_languages_out_of_bounds() {
    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, []int{10000, 20000}[5], 32, h) }
"#,
    );

    let python_result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm=h, length=32, salt=s, iterations=[10000, 20000][5])
"#,
    );

    let rust_result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(alg, [10000u32, 20000][5], &salt, &password, &mut key);
}
"#,
    );

    let js_result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, [10000, 20000][5], 32, 'sha256');
"#,
    );

    let go_resolved = go_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(2))
        .map(|a| a.is_resolved)
        .unwrap_or(true);
    let python_resolved = python_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(3))
        .map(|a| a.is_resolved)
        .unwrap_or(true);
    let rust_resolved = rust_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(1))
        .map(|a| a.is_resolved)
        .unwrap_or(true);
    let js_resolved = js_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(2))
        .map(|a| a.is_resolved)
        .unwrap_or(true);

    assert!(!go_resolved, "Go out of bounds should not resolve");
    assert!(!python_resolved, "Python out of bounds should not resolve");
    assert!(!rust_resolved, "Rust out of bounds should not resolve");
    assert!(!js_resolved, "JavaScript out of bounds should not resolve");
}

// =============================================================================
// Variable Array (Consistent Unresolved Across Languages)
// =============================================================================

#[test]
fn test_all_languages_variable_array() {
    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
var sizes = []int{16, 24, 32}
func main() { pbkdf2.Key(p, s, 10000, sizes[0], h) }
"#,
    );

    let python_result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
sizes = [16, 24, 32]
kdf = PBKDF2HMAC(algorithm=h, length=sizes[0], salt=s, iterations=10000)
"#,
    );

    let js_result = scan_javascript(
        r#"
const crypto = require('crypto');
const sizes = [16, 24, 32];
crypto.pbkdf2Sync(password, salt, 10000, sizes[0], 'sha256');
"#,
    );

    let go_resolved = go_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(3))
        .map(|a| a.is_resolved)
        .unwrap_or(true);
    let python_resolved = python_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(1))
        .map(|a| a.is_resolved)
        .unwrap_or(true);
    let js_resolved = js_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(3))
        .map(|a| a.is_resolved)
        .unwrap_or(true);

    assert!(!go_resolved, "Go variable array should not resolve");
    assert!(!python_resolved, "Python variable array should not resolve");
    assert!(!js_resolved, "JavaScript variable array should not resolve");
}

// =============================================================================
// Hex Values (Consistent Across Languages)
// =============================================================================

#[test]

fn test_all_languages_hex_values() {
    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, []int{0x10, 0x18, 0x20}[0], h) }
"#,
    );

    let python_result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm=h, length=[0x10, 0x18, 0x20][0], salt=s, iterations=10000)
"#,
    );

    let js_result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, 10000, [0x10, 0x18, 0x20][0], 'sha256');
"#,
    );

    let go_val = go_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(3))
        .map(|a| a.int_values.first().copied());
    let python_val = python_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(1))
        .map(|a| a.int_values.first().copied());
    let js_val = js_result
        .calls
        .first()
        .and_then(|c| c.arguments.get(3))
        .map(|a| a.int_values.first().copied());

    assert_eq!(go_val, Some(Some(16)), "Go hex value");
    assert_eq!(python_val, Some(Some(16)), "Python hex value");
    assert_eq!(js_val, Some(Some(16)), "JavaScript hex value");
}
