//! Rust literal resolution tests
//!
//! Tests literal parsing for Rust code patterns.
//! Note: Module named `rust_lang` to avoid conflict with `rust` keyword.
//!
//! LIMITATION: Rust's `Type::method()` syntax (scoped identifiers) is not yet
//! fully supported by the scanner. These tests use simple function calls to
//! verify literal parsing works correctly.

use super::test_utils::{get_first_arg_int, get_first_arg_string, scan_rust};

// =============================================================================
// Integer Literals
// =============================================================================

#[test]
fn test_decimal_integer() {
    let result = scan_rust(
        r#"
fn pbkdf2(password: &[u8], salt: &[u8], iterations: u32, key_len: usize) {}
fn main() {
    pbkdf2(password, salt, 100000, 32);
}
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(100000), "Iterations");
    assert_eq!(get_first_arg_int(&result, 3), Some(32), "Key length");
}

#[test]
fn test_hex_integer() {
    let result = scan_rust(
        r#"
fn encrypt(data: &[u8], key_size: usize) {}
fn main() {
    encrypt(data, 0x20);
}
"#,
    );
    // 0x20 = 32
    assert_eq!(result.call_count(), 1);
    assert_eq!(get_first_arg_int(&result, 1), Some(32));
}

#[test]
fn test_octal_integer() {
    let result = scan_rust(
        r#"
fn hash(data: &[u8], rounds: usize) {}
fn main() {
    hash(data, 0o40);
}
"#,
    );
    // 0o40 = 32
    assert_eq!(get_first_arg_int(&result, 1), Some(32));
}

#[test]
fn test_binary_integer() {
    let result = scan_rust(
        r#"
fn cipher(key: &[u8], block_size: usize) {}
fn main() {
    cipher(key, 0b100000);
}
"#,
    );
    // 0b100000 = 32
    assert_eq!(get_first_arg_int(&result, 1), Some(32));
}

#[test]
fn test_underscore_separator() {
    let result = scan_rust(
        r#"
fn pbkdf2(password: &[u8], salt: &[u8], iterations: u32) {}
fn main() {
    pbkdf2(password, salt, 100_000);
}
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(100000));
}

#[test]
fn test_type_suffix_integer() {
    let result = scan_rust(
        r#"
fn encrypt(data: &[u8], iterations: u32) {}
fn main() {
    encrypt(data, 100000u32);
}
"#,
    );
    assert_eq!(result.call_count(), 1);
    assert_eq!(get_first_arg_int(&result, 1), Some(100000), "u32 suffix");
}

#[test]
fn test_various_type_suffixes() {
    for (suffix, expected) in [
        ("100i8", 100),
        ("200u16", 200),
        ("300i32", 300),
        ("400u64", 400),
        ("500i128", 500),
        ("600usize", 600),
    ] {
        let source = format!(
            r#"
fn cipher(val: i128) {{}}
fn main() {{
    cipher({});
}}
"#,
            suffix
        );
        let result = scan_rust(&source);
        assert_eq!(
            get_first_arg_int(&result, 0),
            Some(expected),
            "Suffix: {}",
            suffix
        );
    }
}

#[test]
fn test_zero() {
    let result = scan_rust(
        r#"
fn cipher(key: &[u8], iv_size: usize) {}
fn main() {
    cipher(key, 0);
}
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(0));
}

#[test]
fn test_large_integer() {
    let result = scan_rust(
        r#"
fn pbkdf2(password: &[u8], iterations: u64) {}
fn main() {
    pbkdf2(password, 1000000000);
}
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(1000000000));
}

// =============================================================================
// String Literals
// =============================================================================

#[test]
fn test_string_literal() {
    let result = scan_rust(
        r#"
fn hash(algorithm: &str, data: &[u8]) {}
fn main() {
    hash("sha256", data);
}
"#,
    );
    assert_eq!(get_first_arg_string(&result, 0), Some("sha256".to_string()));
}

#[test]
fn test_raw_string_literal() {
    let result = scan_rust(
        r##"
fn encrypt(key: &str, data: &[u8]) {}
fn main() {
    encrypt(r#"raw-key-value"#, data);
}
"##,
    );
    assert_eq!(result.call_count(), 1);
}

#[test]
fn test_algorithm_strings() {
    for algo in ["sha256", "sha512", "aes", "rsa"] {
        let source = format!(
            r#"
fn hash(algorithm: &str) {{}}
fn main() {{
    hash("{}");
}}
"#,
            algo
        );
        let result = scan_rust(&source);
        assert_eq!(
            get_first_arg_string(&result, 0),
            Some(algo.to_string()),
            "Algorithm: {}",
            algo
        );
    }
}

// =============================================================================
// Multiple Calls
// =============================================================================

#[test]
fn test_multiple_calls() {
    let result = scan_rust(
        r#"
fn hash(data: &[u8], rounds: u32) {}
fn encrypt(data: &[u8], key_size: u32) {}
fn main() {
    hash(data, 1000);
    encrypt(data, 256);
}
"#,
    );
    assert_eq!(result.call_count(), 2);
    assert_eq!(get_first_arg_int(&result, 1), Some(1000));
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_common_key_sizes() {
    for (size, bits) in [(16, 128), (24, 192), (32, 256), (64, 512)] {
        let source = format!(
            r#"
fn cipher(key_size: usize) {{}}
fn main() {{
    cipher({});
}}
"#,
            size
        );
        let result = scan_rust(&source);
        assert_eq!(get_first_arg_int(&result, 0), Some(size), "{} bits", bits);
    }
}
