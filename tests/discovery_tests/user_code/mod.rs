//! User code discovery tests
//!
//! These tests verify that user code files are correctly discovered and loaded.
//!
//! ## Coverage
//! - Basic file discovery
//! - Excluded directories (testdata, .git)
//! - File metadata extraction
//! - Source type tagging
//!
//! ## Structure
//! - `test_utils` - Shared test helpers
//! - `go` - Go user code discovery tests
//! - (future) `python` - Python user code discovery tests
//! - (future) `javascript` - JavaScript user code discovery tests
//! - (future) `rust_lang` - Rust user code discovery tests
//! - (future) `cross_language` - Cross-language consistency tests

pub mod test_utils;

pub mod go;
