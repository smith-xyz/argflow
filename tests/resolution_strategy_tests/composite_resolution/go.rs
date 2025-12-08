//! Go composite resolution tests

use super::test_utils::{get_first_arg_ints, is_arg_unresolved, scan_go};

// =============================================================================
// Array/Slice Literals as Arguments
// =============================================================================

#[test]
fn test_byte_slice_literal() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key([]byte{0x01, 0x02, 0x03}, s, 10000, 32, h) }
"#,
    );
    let ints = get_first_arg_ints(&result, 0);
    assert!(ints.contains(&1));
    assert!(ints.contains(&2));
    assert!(ints.contains(&3));
}

#[test]
fn test_int_slice_literal() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 10000, []int{16, 24, 32}[idx], h) }
"#,
    );
    // When indexed, the array itself isn't the resolved value
    assert!(is_arg_unresolved(&result, 3));
}

#[test]
fn test_string_slice_literal() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
var algs = []string{"sha256", "sha384", "sha512"}
func main() { pbkdf2.Key(p, s, 10000, 32, algs[0]) }
"#,
    );
    // Indexed access isn't fully resolved
    assert!(is_arg_unresolved(&result, 4));
}

#[test]
fn test_empty_slice_literal() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key([]byte{}, s, 10000, 32, h) }
"#,
    );
    // Empty array should still be detected
    assert!(result.calls.len() == 1);
}

// =============================================================================
// Hex Values in Byte Slices
// =============================================================================

#[test]
fn test_hex_byte_values() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key([]byte{0xFF, 0xAB, 0x12}, s, 10000, 32, h) }
"#,
    );
    let ints = get_first_arg_ints(&result, 0);
    assert!(ints.contains(&255));
    assert!(ints.contains(&171));
    assert!(ints.contains(&18));
}

// =============================================================================
// Mixed Resolved/Unresolved Elements
// =============================================================================

#[test]
fn test_mixed_literal_and_variable() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
var x byte = 0x42
func main() { pbkdf2.Key([]byte{0x01, x, 0x03}, s, 10000, 32, h) }
"#,
    );
    let ints = get_first_arg_ints(&result, 0);
    // Should resolve literals, x may or may not be resolved
    assert!(ints.contains(&1) || ints.contains(&3));
}

// =============================================================================
// Composite Literals with Type
// =============================================================================

#[test]
fn test_typed_array_literal() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key([3]byte{1, 2, 3}[:], s, 10000, 32, h) }
"#,
    );
    // Sliced array
    assert!(result.calls.len() == 1);
}

// =============================================================================
// Nested Composite Literals
// =============================================================================

#[test]
fn test_nested_array_literal() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { 
    keys := [][]byte{{0x01, 0x02}, {0x03, 0x04}}
    pbkdf2.Key(keys[0], s, 10000, 32, h) 
}
"#,
    );
    // Indexed access of nested array
    assert!(is_arg_unresolved(&result, 0));
}

// =============================================================================
// Struct Literals (limited - usually detected differently)
// =============================================================================

#[test]
fn test_struct_literal_expression() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
type Config struct { Iterations int }
func main() { pbkdf2.Key(p, s, Config{Iterations: 10000}.Iterations, 32, h) }
"#,
    );
    // Struct field access on literal
    assert!(result.calls.len() == 1);
}

// =============================================================================
// Crypto-Relevant Scenarios
// =============================================================================

#[test]
fn test_cipher_suite_array() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
var suites = []uint16{0x1301, 0x1302, 0x1303}
func main() { pbkdf2.Key(p, s, int(suites[0]), 32, h) }
"#,
    );
    // Type conversion on indexed array
    assert!(result.calls.len() == 1);
}

#[test]
fn test_salt_array_literal() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { 
    salt := []byte{0xDE, 0xAD, 0xBE, 0xEF}
    pbkdf2.Key(password, salt, 10000, 32, h) 
}
"#,
    );
    let ints = get_first_arg_ints(&result, 1);
    assert!(ints.contains(&0xDE));
    assert!(ints.contains(&0xAD));
    assert!(ints.contains(&0xBE));
    assert!(ints.contains(&0xEF));
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_single_element_array() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key([]byte{42}, s, 10000, 32, h) }
"#,
    );
    let ints = get_first_arg_ints(&result, 0);
    assert!(ints.contains(&42));
}

#[test]
fn test_trailing_comma() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key([]byte{1, 2, 3,}, s, 10000, 32, h) }
"#,
    );
    let ints = get_first_arg_ints(&result, 0);
    assert_eq!(ints.len(), 3);
}

#[test]
fn test_multiline_array() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { 
    pbkdf2.Key([]byte{
        0x01,
        0x02,
        0x03,
    }, s, 10000, 32, h) 
}
"#,
    );
    let ints = get_first_arg_ints(&result, 0);
    assert_eq!(ints.len(), 3);
}
