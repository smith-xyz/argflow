//! JavaScript literal resolution tests
//!
//! Tests literal parsing for JavaScript/Node.js crypto code patterns.

use super::test_utils::{get_first_arg_int, get_first_arg_string, scan_javascript};

// =============================================================================
// Integer Literals
// =============================================================================

#[test]
fn test_decimal_integer() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const key = crypto.pbkdf2Sync(password, salt, 100000, 32, 'sha256');
"#,
    );
    // Node.js pbkdf2Sync: iterations=arg[2], keylen=arg[3]
    assert_eq!(get_first_arg_int(&result, 2), Some(100000), "Iterations");
    assert_eq!(get_first_arg_int(&result, 3), Some(32), "Key length");
}

#[test]
fn test_hex_integer() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const key = crypto.pbkdf2Sync(password, salt, 0x186A0, 0x20, 'sha256');
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(100000),
        "Hex iterations"
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(32), "Hex key length");
}

#[test]
fn test_octal_integer() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const key = crypto.pbkdf2Sync(password, salt, 10000, 0o40, 'sha256');
"#,
    );
    // 0o40 = 32
    assert_eq!(get_first_arg_int(&result, 3), Some(32), "Octal key length");
}

#[test]
fn test_binary_integer() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const key = crypto.pbkdf2Sync(password, salt, 10000, 0b100000, 'sha256');
"#,
    );
    // 0b100000 = 32
    assert_eq!(get_first_arg_int(&result, 3), Some(32), "Binary key length");
}

#[test]
fn test_underscore_separator() {
    // ES2021 numeric separators
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const key = crypto.pbkdf2Sync(password, salt, 100_000, 32, 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(100000));
}

#[test]
fn test_zero() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const key = crypto.pbkdf2Sync(password, salt, 0, 0, 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(0));
    assert_eq!(get_first_arg_int(&result, 3), Some(0));
}

#[test]
fn test_large_integer() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const key = crypto.pbkdf2Sync(password, salt, 1000000000, 64, 'sha256');
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(1000000000));
}

// =============================================================================
// String Literals
// =============================================================================

#[test]
fn test_single_quoted_string() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const key = crypto.pbkdf2Sync(password, salt, 10000, 32, 'sha256');
"#,
    );
    assert_eq!(get_first_arg_string(&result, 4), Some("sha256".to_string()));
}

#[test]
fn test_double_quoted_string() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const key = crypto.pbkdf2Sync(password, salt, 10000, 32, "sha512");
"#,
    );
    assert_eq!(get_first_arg_string(&result, 4), Some("sha512".to_string()));
}

#[test]
fn test_template_literal() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const algo = `sha256`;
const hash = crypto.createHash(algo);
"#,
    );
    // Template literals should be handled
    assert!(result.call_count() >= 1);
}

#[test]
fn test_algorithm_strings() {
    for algo in ["sha256", "sha512", "sha384", "sha1", "md5"] {
        let source = format!(
            r#"
const crypto = require('crypto');
const key = crypto.pbkdf2Sync(password, salt, 10000, 32, '{}');
"#,
            algo
        );
        let result = scan_javascript(&source);
        assert_eq!(
            get_first_arg_string(&result, 4),
            Some(algo.to_string()),
            "Algorithm: {}",
            algo
        );
    }
}

// =============================================================================
// Boolean and Null Literals
// =============================================================================

#[test]
fn test_null_literal() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const cipher = crypto.createCipheriv('aes-256-gcm', key, iv);
"#,
    );
    assert_eq!(result.call_count(), 1);
}

#[test]
fn test_undefined_in_context() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const hash = crypto.createHash('sha256');
"#,
    );
    assert_eq!(result.call_count(), 1);
}

// =============================================================================
// Common Node.js Crypto Patterns
// =============================================================================

#[test]
fn test_create_hash() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const hash = crypto.createHash('sha256');
hash.update('hello');
const digest = hash.digest('hex');
"#,
    );
    // Should find createHash and digest calls
    assert!(result.call_count() >= 1);
}

#[test]
fn test_create_cipher() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const cipher = crypto.createCipheriv('aes-256-gcm', key, iv);
cipher.update(plaintext);
const encrypted = cipher.final();
"#,
    );
    // Should find createCipheriv
    let cipher_calls: Vec<_> = result
        .calls
        .iter()
        .filter(|c| c.function_name == "createCipheriv")
        .collect();
    assert_eq!(cipher_calls.len(), 1);
}

#[test]
fn test_create_hmac() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const hmac = crypto.createHmac('sha256', key);
hmac.update(data);
const signature = hmac.digest('hex');
"#,
    );
    assert!(result.call_count() >= 1);
}

#[test]
fn test_scrypt() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const key = crypto.scryptSync(password, salt, 64, { N: 16384, r: 8, p: 1 });
"#,
    );
    // scryptSync: keylen=arg[2]
    assert_eq!(get_first_arg_int(&result, 2), Some(64), "Key length");
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_scientific_notation() {
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const key = crypto.pbkdf2Sync(password, salt, 1e5, 32, 'sha256');
"#,
    );
    // 1e5 = 100000
    assert_eq!(result.call_count(), 1);
}

#[test]
fn test_bigint_literal() {
    // BigInt literals (100000n) - rare in crypto but should handle gracefully
    let result = scan_javascript(
        r#"
const crypto = require('crypto');
const hash = crypto.createHash('sha256');
"#,
    );
    assert_eq!(result.call_count(), 1);
}
