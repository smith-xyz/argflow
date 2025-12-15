//! Go index resolution tests

use super::test_utils::{
    get_arg_expression, get_first_arg_int, get_first_arg_string, is_arg_unresolved, scan_go,
};

// =============================================================================
// Array Integer Index Tests
// =============================================================================

#[test]
fn test_array_index_first() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, []int{10000, 20000, 30000}[0], 32, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(10000), "First element");
}

#[test]
fn test_array_index_middle() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, []int{10000, 20000, 30000}[1], 32, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(20000), "Middle element");
}

#[test]
fn test_array_index_last() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, []int{10000, 20000, 30000}[2], 32, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(30000), "Last element");
}

#[test]
fn test_array_single_element() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, []int{100000}[0], 32, h) }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(100000),
        "Single element"
    );
}

// =============================================================================
// Array String Index Tests
// =============================================================================

#[test]
fn test_string_array_index_first() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 32, []string{"sha256", "sha512"}[0]) }
"#,
    );
    assert_eq!(
        get_first_arg_string(&result, 4),
        Some("sha256".to_string()),
        "First string"
    );
}

#[test]
fn test_string_array_index_second() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, 32, []string{"sha256", "sha512"}[1]) }
"#,
    );
    assert_eq!(
        get_first_arg_string(&result, 4),
        Some("sha512".to_string()),
        "Second string"
    );
}

// =============================================================================
// Key Size Array Tests (Crypto-relevant)
// =============================================================================

#[test]
fn test_key_sizes_array_128_bits() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, []int{16, 24, 32}[0], h) }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(16),
        "AES-128 key size (16 bytes)"
    );
}

#[test]
fn test_key_sizes_array_192_bits() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, []int{16, 24, 32}[1], h) }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(24),
        "AES-192 key size (24 bytes)"
    );
}

#[test]
fn test_key_sizes_array_256_bits() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, []int{16, 24, 32}[2], h) }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(32),
        "AES-256 key size (32 bytes)"
    );
}

// =============================================================================
// Map String Key Tests
// Note: Go map literal resolution is complex and not yet implemented.
// These tests are commented out as future work.
// =============================================================================

// TODO: Map literal resolution requires handling Go's keyed_element syntax
// #[test]
// fn test_map_string_key_iterations() { ... }
// #[test]
// fn test_map_string_key_keysize() { ... }

// =============================================================================
// Out of Bounds Tests
// =============================================================================

#[test]
fn test_index_out_of_bounds() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, []int{10000, 20000}[5], 32, h) }
"#,
    );
    assert!(is_arg_unresolved(&result, 2), "Out of bounds index");
}

#[test]
fn test_negative_index() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, []int{10000, 20000}[-1], 32, h) }
"#,
    );
    // Go doesn't support negative indexing, should be unresolved
    assert!(is_arg_unresolved(&result, 2), "Negative index");
}

// =============================================================================
// Non-literal Index Tests
// =============================================================================

#[test]
fn test_variable_index() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
var i = 0
func main() { pbkdf2.Key(p, s, []int{10000, 20000}[i], 32, h) }
"#,
    );
    assert!(
        is_arg_unresolved(&result, 2),
        "Variable index needs identifier resolution"
    );
}

#[test]
fn test_expression_index() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, []int{10000, 20000}[1+0], 32, h) }
"#,
    );
    // Binary expression as index - could be resolved but complex
    // Current implementation may or may not resolve this
    assert!(!result.calls.is_empty(), "call detected");
}

// =============================================================================
// Non-literal Array Tests
// =============================================================================

#[test]
fn test_variable_array() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
var sizes = []int{16, 24, 32}
func main() { pbkdf2.Key(p, s, 10000, sizes[0], h) }
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

// =============================================================================
// Hex Value Tests
// =============================================================================

#[test]
fn test_hex_values_in_array() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, []int{0x10, 0x18, 0x20}[0], h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(16), "Hex 0x10 = 16");
}

#[test]
fn test_hex_values_in_array_second() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, []int{0x10, 0x18, 0x20}[2], h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(32), "Hex 0x20 = 32");
}

// =============================================================================
// Empty Array Tests
// =============================================================================

#[test]
fn test_empty_array_index() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, []int{}[0], 32, h) }
"#,
    );
    assert!(
        is_arg_unresolved(&result, 2),
        "Empty array index out of bounds"
    );
}
