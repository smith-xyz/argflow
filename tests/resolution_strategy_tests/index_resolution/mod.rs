//! Exhaustive index resolution tests
//!
//! These tests prove that index expressions (array access, map access, dict access)
//! are correctly evaluated by the scanner. Index expressions are common in crypto code
//! for accessing configured values like `keySizes[0]` or `config["algorithm"]`.
//!
//! ## Structure
//! - `test_utils` - Shared test helpers
//! - `go` - Go index tests
//! - `python` - Python index tests
//! - `rust_lang` - Rust index tests
//! - `javascript` - JavaScript index tests
//! - `cross_language` - Cross-language consistency tests

pub mod test_utils;

pub mod cross_language;
pub mod go;
pub mod javascript;
pub mod python;
pub mod rust_lang;
