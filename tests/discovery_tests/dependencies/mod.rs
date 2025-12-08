//! Dependency discovery tests
//!
//! These tests verify that third-party dependencies and stdlib files are correctly discovered.
//!
//! ## Coverage
//! - Third-party dependency discovery
//! - Stdlib file discovery
//! - Vendor directory handling
//! - Dependency source type tagging
//!
//! ## Structure
//! - `test_utils` - Shared test helpers
//! - `go` - Go dependency discovery tests
//! - (future) `python` - Python dependency discovery tests
//! - (future) `javascript` - JavaScript dependency discovery tests
//! - (future) `rust_lang` - Rust dependency discovery tests
//! - (future) `cross_language` - Cross-language consistency tests

pub mod test_utils;

pub mod go;
