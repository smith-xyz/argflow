//! Crypto file filtering tests
//!
//! These tests verify that crypto file filtering correctly identifies files with crypto usage.
//!
//! ## Coverage
//! - Basic crypto import detection
//! - Filter accuracy (true positives and negatives)
//! - Integration with classifier-rules mappings
//!
//! ## Structure
//! - `test_utils` - Shared test helpers
//! - `go` - Go crypto filtering tests
//! - (future) `python` - Python crypto filtering tests
//! - (future) `javascript` - JavaScript crypto filtering tests
//! - (future) `rust_lang` - Rust crypto filtering tests
//! - (future) `cross_language` - Cross-language consistency tests

pub mod test_utils;

pub mod go;
