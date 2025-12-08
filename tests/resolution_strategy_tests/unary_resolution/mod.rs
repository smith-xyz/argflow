//! Exhaustive unary resolution tests
//!
//! These tests prove that all unary operators are correctly parsed and resolved
//! by the scanner across supported languages.
//!
//! ## Coverage
//! - Negation operator (-x)
//! - Logical NOT operator (!x, not x)
//! - Bitwise NOT operator (~x, ^x)
//! - Positive operator (+x)
//! - Reference/dereference operators (&x, *x)
//!
//! ## Structure
//! - `test_utils` - Shared test helpers
//! - `go` - Go unary tests
//! - `python` - Python unary tests
//! - `rust_lang` - Rust unary tests
//! - `javascript` - JavaScript unary tests
//! - `cross_language` - Cross-language consistency tests

pub mod test_utils;

pub mod cross_language;
pub mod go;
pub mod javascript;
pub mod python;
pub mod rust_lang;
