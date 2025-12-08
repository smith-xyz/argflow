//! Python-specific call resolution tests

use super::test_utils::*;

#[test]
fn test_python_simple_int_return() {
    let source = r#"
from hashlib import pbkdf2_hmac

def get_iterations():
    return 10000

pbkdf2_hmac("sha256", b"password", b"salt", get_iterations())
"#;
    let result = scan_python(source);

    assert_eq!(result.calls.len(), 1);
    assert!(is_arg_resolved(&result, 3));
    assert_eq!(get_first_arg_int(&result, 3), Some(10000));
}

#[test]
fn test_python_simple_string_return() {
    let source = r#"
from hashlib import pbkdf2_hmac

def get_algorithm():
    return "sha256"

pbkdf2_hmac(get_algorithm(), b"password", b"salt", 10000)
"#;
    let result = scan_python(source);

    assert_eq!(result.calls.len(), 1);
    assert!(is_arg_resolved(&result, 0));
    assert_eq!(get_first_arg_string(&result, 0), Some("sha256".to_string()));
}

#[test]
fn test_python_if_else_returns() {
    let source = r#"
from hashlib import pbkdf2_hmac

def get_key_size(aes256):
    if aes256:
        return 32
    return 16

pbkdf2_hmac("sha256", b"password", b"salt", get_key_size(True))
"#;
    let result = scan_python(source);

    assert_eq!(result.calls.len(), 1);
    // The key_size argument should have multiple possible values
    let key_sizes = get_first_arg_ints(&result, 3);
    assert!(key_sizes.contains(&32) || key_sizes.contains(&16));
}

#[test]
fn test_python_tuple_return() {
    let source = r#"
from hashlib import pbkdf2_hmac

def get_config():
    return 10000, 32

pbkdf2_hmac("sha256", b"password", b"salt", get_config())
"#;
    let result = scan_python(source);

    assert_eq!(result.calls.len(), 1);
    // Tuple returns should be detected
    let ints = get_first_arg_ints(&result, 3);
    assert!(ints.contains(&10000));
    assert!(ints.contains(&32));
}

#[test]
fn test_python_function_not_found() {
    let source = r#"
from hashlib import pbkdf2_hmac

pbkdf2_hmac("sha256", b"password", b"salt", unknown_function())
"#;
    let result = scan_python(source);

    assert_eq!(result.calls.len(), 1);
    assert!(!is_arg_resolved(&result, 3));
    assert_eq!(
        get_arg_source(&result, 3),
        Some("function_not_found".to_string())
    );
}
