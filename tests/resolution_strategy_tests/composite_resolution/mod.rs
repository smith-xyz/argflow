//! Exhaustive composite resolution tests
//!
//! These tests prove that array and struct literals are correctly parsed and resolved
//! by the scanner across supported languages.
//!
//! ## Coverage
//! - Array/slice literals ([1, 2, 3], []int{1, 2, 3})
//! - Struct/object literals (Config{...}, {key: value})
//! - Tuple literals (Python)
//! - Dictionary literals (Python)
//! - Mixed resolved/unresolved elements
//! - Nested composite literals
//!
//! ## Structure
//! - `test_utils` - Shared test helpers
//! - `go` - Go composite tests
//! - `python` - Python composite tests
//! - `rust_lang` - Rust composite tests
//! - `javascript` - JavaScript composite tests
//! - `cross_language` - Cross-language consistency tests

pub mod test_utils;

pub mod cross_language;
pub mod go;
pub mod javascript;
pub mod python;
pub mod rust_lang;
