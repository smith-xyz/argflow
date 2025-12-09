//! Integration tests for discovery workflow
//!
//! These tests verify end-to-end discovery workflows combining user code,
//! dependencies, filtering, and source type tagging.
//!
//! ## Coverage
//! - Full workflow (discover -> filter -> scan)
//! - Combined user code + dependencies
//! - Crypto file identification across sources
//!
//! ## Structure
//! - `test_utils` - Shared test helpers
//! - `go` - Go integration tests
//! - (future) `python` - Python integration tests
//! - (future) `javascript` - JavaScript integration tests
//! - (future) `rust_lang` - Rust integration tests
//! - (future) `cross_language` - Cross-language consistency tests

pub mod test_utils;

pub mod go;
pub mod javascript;
pub mod python;
pub mod rust;
