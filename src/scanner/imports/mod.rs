//! Import tracking for accurate classification lookups.
//!
//! This module extracts import declarations from source files and builds
//! a mapping from short package names to full import paths.
//!
//! Language-specific extraction is delegated to sub-modules.

mod go;
mod python;

use std::collections::HashMap;
use tree_sitter::Tree;

#[derive(Debug, Clone, Default)]
pub struct ImportMap {
    imports: HashMap<String, String>,
}

impl ImportMap {
    pub fn new() -> Self {
        Self {
            imports: HashMap::new(),
        }
    }

    pub fn insert(&mut self, short_name: String, full_path: String) {
        self.imports.insert(short_name, full_path);
    }

    pub fn get(&self, short_name: &str) -> Option<&String> {
        self.imports.get(short_name)
    }

    pub fn resolve(&self, package: &str) -> Option<String> {
        self.imports.get(package).cloned()
    }

    pub fn len(&self) -> usize {
        self.imports.len()
    }

    pub fn is_empty(&self) -> bool {
        self.imports.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.imports.iter()
    }
}

pub fn extract_imports(tree: &Tree, source: &[u8], language: &str) -> ImportMap {
    match language {
        "go" => go::extract(tree, source),
        "python" | "py" => python::extract(tree, source),
        _ => ImportMap::new(),
    }
}

pub fn unquote_string(s: &str) -> String {
    let s = s.trim();
    let is_quoted = (s.starts_with('"') && s.ends_with('"'))
        || (s.starts_with('\'') && s.ends_with('\''))
        || (s.starts_with('`') && s.ends_with('`'));

    if is_quoted && s.len() >= 2 {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_map_basic() {
        let mut imports = ImportMap::new();
        imports.insert("sha256".to_string(), "crypto/sha256".to_string());

        assert_eq!(imports.len(), 1);
        assert_eq!(imports.get("sha256"), Some(&"crypto/sha256".to_string()));
        assert_eq!(imports.resolve("sha256"), Some("crypto/sha256".to_string()));
    }

    #[test]
    fn test_import_map_not_found() {
        let imports = ImportMap::new();
        assert_eq!(imports.get("nonexistent"), None);
        assert_eq!(imports.resolve("nonexistent"), None);
    }

    #[test]
    fn test_unquote_string() {
        assert_eq!(unquote_string("\"hello\""), "hello");
        assert_eq!(unquote_string("'hello'"), "hello");
        assert_eq!(unquote_string("`hello`"), "hello");
        assert_eq!(unquote_string("hello"), "hello");
    }
}
