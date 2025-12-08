//! Discovery Test Suite
//!
//! Tests for file discovery, dependency loading, and crypto filtering across supported languages.
//!
//! ## Structure
//! - `user_code` - Tests for user code discovery
//! - `dependencies` - Tests for dependency discovery (third-party + stdlib)
//! - `filtering` - Tests for crypto file filtering
//! - `source_types` - Tests for source type tagging (UserCode, Dependency, Stdlib)
//! - `cache` - Tests for discovery cache functionality
//! - `integration` - End-to-end workflow tests

pub mod cache;
pub mod dependencies;
pub mod filtering;
pub mod integration;
pub mod source_types;
pub mod user_code;
