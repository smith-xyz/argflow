//! Python composite resolution tests

use super::test_utils::scan_python;

// =============================================================================
// List Literals
// =============================================================================

#[test]
fn test_int_list() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm="sha256", length=32, salt=[1, 2, 3], iterations=100000)
"#,
    );
    // List as argument
    assert!(result.calls.len() == 1);
}

#[test]
fn test_bytes_literal() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm="sha256", length=32, salt=b'\x01\x02\x03', iterations=100000)
"#,
    );
    // Bytes literal
    assert!(result.calls.len() == 1);
}

#[test]
fn test_bytearray_call() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
salt = bytearray([0xDE, 0xAD, 0xBE, 0xEF])
kdf = PBKDF2HMAC(algorithm="sha256", length=32, salt=salt, iterations=100000)
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Tuple Literals
// =============================================================================

#[test]
fn test_tuple_literal() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
key_sizes = (128, 192, 256)
kdf = PBKDF2HMAC(algorithm="sha256", length=key_sizes[0], salt=b'salt', iterations=100000)
"#,
    );
    // Tuple indexed access
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Dictionary Literals
// =============================================================================

#[test]
fn test_dict_literal_access() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
config = {"iterations": 100000, "key_size": 32}
kdf = PBKDF2HMAC(algorithm="sha256", length=config["key_size"], salt=b'salt', iterations=config["iterations"])
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// List Comprehensions (Edge Case)
// =============================================================================

#[test]
fn test_list_comprehension() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
salt = bytes([x for x in range(16)])
kdf = PBKDF2HMAC(algorithm="sha256", length=32, salt=salt, iterations=100000)
"#,
    );
    // List comprehension - complex expression
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Empty Collections
// =============================================================================

#[test]
fn test_empty_list() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm="sha256", length=32, salt=[], iterations=100000)
"#,
    );
    assert!(result.calls.len() == 1);
}

#[test]
fn test_empty_dict() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
config = {}
kdf = PBKDF2HMAC(algorithm="sha256", length=32, salt=b'salt', iterations=100000)
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Nested Collections
// =============================================================================

#[test]
fn test_nested_list() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
keys = [[1, 2], [3, 4]]
kdf = PBKDF2HMAC(algorithm="sha256", length=32, salt=bytes(keys[0]), iterations=100000)
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Crypto-Relevant Scenarios
// =============================================================================

#[test]
fn test_salt_bytes() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
salt = bytes([0xDE, 0xAD, 0xBE, 0xEF])
kdf = PBKDF2HMAC(algorithm="sha256", length=32, salt=salt, iterations=100000)
"#,
    );
    assert!(!result.calls.is_empty());
}

#[test]
fn test_algorithm_from_list() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
algorithms = ["sha256", "sha384", "sha512"]
kdf = PBKDF2HMAC(algorithm=algorithms[0], length=32, salt=b'salt', iterations=100000)
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Mixed Types
// =============================================================================

#[test]
fn test_mixed_type_list() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
config = [100000, "sha256", 32]
kdf = PBKDF2HMAC(algorithm=config[1], length=config[2], salt=b'salt', iterations=config[0])
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Set Literals
// =============================================================================

#[test]
fn test_set_literal() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
allowed_sizes = {16, 24, 32}
kdf = PBKDF2HMAC(algorithm="sha256", length=32, salt=b'salt', iterations=100000)
"#,
    );
    // Set exists but not used as argument
    assert!(!result.calls.is_empty());
}
