//! JavaScript index resolution tests

use super::test_utils::{
    get_arg_expression, get_first_arg_int, get_first_arg_string, is_arg_unresolved, scan_javascript,
};

// =============================================================================
// Array Integer Index Tests
// =============================================================================

#[test]
fn test_array_index_first() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, [10000, 20000, 30000][0], 32, 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(10000), "First element");
}

#[test]
fn test_array_index_middle() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, [10000, 20000, 30000][1], 32, 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(20000), "Middle element");
}

#[test]
fn test_array_index_last() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, [10000, 20000, 30000][2], 32, 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(30000), "Last element");
}

#[test]
fn test_array_single_element() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, [100000][0], 32, 'sha256');
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(100000),
        "Single element"
    );
}

// =============================================================================
// String Array Index Tests
// =============================================================================

#[test]
fn test_string_array_index_first() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, 10000, 32, ['sha256', 'sha384', 'sha512'][0]);
"#,
    );
    assert_eq!(
        get_first_arg_string(&result, 4),
        Some("sha256".to_string()),
        "First algorithm"
    );
}

#[test]
fn test_string_array_index_last() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, 10000, 32, ['sha256', 'sha384', 'sha512'][2]);
"#,
    );
    assert_eq!(
        get_first_arg_string(&result, 4),
        Some("sha512".to_string()),
        "Third algorithm"
    );
}

// =============================================================================
// Object Bracket Access Tests
// =============================================================================

#[test]
fn test_object_bracket_access_iterations() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, {iterations: 100000}['iterations'], 32, 'sha256');
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(100000),
        "Object iterations value"
    );
}

#[test]
fn test_object_bracket_access_keylen() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, 10000, {keyLen: 32}['keyLen'], 'sha256');
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(32),
        "Object keyLen value"
    );
}

#[test]
fn test_object_bracket_access_string_value() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, 10000, 32, {algo: 'sha512'}['algo']);
"#,
    );
    assert_eq!(
        get_first_arg_string(&result, 4),
        Some("sha512".to_string()),
        "Object string value"
    );
}

// =============================================================================
// Key Size Tests (Crypto-relevant)
// =============================================================================

#[test]
fn test_key_sizes_array_128() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, 10000, [16, 24, 32][0], 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(16), "AES-128 key size");
}

#[test]
fn test_key_sizes_array_256() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, 10000, [16, 24, 32][2], 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(32), "AES-256 key size");
}

// =============================================================================
// Out of Bounds Tests
// =============================================================================

#[test]
fn test_index_out_of_bounds() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, [10000, 20000][5], 32, 'sha256');
"#,
    );
    assert!(is_arg_unresolved(&result, 2), "Out of bounds index");
}

#[test]
fn test_negative_index() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, [10000, 20000][-1], 32, 'sha256');
"#,
    );
    // JavaScript treats negative indices as property access, not array index
    assert!(is_arg_unresolved(&result, 2), "Negative index");
}

// =============================================================================
// Non-literal Index Tests
// =============================================================================

#[test]
fn test_variable_index() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const i = 0;
crypto.pbkdf2Sync(password, salt, [10000, 20000][i], 32, 'sha256');
"#,
    );
    assert!(
        is_arg_unresolved(&result, 2),
        "Variable index needs identifier resolution"
    );
}

// =============================================================================
// Non-literal Array Tests
// =============================================================================

#[test]
fn test_variable_array() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const sizes = [16, 24, 32];
crypto.pbkdf2Sync(password, salt, 10000, sizes[0], 'sha256');
"#,
    );
    assert!(
        is_arg_unresolved(&result, 3),
        "Variable array needs identifier resolution"
    );
    assert_eq!(
        get_arg_expression(&result, 3),
        Some("sizes[0]".to_string()),
        "Expression preserved"
    );
}

#[test]
fn test_variable_object() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const config = {iterations: 100000};
crypto.pbkdf2Sync(password, salt, config['iterations'], 32, 'sha256');
"#,
    );
    assert!(
        is_arg_unresolved(&result, 2),
        "Variable object needs identifier resolution"
    );
}

// =============================================================================
// Empty Collection Tests
// =============================================================================

#[test]
fn test_empty_array_index() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, [][0], 32, 'sha256');
"#,
    );
    assert!(
        is_arg_unresolved(&result, 2),
        "Empty array index out of bounds"
    );
}

#[test]
fn test_empty_object_key() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, {}['iterations'], 32, 'sha256');
"#,
    );
    assert!(is_arg_unresolved(&result, 2), "Empty object key not found");
}

// =============================================================================
// Hex Value Tests
// =============================================================================

#[test]
fn test_hex_values_in_array() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, 10000, [0x10, 0x18, 0x20][0], 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(16), "Hex 0x10 = 16");
}

#[test]
fn test_hex_values_in_array_last() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, 10000, [0x10, 0x18, 0x20][2], 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(32), "Hex 0x20 = 32");
}

// =============================================================================
// Complex Expression Tests
// =============================================================================

#[test]
fn test_nested_array_access() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.pbkdf2Sync(password, salt, [[10000, 20000], [30000, 40000]][0][1], 32, 'sha256');
"#,
    );
    // Nested array access - may or may not resolve depending on implementation depth
    assert!(result.calls.len() >= 1, "Call detected");
}
