//! Exhaustive binary resolution tests
//!
//! These tests prove that binary expressions (arithmetic, bitwise, comparison, logical)
//! are correctly evaluated by the scanner. Binary expressions are common in crypto code
//! for calculations like `BASE_ITERATIONS + 10000` or `keySize * 8`.
//!
//! ## Structure
//! - `test_utils` - Shared test helpers
//! - `go` - Go binary tests
//! - `python` - Python binary tests
//! - `rust_lang` - Rust binary tests
//! - `javascript` - JavaScript binary tests
//! - `cross_language` - Cross-language consistency tests

pub mod test_utils;

pub mod cross_language;
pub mod go;
pub mod javascript;
pub mod python;
pub mod rust_lang;
