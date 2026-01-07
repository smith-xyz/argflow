//! Shared test utilities for call resolution tests

use argflow::scanner::{ScanResult, Scanner};
use argflow::Resolver;

use crate::fixtures;

fn scanner_with_patterns() -> Scanner {
    Scanner::with_resolver(Resolver::new()).with_patterns(fixtures::test_patterns())
}

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

pub fn scan_go(source: &str) -> ScanResult {
    let tree = parse_go(source);
    scanner_with_patterns().scan_tree(&tree, source.as_bytes(), "test.go", "go")
}

pub fn scan_python(source: &str) -> ScanResult {
    let tree = parse_python(source);
    scanner_with_patterns().scan_tree(&tree, source.as_bytes(), "test.py", "python")
}

pub fn scan_rust(source: &str) -> ScanResult {
    let tree = parse_rust(source);
    scanner_with_patterns().scan_tree(&tree, source.as_bytes(), "test.rs", "rust")
}

pub fn scan_javascript(source: &str) -> ScanResult {
    let tree = parse_javascript(source);
    scanner_with_patterns().scan_tree(&tree, source.as_bytes(), "test.js", "javascript")
}

pub fn get_first_arg_int(result: &ScanResult, arg_idx: usize) -> Option<i64> {
    result
        .calls
        .first()
        .and_then(|c| c.arguments.get(arg_idx))
        .filter(|a| a.is_resolved)
        .and_then(|a| a.int_values.first().copied())
}

pub fn get_first_arg_ints(result: &ScanResult, arg_idx: usize) -> Vec<i64> {
    result
        .calls
        .first()
        .and_then(|c| c.arguments.get(arg_idx))
        .map(|a| a.int_values.clone())
        .unwrap_or_default()
}

pub fn get_first_arg_string(result: &ScanResult, arg_idx: usize) -> Option<String> {
    result
        .calls
        .first()
        .and_then(|c| c.arguments.get(arg_idx))
        .filter(|a| a.is_resolved)
        .and_then(|a| a.string_values.first().cloned())
}

#[allow(dead_code)]
pub fn get_first_arg_strings(result: &ScanResult, arg_idx: usize) -> Vec<String> {
    result
        .calls
        .first()
        .and_then(|c| c.arguments.get(arg_idx))
        .map(|a| a.string_values.clone())
        .unwrap_or_default()
}

pub fn is_arg_resolved(result: &ScanResult, arg_idx: usize) -> bool {
    result
        .calls
        .first()
        .and_then(|c| c.arguments.get(arg_idx))
        .map(|a| a.is_resolved)
        .unwrap_or(false)
}

pub fn get_arg_source(result: &ScanResult, arg_idx: usize) -> Option<String> {
    result
        .calls
        .first()
        .and_then(|c| c.arguments.get(arg_idx))
        .filter(|a| !a.source.is_empty())
        .map(|a| a.source.clone())
}
