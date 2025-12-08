//! Cross-language binary resolution consistency tests
//!
//! These tests ensure that binary expression resolution behaves consistently
//! across all supported languages.

use super::test_utils::{get_first_arg_int, scan_go, scan_javascript, scan_python, scan_rust};

// =============================================================================
// Arithmetic Consistency Tests
// =============================================================================

#[test]
fn test_addition_consistency() {
    let expected = Some(150);

    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 100 + 50, 32, h) }
"#,
    );

    let py_result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(100 + 50, 32)
"#,
    );

    let js_result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 100 + 50, 32, 'sha256');
"#,
    );

    let rs_result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 100 + 50, salt, password, &mut key); }
"#,
    );

    assert_eq!(get_first_arg_int(&go_result, 2), expected, "Go");
    assert_eq!(get_first_arg_int(&py_result, 0), expected, "Python");
    assert_eq!(get_first_arg_int(&js_result, 2), expected, "JavaScript");
    assert_eq!(get_first_arg_int(&rs_result, 1), expected, "Rust");
}

#[test]
fn test_subtraction_consistency() {
    let expected = Some(70);

    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 100 - 30, 32, h) }
"#,
    );

    let py_result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(100 - 30, 32)
"#,
    );

    let js_result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 100 - 30, 32, 'sha256');
"#,
    );

    let rs_result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 100 - 30, salt, password, &mut key); }
"#,
    );

    assert_eq!(get_first_arg_int(&go_result, 2), expected, "Go");
    assert_eq!(get_first_arg_int(&py_result, 0), expected, "Python");
    assert_eq!(get_first_arg_int(&js_result, 2), expected, "JavaScript");
    assert_eq!(get_first_arg_int(&rs_result, 1), expected, "Rust");
}

#[test]
fn test_multiplication_consistency() {
    let expected = Some(256);

    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 32 * 8, h) }
"#,
    );

    let py_result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(10000, 32 * 8)
"#,
    );

    let js_result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 10000, 32 * 8, 'sha256');
"#,
    );

    assert_eq!(get_first_arg_int(&go_result, 3), expected, "Go");
    assert_eq!(get_first_arg_int(&py_result, 1), expected, "Python");
    assert_eq!(get_first_arg_int(&js_result, 3), expected, "JavaScript");
}

// =============================================================================
// Bitwise Shift Consistency Tests
// =============================================================================

#[test]
fn test_shift_left_consistency() {
    let expected = Some(32);

    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 1 << 5, h) }
"#,
    );

    let py_result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(10000, 1 << 5)
"#,
    );

    let js_result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 10000, 1 << 5, 'sha256');
"#,
    );

    let rs_result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 1 << 5, salt, password, &mut key); }
"#,
    );

    assert_eq!(get_first_arg_int(&go_result, 3), expected, "Go");
    assert_eq!(get_first_arg_int(&py_result, 1), expected, "Python");
    assert_eq!(get_first_arg_int(&js_result, 3), expected, "JavaScript");
    assert_eq!(get_first_arg_int(&rs_result, 1), expected, "Rust");
}

#[test]
fn test_shift_right_consistency() {
    let expected = Some(32);

    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 256 >> 3, h) }
"#,
    );

    let py_result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(10000, 256 >> 3)
"#,
    );

    let js_result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 10000, 256 >> 3, 'sha256');
"#,
    );

    let rs_result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 256 >> 3, salt, password, &mut key); }
"#,
    );

    assert_eq!(get_first_arg_int(&go_result, 3), expected, "Go");
    assert_eq!(get_first_arg_int(&py_result, 1), expected, "Python");
    assert_eq!(get_first_arg_int(&js_result, 3), expected, "JavaScript");
    assert_eq!(get_first_arg_int(&rs_result, 1), expected, "Rust");
}

// =============================================================================
// Bitwise Operation Consistency Tests
// =============================================================================

#[test]
fn test_bitwise_and_consistency() {
    let expected = Some(15);

    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 0xFF & 0x0F, h) }
"#,
    );

    let py_result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(10000, 0xFF & 0x0F)
"#,
    );

    let js_result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 10000, 0xFF & 0x0F, 'sha256');
"#,
    );

    let rs_result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 0xFF & 0x0F, salt, password, &mut key); }
"#,
    );

    assert_eq!(get_first_arg_int(&go_result, 3), expected, "Go");
    assert_eq!(get_first_arg_int(&py_result, 1), expected, "Python");
    assert_eq!(get_first_arg_int(&js_result, 3), expected, "JavaScript");
    assert_eq!(get_first_arg_int(&rs_result, 1), expected, "Rust");
}

#[test]
fn test_bitwise_or_consistency() {
    let expected = Some(255);

    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 0xF0 | 0x0F, h) }
"#,
    );

    let py_result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(10000, 0xF0 | 0x0F)
"#,
    );

    let js_result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 10000, 0xF0 | 0x0F, 'sha256');
"#,
    );

    let rs_result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 0xF0 | 0x0F, salt, password, &mut key); }
"#,
    );

    assert_eq!(get_first_arg_int(&go_result, 3), expected, "Go");
    assert_eq!(get_first_arg_int(&py_result, 1), expected, "Python");
    assert_eq!(get_first_arg_int(&js_result, 3), expected, "JavaScript");
    assert_eq!(get_first_arg_int(&rs_result, 1), expected, "Rust");
}

// =============================================================================
// Crypto-Relevant Consistency Tests
// =============================================================================

#[test]
fn test_pbkdf2_iterations_calculation_consistency() {
    let expected = Some(110000);

    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 100000 + 10000, 32, h) }
"#,
    );

    let py_result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(100000 + 10000, 32)
"#,
    );

    let js_result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 100000 + 10000, 32, 'sha256');
"#,
    );

    let rs_result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 100000 + 10000, salt, password, &mut key); }
"#,
    );

    assert_eq!(get_first_arg_int(&go_result, 2), expected, "Go");
    assert_eq!(get_first_arg_int(&py_result, 0), expected, "Python");
    assert_eq!(get_first_arg_int(&js_result, 2), expected, "JavaScript");
    assert_eq!(get_first_arg_int(&rs_result, 1), expected, "Rust");
}

// =============================================================================
// Nested Expression Consistency Tests
// =============================================================================

#[test]
fn test_nested_addition_consistency() {
    let expected = Some(60);

    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10 + 20 + 30, 32, h) }
"#,
    );

    let py_result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(10 + 20 + 30, 32)
"#,
    );

    let js_result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 10 + 20 + 30, 32, 'sha256');
"#,
    );

    let rs_result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 10 + 20 + 30, salt, password, &mut key); }
"#,
    );

    assert_eq!(get_first_arg_int(&go_result, 2), expected, "Go");
    assert_eq!(get_first_arg_int(&py_result, 0), expected, "Python");
    assert_eq!(get_first_arg_int(&js_result, 2), expected, "JavaScript");
    assert_eq!(get_first_arg_int(&rs_result, 1), expected, "Rust");
}

#[test]
fn test_parenthesized_expression_consistency() {
    let expected = Some(300);

    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, (100 + 50) * 2, 32, h) }
"#,
    );

    let py_result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC((100 + 50) * 2, 32)
"#,
    );

    let js_result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, (100 + 50) * 2, 32, 'sha256');
"#,
    );

    let rs_result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, (100 + 50) * 2, salt, password, &mut key); }
"#,
    );

    assert_eq!(get_first_arg_int(&go_result, 2), expected, "Go");
    assert_eq!(get_first_arg_int(&py_result, 0), expected, "Python");
    assert_eq!(get_first_arg_int(&js_result, 2), expected, "JavaScript");
    assert_eq!(get_first_arg_int(&rs_result, 1), expected, "Rust");
}

// =============================================================================
// With Unary Operand Consistency Tests
// =============================================================================

#[test]
fn test_with_unary_operand_consistency() {
    let expected = Some(50);

    let go_result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 100 + -50, 32, h) }
"#,
    );

    let py_result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
PBKDF2HMAC(100 + -50, 32)
"#,
    );

    let js_result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(p, s, 100 + -50, 32, 'sha256');
"#,
    );

    let rs_result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() { pbkdf2::derive(alg, 100 + -50, salt, password, &mut key); }
"#,
    );

    assert_eq!(get_first_arg_int(&go_result, 2), expected, "Go");
    assert_eq!(get_first_arg_int(&py_result, 0), expected, "Python");
    assert_eq!(get_first_arg_int(&js_result, 2), expected, "JavaScript");
    assert_eq!(get_first_arg_int(&rs_result, 1), expected, "Rust");
}
