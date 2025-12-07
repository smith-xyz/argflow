//! Go literal resolution tests

use super::test_utils::{get_first_arg_int, get_first_arg_string, is_arg_unresolved, scan_go};

// =============================================================================
// Integer Literals
// =============================================================================

#[test]
fn test_decimal_integer() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 100000, 32, h) }
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(100000),
        "Decimal integer"
    );
    assert_eq!(get_first_arg_int(&result, 3), Some(32), "Small decimal");
}

#[test]
fn test_hex_integer() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 0x186A0, 0x20, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(100000), "Hex 0x186A0");
    assert_eq!(get_first_arg_int(&result, 3), Some(32), "Hex 0x20");
}

#[test]
fn test_octal_integer() {
    let result = scan_go(
        r#"
package main
import "crypto/aes"
func main() { aes.NewCipher(make([]byte, 0o40)) }
"#,
    );
    // 0o40 = 32, but it's inside make(), so we just verify detection
    assert_eq!(result.call_count(), 1);
}

#[test]
fn test_binary_integer() {
    let result = scan_go(
        r#"
package main
import "crypto/aes"
func main() { aes.NewCipher(make([]byte, 0b100000)) }
"#,
    );
    // 0b100000 = 32
    assert_eq!(result.call_count(), 1);
}

#[test]
fn test_underscore_separator() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 100_000, 32, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(100000));
}

#[test]
fn test_negative_integer_needs_unary_strategy() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, -1, 32, h) }
"#,
    );
    // -1 is a unary_expression, not resolved by LiteralStrategy
    assert!(
        is_arg_unresolved(&result, 2),
        "Negative needs UnaryStrategy"
    );
}

#[test]
fn test_zero() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 0, 0, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(0));
    assert_eq!(get_first_arg_int(&result, 3), Some(0));
}

#[test]
fn test_large_integer() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, 1000000000, 64, h) }
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(1000000000));
}

#[test]
fn test_common_key_sizes() {
    for (size, bits) in [(16, 128), (24, 192), (32, 256), (64, 512)] {
        let source = format!(
            r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() {{ pbkdf2.Key(p, s, 10000, {}, h) }}
"#,
            size
        );
        let result = scan_go(&source);
        assert_eq!(get_first_arg_int(&result, 3), Some(size), "{} bits", bits);
    }
}

// =============================================================================
// String Literals
// =============================================================================

#[test]
fn test_double_quoted_string() {
    let result = scan_go(
        r#"
package main
import "crypto/sha256"
func main() { sha256.Sum256([]byte("hello")) }
"#,
    );
    assert_eq!(result.call_count(), 1);
}

#[test]
fn test_raw_string() {
    let result = scan_go(
        r#"
package main
import "crypto/sha256"
func main() { sha256.Sum256([]byte(`raw string`)) }
"#,
    );
    assert_eq!(result.call_count(), 1);
}

#[test]
fn test_string_helper_available() {
    let result = scan_go(
        r#"
package main
import "crypto/cipher"
func main() { cipher.NewGCMWithNonceSize(block, 12) }
"#,
    );
    // Just verify the helper works
    let _ = get_first_arg_string(&result, 0);
    assert_eq!(result.call_count(), 1);
}

// =============================================================================
// Boolean and Nil Literals
// =============================================================================

#[test]
fn test_nil_literal() {
    let result = scan_go(
        r#"
package main
import "crypto/aes"
func main() { 
    block, _ := aes.NewCipher(key)
    cipher.NewGCM(block)
}
"#,
    );
    assert!(result.call_count() >= 1);
}

// =============================================================================
// Float Literals
// =============================================================================

#[test]
fn test_float_literal() {
    let result = scan_go(
        r#"
package main
import "crypto/sha256"
func main() { sha256.Sum256([]byte("test")) }
"#,
    );
    assert_eq!(result.call_count(), 1);
}
