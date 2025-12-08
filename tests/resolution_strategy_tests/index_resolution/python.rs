//! Python index resolution tests
//!
//! Note: Many Python crypto tests are currently skipped because the scanner
//! doesn't detect PBKDF2HMAC calls with the default patterns. The index
//! strategy itself works correctly - this is a scanner configuration issue.
//! Tests that don't require crypto detection (direct dict/list tests) pass.

use super::test_utils::{
    get_arg_expression, get_first_arg_int, get_first_arg_string, is_arg_unresolved, scan_python,
};

// =============================================================================
// List Integer Index Tests
// Note: These tests are ignored because scanner doesn't detect PBKDF2HMAC
// =============================================================================

#[test]
fn test_list_index_first() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm=h, length=32, salt=s, iterations=[10000, 20000, 30000][0])
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(10000), "First element");
}

#[test]

fn test_list_index_middle() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm=h, length=32, salt=s, iterations=[10000, 20000, 30000][1])
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(20000), "Middle element");
}

#[test]

fn test_list_index_last() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm=h, length=32, salt=s, iterations=[10000, 20000, 30000][2])
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(30000), "Last element");
}

// =============================================================================
// Tuple Index Tests
// =============================================================================

#[test]

fn test_tuple_index_first() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm=h, length=32, salt=s, iterations=(10000, 20000, 30000)[0])
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(10000),
        "First tuple element"
    );
}

#[test]

fn test_tuple_index_last() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm=h, length=32, salt=s, iterations=(10000, 20000, 30000)[2])
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(30000),
        "Last tuple element"
    );
}

// =============================================================================
// Dictionary String Key Tests
// =============================================================================

#[test]

fn test_dict_string_key_iterations() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm=h, length=32, salt=s, iterations={"iterations": 100000}["iterations"])
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(100000),
        "Dict iterations value"
    );
}

#[test]

fn test_dict_string_key_length() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm=h, length={"key_length": 32}["key_length"], salt=s, iterations=10000)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(32), "Dict length value");
}

#[test]

fn test_dict_string_value() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
from cryptography.hazmat.primitives import hashes
kdf = PBKDF2HMAC(algorithm={"algo": "SHA256"}["algo"], length=32, salt=s, iterations=10000)
"#,
    );
    assert_eq!(
        get_first_arg_string(&result, 0),
        Some("SHA256".to_string()),
        "Dict string value"
    );
}

// =============================================================================
// String List Index Tests
// =============================================================================

#[test]

fn test_string_list_index() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm=["sha256", "sha384", "sha512"][0], length=32, salt=s, iterations=10000)
"#,
    );
    assert_eq!(
        get_first_arg_string(&result, 0),
        Some("sha256".to_string()),
        "First algorithm"
    );
}

#[test]

fn test_string_list_index_second() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm=["sha256", "sha384", "sha512"][2], length=32, salt=s, iterations=10000)
"#,
    );
    assert_eq!(
        get_first_arg_string(&result, 0),
        Some("sha512".to_string()),
        "Third algorithm"
    );
}

// =============================================================================
// Key Size Tests (Crypto-relevant)
// =============================================================================

#[test]

fn test_key_sizes_list_128() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm=h, length=[16, 24, 32][0], salt=s, iterations=10000)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(16), "AES-128 key size");
}

#[test]

fn test_key_sizes_list_256() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm=h, length=[16, 24, 32][2], salt=s, iterations=10000)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(32), "AES-256 key size");
}

// =============================================================================
// Out of Bounds Tests
// =============================================================================

#[test]

fn test_index_out_of_bounds() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm=h, length=32, salt=s, iterations=[10000, 20000][5])
"#,
    );
    assert!(is_arg_unresolved(&result, 3), "Out of bounds index");
}

// Note: Python supports negative indexing, but our implementation doesn't yet
#[test]

fn test_negative_index() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm=h, length=32, salt=s, iterations=[10000, 20000][-1])
"#,
    );
    // Current implementation doesn't support negative indexing
    assert!(
        is_arg_unresolved(&result, 3),
        "Negative index not supported yet"
    );
}

// =============================================================================
// Non-literal Index Tests
// =============================================================================

#[test]

fn test_variable_index() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
i = 0
kdf = PBKDF2HMAC(algorithm=h, length=32, salt=s, iterations=[10000, 20000][i])
"#,
    );
    assert!(
        is_arg_unresolved(&result, 3),
        "Variable index needs identifier resolution"
    );
}

// =============================================================================
// Non-literal List Tests
// =============================================================================

#[test]

fn test_variable_list() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
sizes = [16, 24, 32]
kdf = PBKDF2HMAC(algorithm=h, length=sizes[0], salt=s, iterations=10000)
"#,
    );
    assert!(
        is_arg_unresolved(&result, 1),
        "Variable list needs identifier resolution"
    );
    assert_eq!(
        get_arg_expression(&result, 1),
        Some("sizes[0]".to_string()),
        "Expression preserved"
    );
}

// =============================================================================
// Empty Collection Tests
// =============================================================================

#[test]
fn test_empty_list_index() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm=h, length=32, salt=s, iterations=[][0])
"#,
    );
    assert!(
        is_arg_unresolved(&result, 3),
        "Empty list index out of bounds"
    );
}

#[test]
fn test_empty_dict_key() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm=h, length=32, salt=s, iterations={}["key"])
"#,
    );
    assert!(is_arg_unresolved(&result, 3), "Empty dict key not found");
}

// =============================================================================
// Hex Value Tests
// =============================================================================

#[test]

fn test_hex_values_in_list() {
    let result = scan_python(
        r#"
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
kdf = PBKDF2HMAC(algorithm=h, length=[0x10, 0x18, 0x20][0], salt=s, iterations=10000)
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(16), "Hex 0x10 = 16");
}
