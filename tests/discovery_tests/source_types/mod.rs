//! Source type tagging tests
//!
//! These tests verify that files are correctly tagged with their source type
//! (UserCode, Dependency, Stdlib).
//!
//! ## Coverage
//! - User code tagging
//! - Dependency tagging
//! - Stdlib tagging
//! - Tagging consistency
//!
//! ## Structure
//! - `test_utils` - Shared test helpers
//! - `go` - Go source type tagging tests
//! - (future) `python` - Python source type tagging tests
//! - (future) `javascript` - JavaScript source type tagging tests
//! - (future) `rust_lang` - Rust source type tagging tests
//! - (future) `cross_language` - Cross-language consistency tests

pub mod test_utils;

pub mod go;
