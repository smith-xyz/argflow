//! Shared test utilities for literal resolution tests

use crypto_extractor_core::scanner::{ScanResult, Scanner};

/// Parse Go source code into a tree-sitter Tree
pub fn parse_go(source: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_go::LANGUAGE.into())
        .unwrap();
    parser.parse(source, None).unwrap()
}

/// Parse Python source code into a tree-sitter Tree
pub fn parse_python(source: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .unwrap();
    parser.parse(source, None).unwrap()
}

/// Parse Rust source code into a tree-sitter Tree
pub fn parse_rust(source: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .unwrap();
    parser.parse(source, None).unwrap()
}

/// Parse JavaScript source code into a tree-sitter Tree
pub fn parse_javascript(source: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_javascript::LANGUAGE.into())
        .unwrap();
    parser.parse(source, None).unwrap()
}

/// Scan Go source code and return results
pub fn scan_go(source: &str) -> ScanResult {
    let tree = parse_go(source);
    Scanner::new().scan_tree(&tree, source.as_bytes(), "test.go", "go")
}

/// Scan Python source code and return results
pub fn scan_python(source: &str) -> ScanResult {
    let tree = parse_python(source);
    Scanner::new().scan_tree(&tree, source.as_bytes(), "test.py", "python")
}

/// Scan Rust source code and return results
pub fn scan_rust(source: &str) -> ScanResult {
    let tree = parse_rust(source);
    Scanner::new().scan_tree(&tree, source.as_bytes(), "test.rs", "rust")
}

/// Scan JavaScript source code and return results
pub fn scan_javascript(source: &str) -> ScanResult {
    let tree = parse_javascript(source);
    Scanner::new().scan_tree(&tree, source.as_bytes(), "test.js", "javascript")
}

/// Extract the first resolved integer argument at the given index
pub fn get_first_arg_int(result: &ScanResult, arg_idx: usize) -> Option<i64> {
    result
        .calls
        .first()
        .and_then(|c| c.arguments.get(arg_idx))
        .filter(|a| a.is_resolved)
        .and_then(|a| a.int_values.first().copied())
}

/// Extract the first resolved string argument at the given index
pub fn get_first_arg_string(result: &ScanResult, arg_idx: usize) -> Option<String> {
    result
        .calls
        .first()
        .and_then(|c| c.arguments.get(arg_idx))
        .filter(|a| a.is_resolved)
        .and_then(|a| a.string_values.first().cloned())
}

/// Check if an argument is unresolved
pub fn is_arg_unresolved(result: &ScanResult, arg_idx: usize) -> bool {
    result
        .calls
        .first()
        .and_then(|c| c.arguments.get(arg_idx))
        .map(|a| !a.is_resolved)
        .unwrap_or(true)
}
