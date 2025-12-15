//! JavaScript composite resolution tests

use super::test_utils::scan_javascript;

// =============================================================================
// Array Literals
// =============================================================================

#[test]
fn test_number_array() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const sizes = [16, 24, 32];
crypto.pbkdf2Sync(password, salt, 100000, sizes[0], 'sha256');
"#,
    );
    assert!(!result.calls.is_empty());
}

#[test]
fn test_uint8array() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const salt = new Uint8Array([0xDE, 0xAD, 0xBE, 0xEF]);
crypto.pbkdf2Sync(password, salt, 100000, 32, 'sha256');
"#,
    );
    assert!(!result.calls.is_empty());
}

#[test]
fn test_buffer_from_array() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const salt = Buffer.from([0x01, 0x02, 0x03]);
crypto.pbkdf2Sync(password, salt, 100000, 32, 'sha256');
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Object Literals
// =============================================================================

#[test]
fn test_config_object() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const config = { iterations: 100000, keyLen: 32 };
crypto.pbkdf2Sync(password, salt, config.iterations, config.keyLen, 'sha256');
"#,
    );
    assert!(!result.calls.is_empty());
}

#[test]
fn test_inline_object() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.createCipheriv('aes-256-gcm', key, iv, { authTagLength: 16 });
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Destructuring Patterns
// =============================================================================

#[test]
fn test_array_destructure() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const [iter, keyLen] = [100000, 32];
crypto.pbkdf2Sync(password, salt, iter, keyLen, 'sha256');
"#,
    );
    assert!(!result.calls.is_empty());
}

#[test]
fn test_object_destructure() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const { iterations, keyLength } = { iterations: 100000, keyLength: 32 };
crypto.pbkdf2Sync(password, salt, iterations, keyLength, 'sha256');
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Spread Operator
// =============================================================================

#[test]
fn test_spread_array() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const base = [100000, 32];
crypto.pbkdf2Sync(password, salt, ...base, 'sha256');
"#,
    );
    // Spread operator - complex
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Empty Collections
// =============================================================================

#[test]
fn test_empty_array() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const empty = [];
crypto.pbkdf2Sync(password, salt, 100000, 32, 'sha256');
"#,
    );
    assert!(!result.calls.is_empty());
}

#[test]
fn test_empty_object() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
crypto.createCipheriv('aes-256-gcm', key, iv, {});
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Nested Collections
// =============================================================================

#[test]
fn test_nested_array() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const configs = [[100000, 32], [200000, 64]];
crypto.pbkdf2Sync(password, salt, configs[0][0], configs[0][1], 'sha256');
"#,
    );
    assert!(!result.calls.is_empty());
}

#[test]
fn test_nested_object() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const config = { pbkdf2: { iterations: 100000, keyLen: 32 } };
crypto.pbkdf2Sync(password, salt, config.pbkdf2.iterations, config.pbkdf2.keyLen, 'sha256');
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Crypto-Relevant Scenarios
// =============================================================================

#[test]
fn test_cipher_options() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const options = {
    authTagLength: 16,
    padding: 'auto'
};
crypto.createCipheriv('aes-256-gcm', key, iv, options);
"#,
    );
    assert!(!result.calls.is_empty());
}

#[test]
fn test_algorithms_array() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const algorithms = ['aes-128-gcm', 'aes-256-gcm', 'chacha20-poly1305'];
crypto.createCipheriv(algorithms[1], key, iv);
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// TypedArrays
// =============================================================================

#[test]
fn test_typed_array_literal() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const iv = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
crypto.createCipheriv('aes-256-gcm', key, iv);
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Template Literals (Edge Case)
// =============================================================================

#[test]
fn test_template_in_array() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const prefix = 'aes';
const algorithms = [`${prefix}-128-gcm`, `${prefix}-256-gcm`];
crypto.createCipheriv(algorithms[0], key, iv);
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Computed Properties
// =============================================================================

#[test]
fn test_computed_property() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const key = 'iterations';
const config = { [key]: 100000 };
crypto.pbkdf2Sync(password, salt, config.iterations, 32, 'sha256');
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Shorthand Properties
// =============================================================================

#[test]
fn test_shorthand_property() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const iterations = 100000;
const keyLength = 32;
const config = { iterations, keyLength };
crypto.pbkdf2Sync(password, salt, config.iterations, config.keyLength, 'sha256');
"#,
    );
    assert!(!result.calls.is_empty());
}
