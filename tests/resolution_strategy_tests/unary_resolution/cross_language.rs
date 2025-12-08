//! Cross-language unary resolution consistency tests

use super::test_utils::{
    get_first_arg_int, is_arg_unresolved, scan_go, scan_javascript, scan_python, scan_rust,
};

// =============================================================================
// Negation Consistency
// =============================================================================

#[test]
fn test_negative_100000_all_languages() {
    let go = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, -100000, 32, h) }
"#,
    );

    let python = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm, length, salt, -100000)
"#,
    );

    let rust = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(algorithm, -100000, salt, password, &mut out);
}
"#,
    );

    let js = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, -100000, 32, 'sha256');
"#,
    );

    assert_eq!(get_first_arg_int(&go, 2), Some(-100000), "Go: -100000");
    assert_eq!(
        get_first_arg_int(&python, 3),
        Some(-100000),
        "Python: -100000"
    );
    assert_eq!(get_first_arg_int(&rust, 1), Some(-100000), "Rust: -100000");
    assert_eq!(
        get_first_arg_int(&js, 2),
        Some(-100000),
        "JavaScript: -100000"
    );
}

// =============================================================================
// Bitwise NOT Consistency
// =============================================================================

#[test]
fn test_bitwise_not_255_all_languages() {
    // Go uses ^ for bitwise NOT
    let go = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, ^255, 32, h) }
"#,
    );

    // Python uses ~
    let python = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm, length, salt, ~255)
"#,
    );

    // JavaScript uses ~
    let js = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, ~255, 32, 'sha256');
"#,
    );

    // All should resolve to -256 (two's complement of 255)
    assert_eq!(get_first_arg_int(&go, 2), Some(-256), "Go: ^255 = -256");
    assert_eq!(
        get_first_arg_int(&python, 3),
        Some(-256),
        "Python: ~255 = -256"
    );
    assert_eq!(
        get_first_arg_int(&js, 2),
        Some(-256),
        "JavaScript: ~255 = -256"
    );
}

// =============================================================================
// Logical NOT Consistency
// =============================================================================

#[test]
fn test_logical_not_true_all_languages() {
    let go = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 32, !true) }
"#,
    );

    let python = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm, length, salt, not True)
"#,
    );

    let rust = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(algorithm, 100000, salt, !true, &mut out);
}
"#,
    );

    let js = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, 100000, 32, !true);
"#,
    );

    // All should resolve to 0 (false)
    assert_eq!(get_first_arg_int(&go, 4), Some(0), "Go: !true = 0");
    assert_eq!(
        get_first_arg_int(&python, 3),
        Some(0),
        "Python: not True = 0"
    );
    assert_eq!(get_first_arg_int(&rust, 3), Some(0), "Rust: !true = 0");
    assert_eq!(get_first_arg_int(&js, 4), Some(0), "JavaScript: !true = 0");
}

#[test]
fn test_logical_not_false_all_languages() {
    let go = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 32, !false) }
"#,
    );

    let python = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm, length, salt, not False)
"#,
    );

    let rust = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(algorithm, 100000, salt, !false, &mut out);
}
"#,
    );

    let js = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, 100000, 32, !false);
"#,
    );

    // All should resolve to 1 (true)
    assert_eq!(get_first_arg_int(&go, 4), Some(1), "Go: !false = 1");
    assert_eq!(
        get_first_arg_int(&python, 3),
        Some(1),
        "Python: not False = 1"
    );
    assert_eq!(get_first_arg_int(&rust, 3), Some(1), "Rust: !false = 1");
    assert_eq!(get_first_arg_int(&js, 4), Some(1), "JavaScript: !false = 1");
}

// =============================================================================
// Unresolved Variable Consistency
// =============================================================================

#[test]
fn test_negative_variable_all_languages() {
    let go = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
var iterations = 100000
func main() { pbkdf2.Key(p, s, -iterations, 32, h) }
"#,
    );

    let python = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
iterations = 100000
kdf = PBKDF2HMAC(algorithm, length, salt, -iterations)
"#,
    );

    let rust = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    let iterations = 100000;
    pbkdf2::derive(algorithm, -iterations, salt, password, &mut out);
}
"#,
    );

    let js = scan_javascript(
        r#"
const crypto = require('crypto');
const iterations = 100000;
crypto.pbkdf2Sync(password, salt, -iterations, 32, 'sha256');
"#,
    );

    // All should be unresolved (needs identifier resolution)
    assert!(
        is_arg_unresolved(&go, 2),
        "Go: -iterations needs identifier resolution"
    );
    assert!(
        is_arg_unresolved(&python, 3),
        "Python: -iterations needs identifier resolution"
    );
    assert!(
        is_arg_unresolved(&rust, 1),
        "Rust: -iterations needs identifier resolution"
    );
    assert!(
        is_arg_unresolved(&js, 2),
        "JavaScript: -iterations needs identifier resolution"
    );
}

// =============================================================================
// Nested Unary Consistency
// =============================================================================

#[test]
fn test_double_negative_all_languages() {
    let go = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, --42, 32, h) }
"#,
    );

    let python = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm, length, salt, --42)
"#,
    );

    let rust = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(algorithm, --42, salt, password, &mut out);
}
"#,
    );

    // All should resolve to 42
    assert_eq!(get_first_arg_int(&go, 2), Some(42), "Go: --42 = 42");
    assert_eq!(get_first_arg_int(&python, 3), Some(42), "Python: --42 = 42");
    assert_eq!(get_first_arg_int(&rust, 1), Some(42), "Rust: --42 = 42");
}
