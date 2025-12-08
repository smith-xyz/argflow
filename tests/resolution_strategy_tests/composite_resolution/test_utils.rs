//! Shared test utilities for composite resolution tests

use crypto_extractor_core::scanner::{default_patterns, ScanResult, Scanner};

pub fn parse_go(source: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_go::LANGUAGE.into())
        .unwrap();
    parser.parse(source, None).unwrap()
}

pub fn parse_python(source: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .unwrap();
    parser.parse(source, None).unwrap()
}

pub fn parse_rust(source: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .unwrap();
    parser.parse(source, None).unwrap()
}

pub fn parse_javascript(source: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_javascript::LANGUAGE.into())
        .unwrap();
    parser.parse(source, None).unwrap()
}

fn create_scanner() -> Scanner {
    Scanner::new().with_patterns(default_patterns())
}

pub fn scan_go(source: &str) -> ScanResult {
    let tree = parse_go(source);
    create_scanner().scan_tree(&tree, source.as_bytes(), "test.go", "go")
}

pub fn scan_python(source: &str) -> ScanResult {
    let tree = parse_python(source);
    create_scanner().scan_tree(&tree, source.as_bytes(), "test.py", "python")
}

pub fn scan_rust(source: &str) -> ScanResult {
    let tree = parse_rust(source);
    create_scanner().scan_tree(&tree, source.as_bytes(), "test.rs", "rust")
}

pub fn scan_javascript(source: &str) -> ScanResult {
    let tree = parse_javascript(source);
    create_scanner().scan_tree(&tree, source.as_bytes(), "test.js", "javascript")
}

pub fn get_first_arg_ints(result: &ScanResult, arg_idx: usize) -> Vec<i64> {
    result
        .calls
        .first()
        .and_then(|c| c.arguments.get(arg_idx))
        .map(|a| a.int_values.clone())
        .unwrap_or_default()
}

fn is_arg_resolved(result: &ScanResult, arg_idx: usize) -> bool {
    result
        .calls
        .first()
        .and_then(|c| c.arguments.get(arg_idx))
        .map(|a| a.is_resolved)
        .unwrap_or(false)
}

pub fn is_arg_unresolved(result: &ScanResult, arg_idx: usize) -> bool {
    !is_arg_resolved(result, arg_idx)
}
