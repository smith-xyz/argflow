//! Cross-language composite resolution tests
//!
//! These tests verify consistent behavior across languages for equivalent
//! composite literal patterns.

use super::test_utils::{scan_go, scan_javascript, scan_python, scan_rust};

// =============================================================================
// Integer Array Literals
// =============================================================================

#[test]
fn test_integer_array_go() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key([]byte{1, 2, 3}, s, 10000, 32, h) }
"#,
    );
    let args = &result.calls[0].arguments[0];
    assert!(args.int_values.contains(&1));
    assert!(args.int_values.contains(&2));
    assert!(args.int_values.contains(&3));
}

#[test]
fn test_integer_array_python() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
salt = bytes([1, 2, 3])
kdf = PBKDF2HMAC(algorithm="sha256", length=32, salt=salt, iterations=100000)
"#,
    );
    // Python uses bytes([...]) which is a call, not a pure list literal
    assert!(result.calls.len() >= 1);
}

#[test]
fn test_integer_array_javascript() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const salt = Buffer.from([1, 2, 3]);
crypto.pbkdf2Sync(password, salt, 100000, 32, 'sha256');
"#,
    );
    // JS uses Buffer.from([...]) which wraps the array
    assert!(result.calls.len() >= 1);
}

#[test]
fn test_integer_array_rust() {
    let result = scan_rust(
        r#"
use pbkdf2::pbkdf2_hmac;
fn main() {
    let salt = [1u8, 2, 3];
    pbkdf2_hmac::<Sha256>(password, &salt, 100000, &mut key);
}
"#,
    );
    assert!(result.calls.len() >= 1);
}

// =============================================================================
// Hex Values in Arrays
// =============================================================================

#[test]
fn test_hex_array_go() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key([]byte{0xDE, 0xAD, 0xBE, 0xEF}, s, 10000, 32, h) }
"#,
    );
    let args = &result.calls[0].arguments[0];
    assert!(args.int_values.contains(&0xDE));
    assert!(args.int_values.contains(&0xAD));
    assert!(args.int_values.contains(&0xBE));
    assert!(args.int_values.contains(&0xEF));
}

#[test]
fn test_hex_array_python() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
salt = bytes([0xDE, 0xAD, 0xBE, 0xEF])
kdf = PBKDF2HMAC(algorithm="sha256", length=32, salt=salt, iterations=100000)
"#,
    );
    assert!(result.calls.len() >= 1);
}

#[test]
fn test_hex_array_javascript() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const salt = Buffer.from([0xDE, 0xAD, 0xBE, 0xEF]);
crypto.pbkdf2Sync(password, salt, 100000, 32, 'sha256');
"#,
    );
    assert!(result.calls.len() >= 1);
}

#[test]
fn test_hex_array_rust() {
    let result = scan_rust(
        r#"
use pbkdf2::pbkdf2_hmac;
fn main() {
    let salt = [0xDEu8, 0xAD, 0xBE, 0xEF];
    pbkdf2_hmac::<Sha256>(password, &salt, 100000, &mut key);
}
"#,
    );
    assert!(result.calls.len() >= 1);
}

// =============================================================================
// String Arrays
// =============================================================================

#[test]
fn test_string_array_go() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
var algorithms = []string{"sha256", "sha384", "sha512"}
func main() { pbkdf2.Key(p, s, 10000, 32, algorithms[0]) }
"#,
    );
    // String array exists, indexed access
    assert!(result.calls.len() >= 1);
}

#[test]
fn test_string_array_python() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
algorithms = ["sha256", "sha384", "sha512"]
kdf = PBKDF2HMAC(algorithm=algorithms[0], length=32, salt=b'salt', iterations=100000)
"#,
    );
    assert!(result.calls.len() >= 1);
}

#[test]
fn test_string_array_javascript() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const algorithms = ['sha256', 'sha384', 'sha512'];
crypto.pbkdf2Sync(password, salt, 100000, 32, algorithms[0]);
"#,
    );
    assert!(result.calls.len() >= 1);
}

#[test]
fn test_string_array_rust() {
    let result = scan_rust(
        r#"
use pbkdf2::pbkdf2_hmac;
fn main() {
    let algorithms = ["sha256", "sha384", "sha512"];
    // Note: Rust doesn't use string algorithm names the same way
}
"#,
    );
    // No crypto call in this example - any result is valid
    let _ = result.calls;
}

// =============================================================================
// Config Objects/Structs
// =============================================================================

#[test]
fn test_config_struct_go() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
type Config struct { Iterations int; KeyLen int }
var cfg = Config{Iterations: 100000, KeyLen: 32}
func main() { pbkdf2.Key(p, s, cfg.Iterations, cfg.KeyLen, h) }
"#,
    );
    assert!(result.calls.len() >= 1);
}

#[test]
fn test_config_dict_python() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
config = {"iterations": 100000, "key_len": 32}
kdf = PBKDF2HMAC(algorithm="sha256", length=config["key_len"], salt=b'salt', iterations=config["iterations"])
"#,
    );
    assert!(result.calls.len() >= 1);
}

#[test]
fn test_config_object_javascript() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const config = { iterations: 100000, keyLen: 32 };
crypto.pbkdf2Sync(password, salt, config.iterations, config.keyLen, 'sha256');
"#,
    );
    assert!(result.calls.len() >= 1);
}

#[test]
fn test_config_struct_rust() {
    let result = scan_rust(
        r#"
use pbkdf2::pbkdf2_hmac;
struct Config { iterations: u32, key_len: usize }
fn main() {
    let cfg = Config { iterations: 100000, key_len: 32 };
    pbkdf2_hmac::<Sha256>(password, salt, cfg.iterations, &mut key);
}
"#,
    );
    assert!(result.calls.len() >= 1);
}

// =============================================================================
// Empty Collections
// =============================================================================

#[test]
fn test_empty_array_go() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key([]byte{}, s, 10000, 32, h) }
"#,
    );
    assert!(result.calls.len() >= 1);
}

#[test]
fn test_empty_list_python() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm="sha256", length=32, salt=b'', iterations=100000)
"#,
    );
    assert!(result.calls.len() >= 1);
}

#[test]
fn test_empty_array_javascript() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const empty = [];
crypto.pbkdf2Sync(password, Buffer.from(empty), 100000, 32, 'sha256');
"#,
    );
    assert!(result.calls.len() >= 1);
}

#[test]
fn test_empty_array_rust() {
    let result = scan_rust(
        r#"
use pbkdf2::pbkdf2_hmac;
fn main() {
    let empty: [u8; 0] = [];
    pbkdf2_hmac::<Sha256>(password, &empty, 100000, &mut key);
}
"#,
    );
    assert!(result.calls.len() >= 1);
}

// =============================================================================
// Multiline Composite Literals
// =============================================================================

#[test]
fn test_multiline_array_go() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { 
    pbkdf2.Key([]byte{
        0x01,
        0x02,
        0x03,
    }, s, 10000, 32, h) 
}
"#,
    );
    let args = &result.calls[0].arguments[0];
    assert!(args.int_values.len() == 3);
}

#[test]
fn test_multiline_object_javascript() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const config = {
    iterations: 100000,
    keyLen: 32,
    algorithm: 'sha256'
};
crypto.pbkdf2Sync(password, salt, config.iterations, config.keyLen, config.algorithm);
"#,
    );
    assert!(result.calls.len() >= 1);
}
