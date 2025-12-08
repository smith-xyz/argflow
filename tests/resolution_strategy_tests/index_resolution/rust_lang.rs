//! Rust index resolution tests

use super::test_utils::{get_arg_expression, get_first_arg_int, is_arg_unresolved, scan_rust};

// =============================================================================
// Array Integer Index Tests
// =============================================================================

#[test]
fn test_array_index_first() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(alg, [10000u32, 20000, 30000][0], &salt, &password, &mut key);
}
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(10000), "First element");
}

#[test]
fn test_array_index_middle() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(alg, [10000u32, 20000, 30000][1], &salt, &password, &mut key);
}
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(20000), "Middle element");
}

#[test]
fn test_array_index_last() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(alg, [10000u32, 20000, 30000][2], &salt, &password, &mut key);
}
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(30000), "Last element");
}

#[test]
fn test_array_single_element() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(alg, [100000u32][0], &salt, &password, &mut key);
}
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 1),
        Some(100000),
        "Single element"
    );
}

// =============================================================================
// Key Size Tests (Crypto-relevant)
// =============================================================================

#[test]
fn test_key_sizes_array_128() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    let key_len = [16usize, 24, 32][0];
    pbkdf2::derive(alg, 10000, &salt, &password, &mut key);
}
"#,
    );
    // This tests that the scanner detects the call
    assert!(result.calls.len() >= 1, "Call detected");
}

#[test]
fn test_key_sizes_direct_index() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(alg, 10000, &[16usize, 24, 32][2], &password, &mut key);
}
"#,
    );
    // Array as part of slice reference
    assert!(result.calls.len() >= 1, "Call detected");
}

// =============================================================================
// Out of Bounds Tests
// =============================================================================

#[test]
fn test_index_out_of_bounds() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(alg, [10000u32, 20000][5], &salt, &password, &mut key);
}
"#,
    );
    assert!(is_arg_unresolved(&result, 1), "Out of bounds index");
}

// =============================================================================
// Non-literal Index Tests
// =============================================================================

#[test]
fn test_variable_index() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    let i = 0;
    pbkdf2::derive(alg, [10000u32, 20000][i], &salt, &password, &mut key);
}
"#,
    );
    assert!(
        is_arg_unresolved(&result, 1),
        "Variable index needs identifier resolution"
    );
}

// =============================================================================
// Non-literal Array Tests
// =============================================================================

#[test]
fn test_variable_array() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    let iterations = [10000u32, 20000, 30000];
    pbkdf2::derive(alg, iterations[0], &salt, &password, &mut key);
}
"#,
    );
    assert!(
        is_arg_unresolved(&result, 1),
        "Variable array needs identifier resolution"
    );
    assert_eq!(
        get_arg_expression(&result, 1),
        Some("iterations[0]".to_string()),
        "Expression preserved"
    );
}

// =============================================================================
// Typed Integer Tests
// =============================================================================

#[test]
fn test_typed_integers_u32() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(alg, [100000_u32, 200000_u32][0], &salt, &password, &mut key);
}
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(100000), "u32 suffix");
}

#[test]
fn test_typed_integers_i64() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(alg, [100000_i64, 200000_i64][1], &salt, &password, &mut key);
}
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(200000), "i64 suffix");
}

// =============================================================================
// Empty Array Tests
// =============================================================================

#[test]
fn test_empty_array_index() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    let arr: [u32; 0] = [];
    pbkdf2::derive(alg, arr[0], &salt, &password, &mut key);
}
"#,
    );
    // This would be a variable array, so it's unresolved
    assert!(is_arg_unresolved(&result, 1), "Variable empty array");
}

// =============================================================================
// Hex Value Tests
// =============================================================================

#[test]
fn test_hex_values_in_array() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(alg, [0x10u32, 0x18, 0x20][0], &salt, &password, &mut key);
}
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(16), "Hex 0x10 = 16");
}

#[test]
fn test_hex_values_in_array_last() {
    let result = scan_rust(
        r#"
use ring::pbkdf2;
fn main() {
    pbkdf2::derive(alg, [0x10u32, 0x18, 0x20][2], &salt, &password, &mut key);
}
"#,
    );
    assert_eq!(get_first_arg_int(&result, 1), Some(32), "Hex 0x20 = 32");
}
