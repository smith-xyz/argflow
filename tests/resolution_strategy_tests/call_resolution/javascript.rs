//! JavaScript-specific call resolution tests

use super::test_utils::*;

#[test]
fn test_js_simple_int_return() {
    let source = r#"
const crypto = require('crypto');

function getIterations() {
    return 10000;
}

crypto.pbkdf2Sync("password", "salt", getIterations(), 32, "sha256");
"#;
    let result = scan_javascript(source);

    assert_eq!(result.calls.len(), 1);
    assert!(is_arg_resolved(&result, 2));
    assert_eq!(get_first_arg_int(&result, 2), Some(10000));
}

#[test]
fn test_js_simple_string_return() {
    let source = r#"
const crypto = require('crypto');

function getAlgorithm() {
    return "sha256";
}

crypto.pbkdf2Sync("password", "salt", 10000, 32, getAlgorithm());
"#;
    let result = scan_javascript(source);

    assert_eq!(result.calls.len(), 1);
    assert!(is_arg_resolved(&result, 4));
    assert_eq!(get_first_arg_string(&result, 4), Some("sha256".to_string()));
}

#[test]
fn test_js_if_else_returns() {
    let source = r#"
const crypto = require('crypto');

function getKeySize(aes256) {
    if (aes256) {
        return 32;
    }
    return 16;
}

crypto.pbkdf2Sync("password", "salt", 10000, getKeySize(true), "sha256");
"#;
    let result = scan_javascript(source);

    assert_eq!(result.calls.len(), 1);
    let key_sizes = get_first_arg_ints(&result, 3);
    assert!(key_sizes.contains(&32));
    assert!(key_sizes.contains(&16));
}

#[test]
fn test_js_switch_returns() {
    let source = r#"
const crypto = require('crypto');

function getBlockSize(mode) {
    switch (mode) {
        case "AES-128":
            return 16;
        case "AES-192":
            return 24;
        case "AES-256":
            return 32;
        default:
            return 16;
    }
}

crypto.pbkdf2Sync("password", "salt", 10000, getBlockSize("AES-256"), "sha256");
"#;
    let result = scan_javascript(source);

    assert_eq!(result.calls.len(), 1);
    let sizes = get_first_arg_ints(&result, 3);
    assert!(sizes.contains(&16));
    assert!(sizes.contains(&24));
    assert!(sizes.contains(&32));
}

#[test]
fn test_js_function_not_found() {
    let source = r#"
const crypto = require('crypto');

crypto.pbkdf2Sync("password", "salt", unknownFunction(), 32, "sha256");
"#;
    let result = scan_javascript(source);

    assert_eq!(result.calls.len(), 1);
    assert!(!is_arg_resolved(&result, 2));
    assert_eq!(
        get_arg_source(&result, 2),
        Some("function_not_found".to_string())
    );
}

#[test]
fn test_js_arrow_function_return() {
    let source = r#"
const crypto = require('crypto');

const getIterations = () => {
    return 10000;
};

crypto.pbkdf2Sync("password", "salt", getIterations(), 32, "sha256");
"#;
    let result = scan_javascript(source);

    // Arrow functions might need special handling
    assert_eq!(result.calls.len(), 1);
}
