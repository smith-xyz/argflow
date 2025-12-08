use crate::error::QueryError;
use std::collections::HashMap;
use tracing::{trace, warn};
use tree_sitter::{Language, Node, Query, QueryCursor, StreamingIterator};

#[derive(Debug, Clone)]
pub struct Capture {
    pub name: String,
    pub text: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_row: usize,
    pub start_col: usize,
}

#[derive(Debug, Clone)]
pub struct Match {
    pub captures: HashMap<String, Capture>,
}

impl Match {
    pub fn get(&self, name: &str) -> Option<&str> {
        self.captures.get(name).map(|c| c.text.as_str())
    }

    pub fn get_capture(&self, name: &str) -> Option<&Capture> {
        self.captures.get(name)
    }
}

pub struct QueryEngine {
    queries: HashMap<String, HashMap<String, Query>>,
}

impl QueryEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            queries: HashMap::new(),
        };

        engine.load_go_queries();
        engine.load_python_queries();
        engine.load_rust_queries();
        engine.load_javascript_queries();

        engine
    }

    pub fn query<'a>(
        &self,
        language: &str,
        query_name: &str,
        node: Node<'a>,
        source: &'a str,
    ) -> Result<Vec<Match>, QueryError> {
        trace!(language, query_name, "executing query");

        let lang_queries = self
            .queries
            .get(language)
            .ok_or_else(|| QueryError::language_not_supported(language))?;

        let query = lang_queries
            .get(query_name)
            .ok_or_else(|| QueryError::query_not_found(language, query_name))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(query, node, source.as_bytes());
        let mut results = Vec::new();

        while let Some(m) = matches.next() {
            let mut captures = HashMap::new();

            for capture in m.captures {
                let name = query.capture_names()[capture.index as usize].to_string();
                let text = capture
                    .node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("")
                    .to_string();
                let start = capture.node.start_position();

                captures.insert(
                    name.clone(),
                    Capture {
                        name,
                        text,
                        start_byte: capture.node.start_byte(),
                        end_byte: capture.node.end_byte(),
                        start_row: start.row,
                        start_col: start.column,
                    },
                );
            }

            results.push(Match { captures });
        }

        Ok(results)
    }

    fn add_query(&mut self, language: &str, name: &str, ts_lang: &Language, pattern: &str) {
        trace!(language, name, "compiling query");

        let query = match Query::new(ts_lang, pattern) {
            Ok(q) => q,
            Err(e) => {
                warn!(language, name, error = %e, "failed to compile query");
                return;
            }
        };

        self.queries
            .entry(language.to_string())
            .or_default()
            .insert(name.to_string(), query);
    }

    fn load_go_queries(&mut self) {
        let lang: Language = tree_sitter_go::LANGUAGE.into();

        self.add_query(
            "go",
            "imports",
            &lang,
            r#"
            [
              (import_spec
                (interpreted_string_literal) @path)
              (import_spec
                (package_identifier) @alias
                (interpreted_string_literal) @path)
            ]
            "#,
        );

        self.add_query(
            "go",
            "calls",
            &lang,
            r#"
            (call_expression
              function: (selector_expression
                operand: (identifier) @package
                field: (field_identifier) @function)
              arguments: (argument_list) @args)
            "#,
        );
    }

    fn load_python_queries(&mut self) {
        let lang: Language = tree_sitter_python::LANGUAGE.into();

        self.add_query(
            "python",
            "imports",
            &lang,
            r#"
            [
              (import_statement
                name: (dotted_name) @path)

              (import_statement
                name: (aliased_import
                  name: (dotted_name) @path
                  alias: (identifier) @alias))

              (import_from_statement
                module_name: (dotted_name) @module
                name: (dotted_name) @name)

              (import_from_statement
                module_name: (dotted_name) @module
                name: (aliased_import
                  name: (dotted_name) @name
                  alias: (identifier) @alias))
            ]
            "#,
        );

        self.add_query(
            "python",
            "calls",
            &lang,
            r#"
            (call
              function: (attribute
                object: (identifier) @package
                attribute: (identifier) @function)
              arguments: (argument_list) @args)
            "#,
        );
    }

    fn load_rust_queries(&mut self) {
        let lang: Language = tree_sitter_rust::LANGUAGE.into();

        self.add_query(
            "rust",
            "imports",
            &lang,
            r#"
            (use_declaration
              argument: (scoped_identifier) @path)
            "#,
        );

        self.add_query(
            "rust",
            "calls",
            &lang,
            r#"
            (call_expression
              function: (scoped_identifier
                path: (identifier) @package
                name: (identifier) @function)
              arguments: (arguments) @args)

            (call_expression
              function: (field_expression
                value: (identifier) @package
                field: (field_identifier) @function)
              arguments: (arguments) @args)
            "#,
        );
    }

    fn load_javascript_queries(&mut self) {
        let lang: Language = tree_sitter_javascript::LANGUAGE.into();

        self.add_query(
            "javascript",
            "imports",
            &lang,
            r#"
            [
              (import_statement
                source: (string) @path)

              (variable_declarator
                name: (identifier) @alias
                value: (call_expression
                  function: (identifier) @_require
                  arguments: (arguments (string) @path))
                (#eq? @_require "require"))
            ]
            "#,
        );

        self.add_query(
            "javascript",
            "calls",
            &lang,
            r#"
            (call_expression
              function: (member_expression
                object: (identifier) @package
                property: (property_identifier) @function)
              arguments: (arguments) @args)
            "#,
        );
    }
}

impl Default for QueryEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_go(source: &str) -> tree_sitter::Tree {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    fn parse_python(source: &str) -> tree_sitter::Tree {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    #[test]
    fn test_go_imports_simple() {
        let source = r#"
package main

import "crypto/sha256"
"#;
        let tree = parse_go(source);
        let engine = QueryEngine::new();

        let matches = engine
            .query("go", "imports", tree.root_node(), source)
            .unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].get("path"), Some("\"crypto/sha256\""));
        assert!(matches[0].get("alias").is_none());
    }

    #[test]
    fn test_go_imports_aliased() {
        let source = r#"
package main

import pb "golang.org/x/crypto/pbkdf2"
"#;
        let tree = parse_go(source);
        let engine = QueryEngine::new();

        let matches = engine
            .query("go", "imports", tree.root_node(), source)
            .unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(
            matches[0].get("path"),
            Some("\"golang.org/x/crypto/pbkdf2\"")
        );
        assert_eq!(matches[0].get("alias"), Some("pb"));
    }

    #[test]
    fn test_go_calls() {
        let source = r#"
package main

func main() {
    sha256.New()
    pb.Key(password, salt, 100000, 32)
}
"#;
        let tree = parse_go(source);
        let engine = QueryEngine::new();

        let matches = engine
            .query("go", "calls", tree.root_node(), source)
            .unwrap();

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].get("package"), Some("sha256"));
        assert_eq!(matches[0].get("function"), Some("New"));
        assert_eq!(matches[1].get("package"), Some("pb"));
        assert_eq!(matches[1].get("function"), Some("Key"));
    }

    #[test]
    fn test_python_imports_simple() {
        let source = "import hashlib";
        let tree = parse_python(source);
        let engine = QueryEngine::new();

        let matches = engine
            .query("python", "imports", tree.root_node(), source)
            .unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].get("path"), Some("hashlib"));
    }

    #[test]
    fn test_python_imports_from() {
        let source = "from cryptography.hazmat.primitives import hashes";
        let tree = parse_python(source);
        let engine = QueryEngine::new();

        let matches = engine
            .query("python", "imports", tree.root_node(), source)
            .unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(
            matches[0].get("module"),
            Some("cryptography.hazmat.primitives")
        );
        assert_eq!(matches[0].get("name"), Some("hashes"));
    }

    #[test]
    fn test_python_calls() {
        let source = r#"
key = hashlib.pbkdf2_hmac('sha256', password, salt, 100000)
"#;
        let tree = parse_python(source);
        let engine = QueryEngine::new();

        let matches = engine
            .query("python", "calls", tree.root_node(), source)
            .unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].get("package"), Some("hashlib"));
        assert_eq!(matches[0].get("function"), Some("pbkdf2_hmac"));
    }
}
