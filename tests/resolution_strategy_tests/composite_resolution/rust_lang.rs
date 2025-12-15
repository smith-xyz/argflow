//! Rust composite resolution tests

use super::test_utils::scan_rust;

// =============================================================================
// Array Literals
// =============================================================================

#[test]
fn test_array_literal() {
    let result = scan_rust(
        r#"
use pbkdf2::pbkdf2_hmac;
fn main() {
    let salt = [0x01u8, 0x02, 0x03];
    pbkdf2_hmac::<Sha256>(password, &salt, 100000, &mut key);
}
"#,
    );
    assert!(!result.calls.is_empty());
}

#[test]
fn test_array_with_size() {
    let result = scan_rust(
        r#"
use pbkdf2::pbkdf2_hmac;
fn main() {
    let sizes: [u8; 3] = [16, 24, 32];
    pbkdf2_hmac::<Sha256>(password, salt, 100000, &mut key);
}
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Vec Literals (vec! macro)
// =============================================================================

#[test]
fn test_vec_macro() {
    let result = scan_rust(
        r#"
use pbkdf2::pbkdf2_hmac;
fn main() {
    let salt = vec![0xDE, 0xAD, 0xBE, 0xEF];
    pbkdf2_hmac::<Sha256>(password, &salt, 100000, &mut key);
}
"#,
    );
    assert!(!result.calls.is_empty());
}

#[test]
fn test_vec_with_capacity() {
    let result = scan_rust(
        r#"
use pbkdf2::pbkdf2_hmac;
fn main() {
    let mut salt = Vec::with_capacity(16);
    salt.extend_from_slice(&[0x01, 0x02, 0x03]);
    pbkdf2_hmac::<Sha256>(password, &salt, 100000, &mut key);
}
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Struct Literals
// =============================================================================

#[test]
fn test_struct_literal() {
    let result = scan_rust(
        r#"
use pbkdf2::pbkdf2_hmac;
struct Config {
    iterations: u32,
    key_len: usize,
}
fn main() {
    let cfg = Config { iterations: 100000, key_len: 32 };
    pbkdf2_hmac::<Sha256>(password, salt, cfg.iterations, &mut key);
}
"#,
    );
    assert!(!result.calls.is_empty());
}

#[test]
fn test_struct_field_shorthand() {
    let result = scan_rust(
        r#"
use pbkdf2::pbkdf2_hmac;
struct Config {
    iterations: u32,
}
fn main() {
    let iterations = 100000u32;
    let cfg = Config { iterations };
    pbkdf2_hmac::<Sha256>(password, salt, cfg.iterations, &mut key);
}
"#,
    );
    assert!(!result.calls.is_empty());
}

#[test]
fn test_struct_update_syntax() {
    let result = scan_rust(
        r#"
use pbkdf2::pbkdf2_hmac;
struct Config {
    iterations: u32,
    key_len: usize,
}
fn main() {
    let default = Config { iterations: 10000, key_len: 16 };
    let cfg = Config { iterations: 100000, ..default };
    pbkdf2_hmac::<Sha256>(password, salt, cfg.iterations, &mut key);
}
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Tuple Literals
// =============================================================================

#[test]
fn test_tuple_literal() {
    let result = scan_rust(
        r#"
use pbkdf2::pbkdf2_hmac;
fn main() {
    let params = (100000u32, 32usize);
    pbkdf2_hmac::<Sha256>(password, salt, params.0, &mut key);
}
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Slice Patterns
// =============================================================================

#[test]
fn test_slice_from_array() {
    let result = scan_rust(
        r#"
use pbkdf2::pbkdf2_hmac;
fn main() {
    let arr = [0x01u8, 0x02, 0x03, 0x04];
    let salt = &arr[..];
    pbkdf2_hmac::<Sha256>(password, salt, 100000, &mut key);
}
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Const Arrays
// =============================================================================

#[test]
fn test_const_array() {
    let result = scan_rust(
        r#"
use pbkdf2::pbkdf2_hmac;
const SALT: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
fn main() {
    pbkdf2_hmac::<Sha256>(password, &SALT, 100000, &mut key);
}
"#,
    );
    assert!(!result.calls.is_empty());
}

#[test]
fn test_static_array() {
    let result = scan_rust(
        r#"
use pbkdf2::pbkdf2_hmac;
static ITERATIONS: [u32; 3] = [10000, 50000, 100000];
fn main() {
    pbkdf2_hmac::<Sha256>(password, salt, ITERATIONS[2], &mut key);
}
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Empty Collections
// =============================================================================

#[test]
fn test_empty_array() {
    let result = scan_rust(
        r#"
use pbkdf2::pbkdf2_hmac;
fn main() {
    let empty: [u8; 0] = [];
    pbkdf2_hmac::<Sha256>(password, &empty, 100000, &mut key);
}
"#,
    );
    assert!(!result.calls.is_empty());
}

#[test]
fn test_empty_vec() {
    let result = scan_rust(
        r#"
use pbkdf2::pbkdf2_hmac;
fn main() {
    let empty: Vec<u8> = vec![];
    pbkdf2_hmac::<Sha256>(password, &empty, 100000, &mut key);
}
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Crypto-Relevant Scenarios
// =============================================================================

#[test]
fn test_key_sizes_array() {
    let result = scan_rust(
        r#"
use aes::cipher::KeySizeUser;
fn main() {
    let key_sizes = [16usize, 24, 32];
    let key_len = key_sizes[2];
}
"#,
    );
    // This doesn't call a crypto function directly - any result is valid
    let _ = result.calls;
}

#[test]
fn test_cipher_config_struct() {
    let result = scan_rust(
        r#"
use aes_gcm::Aes256Gcm;
struct CipherConfig {
    key_size: usize,
    nonce_size: usize,
    tag_size: usize,
}
fn main() {
    let config = CipherConfig {
        key_size: 32,
        nonce_size: 12,
        tag_size: 16,
    };
}
"#,
    );
    // Struct definition doesn't call crypto functions - any result is valid
    let _ = result.calls;
}

// =============================================================================
// Repeat Expressions
// =============================================================================

#[test]
fn test_repeat_expression() {
    let result = scan_rust(
        r#"
use pbkdf2::pbkdf2_hmac;
fn main() {
    let zeros = [0u8; 16];
    pbkdf2_hmac::<Sha256>(password, &zeros, 100000, &mut key);
}
"#,
    );
    assert!(!result.calls.is_empty());
}

// =============================================================================
// Range Expressions in Arrays
// =============================================================================

#[test]
fn test_array_range_index() {
    let result = scan_rust(
        r#"
use pbkdf2::pbkdf2_hmac;
fn main() {
    let data = [0u8, 1, 2, 3, 4, 5, 6, 7];
    let salt = &data[0..4];
    pbkdf2_hmac::<Sha256>(password, salt, 100000, &mut key);
}
"#,
    );
    assert!(!result.calls.is_empty());
}
