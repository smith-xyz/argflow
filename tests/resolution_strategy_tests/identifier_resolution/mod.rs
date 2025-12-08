//! Exhaustive identifier resolution tests
//!
//! These tests prove that all identifier patterns are correctly resolved
//! by the scanner across supported languages.
//!
//! ## Coverage
//! - Local variable resolution (within function scope)
//! - File-level constant resolution
//! - Function parameter detection (marked as unresolved)
//! - Variable shadowing
//! - Variable reassignment
//! - String variable resolution
//!
//! ## Structure
//! - `test_utils` - Shared test helpers
//! - `go` - Go identifier tests
//! - `python` - Python identifier tests
//! - `rust_lang` - Rust identifier tests
//! - `javascript` - JavaScript identifier tests
//! - `cross_language` - Cross-language consistency tests

pub mod test_utils;

pub mod cross_language;
pub mod go;
pub mod javascript;
pub mod python;
pub mod rust_lang;
