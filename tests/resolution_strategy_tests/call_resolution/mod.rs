//! Call resolution tests
//!
//! These tests prove that function call return values are correctly resolved
//! by the scanner across supported languages.
//!
//! ## Coverage
//! - Simple return value resolution
//! - Tuple/multi-return resolution (Go, Python)
//! - Control flow enumeration (if/else, switch)
//! - Unresolvable return detection
//!
//! ## Structure
//! - `test_utils` - Shared test helpers
//! - `go` - Go call resolution tests
//! - `python` - Python call resolution tests
//! - `rust_lang` - Rust call resolution tests
//! - `javascript` - JavaScript call resolution tests
//! - `cross_language` - Cross-language consistency tests

pub mod test_utils;

pub mod cross_language;
pub mod go;
pub mod javascript;
pub mod python;
pub mod rust_lang;
