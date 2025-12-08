//! Exhaustive selector resolution tests
//!
//! These tests prove that selector expressions (field access, package qualifiers)
//! are correctly handled by the scanner. Selector expressions are common in crypto code
//! for accessing constants like `crypto.DefaultIterations` or config fields like `cfg.KeySize`.
//!
//! ## Structure
//! - `test_utils` - Shared test helpers
//! - `go` - Go selector tests
//! - `python` - Python selector tests
//! - `rust_lang` - Rust selector tests
//! - `javascript` - JavaScript selector tests
//! - `cross_language` - Cross-language consistency tests

pub mod test_utils;

pub mod cross_language;
pub mod go;
pub mod javascript;
pub mod python;
pub mod rust_lang;
