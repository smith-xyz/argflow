//! Rust-specific call resolution tests

use super::test_utils::*;

#[test]
fn test_rust_simple_int_return() {
    let source = r#"
use ring::pbkdf2;

fn get_iterations() -> u32 {
    return 10000;
}

fn main() {
    pbkdf2::derive(get_iterations());
}
"#;
    let result = scan_rust(source);

    assert_eq!(result.calls.len(), 1);
    assert!(is_arg_resolved(&result, 0));
    assert_eq!(get_first_arg_int(&result, 0), Some(10000));
}

#[test]
fn test_rust_simple_string_return() {
    let source = r#"
use ring::digest;

fn get_algorithm() -> &'static str {
    return "sha256";
}

fn main() {
    digest::digest(get_algorithm());
}
"#;
    let result = scan_rust(source);

    assert_eq!(result.calls.len(), 1);
    assert!(is_arg_resolved(&result, 0));
    assert_eq!(get_first_arg_string(&result, 0), Some("sha256".to_string()));
}

#[test]
fn test_rust_if_else_returns() {
    let source = r#"
use ring::pbkdf2;

fn get_key_size(aes256: bool) -> u32 {
    if aes256 {
        return 32;
    }
    return 16;
}

fn main() {
    pbkdf2::derive(get_key_size(true));
}
"#;
    let result = scan_rust(source);

    assert_eq!(result.calls.len(), 1);
    let key_sizes = get_first_arg_ints(&result, 0);
    assert!(key_sizes.contains(&32));
    assert!(key_sizes.contains(&16));
}

#[test]
fn test_rust_match_returns() {
    let source = r#"
use ring::pbkdf2;

fn get_block_size(mode: &str) -> u32 {
    match mode {
        "AES-128" => return 16,
        "AES-192" => return 24,
        "AES-256" => return 32,
        _ => return 16,
    }
}

fn main() {
    pbkdf2::derive(get_block_size("AES-256"));
}
"#;
    let result = scan_rust(source);

    assert_eq!(result.calls.len(), 1);
    let sizes = get_first_arg_ints(&result, 0);
    assert!(sizes.contains(&16));
    assert!(sizes.contains(&24));
    assert!(sizes.contains(&32));
}

#[test]
fn test_rust_function_not_found() {
    let source = r#"
use ring::pbkdf2;

fn main() {
    pbkdf2::derive(unknown_function());
}
"#;
    let result = scan_rust(source);

    assert_eq!(result.calls.len(), 1);
    assert!(!is_arg_resolved(&result, 0));
    assert_eq!(
        get_arg_source(&result, 0),
        Some("function_not_found".to_string())
    );
}
