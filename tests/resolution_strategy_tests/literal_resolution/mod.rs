//! Exhaustive literal resolution tests
//!
//! These tests prove that ALL literal types are correctly parsed and resolved
//! by the scanner without any extra configuration. This is the foundation
//! for the "human equivalence" goal - if a human can read a literal value,
//! so can this tool.
//!
//! ## Structure
//! - `test_utils` - Shared test helpers
//! - `go` - Go literal tests
//! - `python` - Python literal tests
//! - `rust_lang` - Rust literal tests
//! - `javascript` - JavaScript literal tests
//! - `cross_language` - Cross-language consistency tests

pub mod test_utils;

pub mod cross_language;
pub mod go;
pub mod javascript;
pub mod python;
pub mod rust_lang;
