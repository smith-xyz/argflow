mod imports;
mod patterns;

use std::collections::HashMap;
use tracing::{debug, trace, warn};
use tree_sitter::{Node, Tree};

use crate::engine::{Context, NodeCategory, Resolver, Value};
use crate::query::QueryEngine;
use crate::utils::{extract_last_segment, unquote_string};
pub use imports::ImportMap;
pub use patterns::default_patterns;

/// Trait for matching function calls to crypto patterns.
///
/// This abstraction allows swapping the matching strategy:
/// - Default: Simple pattern matching against known crypto terms
/// - Future: Classifier-based matching using `crypto-classifier-rules` submodule
pub trait CryptoMatcher: Send + Sync {
    fn is_crypto_call(
        &self,
        function_name: &str,
        package: Option<&str>,
        import_path: Option<&str>,
    ) -> bool;
}

/// Default pattern-based matcher. Matches against a list of known crypto terms.
/// This is a simple MVP implementation; for production use the classifier module.
pub struct PatternMatcher {
    patterns: Vec<String>,
}

impl PatternMatcher {
    pub fn new(patterns: Vec<String>) -> Self {
        Self { patterns }
    }

    pub fn default_patterns() -> Self {
        Self::new(patterns::default_patterns())
    }
}

impl CryptoMatcher for PatternMatcher {
    fn is_crypto_call(
        &self,
        function_name: &str,
        package: Option<&str>,
        import_path: Option<&str>,
    ) -> bool {
        let full_name = match package {
            Some(pkg) => format!("{pkg}.{function_name}").to_lowercase(),
            None => function_name.to_lowercase(),
        };

        let check_pattern = |text: &str| {
            self.patterns
                .iter()
                .any(|p| text.to_lowercase().contains(&p.to_lowercase()))
        };

        // Check function name + package first
        if check_pattern(&full_name) {
            return true;
        }

        // Also check import path (for aliased imports like `pb "golang.org/x/crypto/pbkdf2"`)
        if let Some(path) = import_path {
            if check_pattern(path) {
                return true;
            }
        }

        false
    }
}

#[derive(Debug, Clone)]
pub struct CryptoCall {
    pub file_path: String,
    pub line: usize,
    pub column: usize,
    pub function_name: String,
    pub package: Option<String>,
    pub import_path: Option<String>,
    pub arguments: Vec<Value>,
    pub raw_text: String,
    pub language: String,
}

impl CryptoCall {
    pub fn full_name(&self) -> String {
        match &self.package {
            Some(pkg) => format!("{}.{}", pkg, self.function_name),
            None => self.function_name.clone(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ScanResult {
    pub file_path: String,
    pub calls: Vec<CryptoCall>,
    pub errors: Vec<String>,
}

impl ScanResult {
    pub fn new(file_path: String) -> Self {
        Self {
            file_path,
            calls: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn add_call(&mut self, call: CryptoCall) {
        self.calls.push(call);
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn call_count(&self) -> usize {
        self.calls.len()
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

pub struct Scanner {
    resolver: Resolver,
    matcher: Box<dyn CryptoMatcher>,
    query_engine: QueryEngine,
}

impl Scanner {
    pub fn new() -> Self {
        Self {
            resolver: Resolver::new(),
            matcher: Box::new(PatternMatcher::default_patterns()),
            query_engine: QueryEngine::new(),
        }
    }

    pub fn with_resolver(resolver: Resolver) -> Self {
        Self {
            resolver,
            matcher: Box::new(PatternMatcher::default_patterns()),
            query_engine: QueryEngine::new(),
        }
    }

    pub fn with_matcher<M: CryptoMatcher + 'static>(mut self, matcher: M) -> Self {
        self.matcher = Box::new(matcher);
        self
    }

    pub fn with_patterns(self, patterns: Vec<String>) -> Self {
        self.with_matcher(PatternMatcher::new(patterns))
    }

    pub fn scan_tree<'a>(
        &self,
        tree: &'a Tree,
        source: &'a [u8],
        file_path: &str,
        language: &str,
    ) -> ScanResult {
        trace!(file_path, language, "scanning tree");

        let source_str = std::str::from_utf8(source).unwrap_or("");
        let ctx = Context::new(
            tree,
            source,
            file_path.to_string(),
            language.to_string(),
            HashMap::new(),
        );

        let imports = self.extract_imports_via_query(tree, source_str, language);
        trace!(import_count = imports.len(), "extracted imports");

        let mut result = ScanResult::new(file_path.to_string());
        self.traverse_node(tree.root_node(), &ctx, &imports, &mut result);

        debug!(
            file_path,
            calls = result.call_count(),
            errors = result.errors.len(),
            "scan complete"
        );
        result
    }

    fn extract_imports_via_query(&self, tree: &Tree, source: &str, language: &str) -> ImportMap {
        let mut imports = ImportMap::new();

        let matches = match self
            .query_engine
            .query(language, "imports", tree.root_node(), source)
        {
            Ok(m) => m,
            Err(e) => {
                warn!(language, error = %e, "failed to extract imports");
                return imports;
            }
        };

        for m in matches {
            let path = m.get("path").map(unquote_string);
            let alias = m.get("alias");
            let module = m.get("module");
            let name = m.get("name");

            match (path, module, name, alias) {
                // from module import name (Python style)
                (None, Some(mod_path), Some(imported_name), alias_opt) => {
                    let key = alias_opt
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| imported_name.to_string());
                    let full_path = format!("{mod_path}.{imported_name}");
                    imports.insert(key, full_path);
                }
                // Simple import: import "path" or import alias "path"
                (Some(p), None, None, alias_opt) => {
                    let short_name = alias_opt
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| extract_last_segment(&p));
                    imports.insert(short_name, p);
                }
                _ => {}
            }
        }

        imports
    }

    fn traverse_node<'a>(
        &self,
        node: Node<'a>,
        ctx: &Context<'a>,
        imports: &ImportMap,
        result: &mut ScanResult,
    ) {
        if ctx.is_node_category(node.kind(), NodeCategory::CallExpression) {
            if let Some(call) = self.process_call_node(&node, ctx, imports) {
                if self.is_crypto_call(&call) {
                    result.add_call(call);
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_node(child, ctx, imports, result);
        }
    }

    fn process_call_node<'a>(
        &self,
        node: &Node<'a>,
        ctx: &Context<'a>,
        imports: &ImportMap,
    ) -> Option<CryptoCall> {
        let (function_name, package) = self.extract_function_name(node, ctx)?;
        let arguments = self.extract_arguments(node, ctx);
        let raw_text = ctx.get_node_text(node);

        let import_path = package.as_ref().and_then(|pkg| imports.resolve(pkg));

        let start = node.start_position();

        Some(CryptoCall {
            file_path: ctx.file_path().to_string(),
            line: start.row + 1, // 1-indexed
            column: start.column + 1,
            function_name,
            package,
            import_path,
            arguments,
            raw_text,
            language: ctx.language().to_string(),
        })
    }

    fn extract_function_name<'a>(
        &self,
        node: &Node<'a>,
        ctx: &Context<'a>,
    ) -> Option<(String, Option<String>)> {
        // Different languages structure call expressions differently
        // Go: call_expression -> function (selector_expression or identifier)
        // Python: call -> function (attribute or identifier)

        let func_node = node.child_by_field_name("function")?;

        match func_node.kind() {
            // Go: pkg.Function or obj.Method
            "selector_expression" => {
                let operand = func_node.child_by_field_name("operand")?;
                let field = func_node.child_by_field_name("field")?;
                let package = ctx.get_node_text(&operand);
                let name = ctx.get_node_text(&field);
                Some((name, Some(package)))
            }
            // Python: module.function
            "attribute" => {
                let obj = func_node.child_by_field_name("object")?;
                let attr = func_node.child_by_field_name("attribute")?;
                let package = ctx.get_node_text(&obj);
                let name = ctx.get_node_text(&attr);
                Some((name, Some(package)))
            }
            // Simple identifier: function()
            "identifier" => {
                let name = ctx.get_node_text(&func_node);
                Some((name, None))
            }
            // Member expression (JS): obj.method
            "member_expression" => {
                let obj = func_node.child_by_field_name("object")?;
                let prop = func_node.child_by_field_name("property")?;
                let package = ctx.get_node_text(&obj);
                let name = ctx.get_node_text(&prop);
                Some((name, Some(package)))
            }
            _ => {
                // Fallback: just get the text
                let name = ctx.get_node_text(&func_node);
                Some((name, None))
            }
        }
    }

    fn extract_arguments<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Vec<Value> {
        let mut arguments = Vec::new();

        // Find the arguments node - try field name first, then search children
        let args_node = node
            .child_by_field_name("arguments")
            .or_else(|| self.find_arguments_child(node));

        if let Some(args) = args_node {
            let mut cursor = args.walk();
            for child in args.children(&mut cursor) {
                // Skip punctuation (commas, parens)
                if child.is_named() {
                    // For Python keyword arguments (name=value), extract just the value
                    let value_node = if child.kind() == "keyword_argument" {
                        child.child_by_field_name("value").unwrap_or(child)
                    } else {
                        child
                    };
                    let value = self.resolver.resolve(&value_node, ctx);
                    arguments.push(value);
                }
            }
        }

        arguments
    }

    fn find_arguments_child<'a>(&self, node: &Node<'a>) -> Option<Node<'a>> {
        let mut cursor = node.walk();
        let result = node
            .children(&mut cursor)
            .find(|child| child.kind() == "argument_list" || child.kind() == "arguments");
        result
    }

    fn is_crypto_call(&self, call: &CryptoCall) -> bool {
        self.matcher.is_crypto_call(
            &call.function_name,
            call.package.as_deref(),
            call.import_path.as_deref(),
        )
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_go(source: &str) -> Tree {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    #[test]
    fn test_scanner_creation() {
        let scanner = Scanner::new();
        // Scanner is created with default pattern matcher
        let source = r#"package main
func main() { pbkdf2.Key() }"#;
        let tree = parse_go(source);
        let result = scanner.scan_tree(&tree, source.as_bytes(), "test.go", "go");
        assert_eq!(result.call_count(), 1); // Should detect pbkdf2
    }

    #[test]
    fn test_crypto_call_full_name() {
        let call = CryptoCall {
            file_path: "test.go".to_string(),
            line: 10,
            column: 5,
            function_name: "Key".to_string(),
            package: Some("pbkdf2".to_string()),
            import_path: Some("golang.org/x/crypto/pbkdf2".to_string()),
            arguments: vec![],
            raw_text: "pbkdf2.Key(...)".to_string(),
            language: "go".to_string(),
        };
        assert_eq!(call.full_name(), "pbkdf2.Key");
    }

    #[test]
    fn test_crypto_call_no_package() {
        let call = CryptoCall {
            file_path: "test.go".to_string(),
            line: 10,
            column: 5,
            function_name: "encrypt".to_string(),
            package: None,
            import_path: None,
            arguments: vec![],
            raw_text: "encrypt(...)".to_string(),
            language: "go".to_string(),
        };
        assert_eq!(call.full_name(), "encrypt");
    }

    #[test]
    fn test_scan_result() {
        let mut result = ScanResult::new("test.go".to_string());
        assert_eq!(result.call_count(), 0);
        assert!(!result.has_errors());

        result.add_call(CryptoCall {
            file_path: "test.go".to_string(),
            line: 1,
            column: 1,
            function_name: "test".to_string(),
            package: None,
            import_path: None,
            arguments: vec![],
            raw_text: "test()".to_string(),
            language: "go".to_string(),
        });
        assert_eq!(result.call_count(), 1);

        result.add_error("test error".to_string());
        assert!(result.has_errors());
    }

    #[test]
    fn test_scan_detects_pbkdf2() {
        let source = r#"
package main

import "golang.org/x/crypto/pbkdf2"

func main() {
    key := pbkdf2.Key(password, salt, 10000, 32, sha256.New)
}
"#;
        let tree = parse_go(source);
        let scanner = Scanner::new();
        let result = scanner.scan_tree(&tree, source.as_bytes(), "test.go", "go");

        assert_eq!(result.call_count(), 1);
        let call = &result.calls[0];
        assert_eq!(call.function_name, "Key");
        assert_eq!(call.package, Some("pbkdf2".to_string()));
    }

    #[test]
    fn test_scan_detects_multiple_calls() {
        let source = r#"
package main

func main() {
    h := sha256.New()
    key := pbkdf2.Key(p, s, 10000, 32, h)
    encrypted := aes.NewCipher(key)
}
"#;
        let tree = parse_go(source);
        let scanner = Scanner::new();
        let result = scanner.scan_tree(&tree, source.as_bytes(), "test.go", "go");

        assert_eq!(result.call_count(), 3);
    }

    #[test]
    fn test_scan_ignores_non_crypto() {
        let source = r#"
package main

func main() {
    fmt.Println("hello")
    x := strings.Split(s, ",")
}
"#;
        let tree = parse_go(source);
        let scanner = Scanner::new();
        let result = scanner.scan_tree(&tree, source.as_bytes(), "test.go", "go");

        assert_eq!(result.call_count(), 0);
    }

    #[test]
    fn test_scan_extracts_arguments() {
        let source = r#"
package main

func main() {
    key := pbkdf2.Key(password, salt, 10000, 32, sha256.New)
}
"#;
        let tree = parse_go(source);
        let scanner = Scanner::new();
        let result = scanner.scan_tree(&tree, source.as_bytes(), "test.go", "go");

        assert_eq!(result.call_count(), 1);
        let call = &result.calls[0];
        assert_eq!(call.arguments.len(), 5);

        // Third argument should be resolved as 10000
        let iterations = &call.arguments[2];
        assert!(iterations.is_resolved);
        assert_eq!(iterations.int_values, vec![10000]);

        // Fourth argument should be resolved as 32
        let key_len = &call.arguments[3];
        assert!(key_len.is_resolved);
        assert_eq!(key_len.int_values, vec![32]);
    }

    #[test]
    fn test_custom_patterns() {
        let source = r#"
package main

func main() {
    myCustomCrypto.DoThing()
}
"#;
        let tree = parse_go(source);
        let scanner = Scanner::new().with_patterns(vec!["mycustomcrypto".to_string()]);
        let result = scanner.scan_tree(&tree, source.as_bytes(), "test.go", "go");

        assert_eq!(result.call_count(), 1);
    }

    // =========================================================================
    // Import Tracking Integration Tests
    // =========================================================================

    #[test]
    fn test_import_tracking_go_simple() {
        let source = r#"
package main

import "golang.org/x/crypto/pbkdf2"

func main() {
    key := pbkdf2.Key(password, salt, 10000, 32, sha256.New)
}
"#;
        let tree = parse_go(source);
        let scanner = Scanner::new();
        let result = scanner.scan_tree(&tree, source.as_bytes(), "test.go", "go");

        assert_eq!(result.call_count(), 1);
        let call = &result.calls[0];
        assert_eq!(call.package, Some("pbkdf2".to_string()));
        assert_eq!(
            call.import_path,
            Some("golang.org/x/crypto/pbkdf2".to_string())
        );
    }

    #[test]
    fn test_import_tracking_go_aliased() {
        let source = r#"
package main

import pb "golang.org/x/crypto/pbkdf2"

func main() {
    key := pb.Key(password, salt, 10000, 32, sha256.New)
}
"#;
        let tree = parse_go(source);
        let scanner = Scanner::new();
        let result = scanner.scan_tree(&tree, source.as_bytes(), "test.go", "go");

        assert_eq!(result.call_count(), 1);
        let call = &result.calls[0];
        assert_eq!(call.package, Some("pb".to_string()));
        assert_eq!(
            call.import_path,
            Some("golang.org/x/crypto/pbkdf2".to_string())
        );
    }

    #[test]
    fn test_import_tracking_go_grouped() {
        let source = r#"
package main

import (
    "crypto/sha256"
    "golang.org/x/crypto/pbkdf2"
)

func main() {
    h := sha256.New()
    key := pbkdf2.Key(password, salt, 10000, 32, h)
}
"#;
        let tree = parse_go(source);
        let scanner = Scanner::new();
        let result = scanner.scan_tree(&tree, source.as_bytes(), "test.go", "go");

        assert_eq!(result.call_count(), 2);

        let sha_call = result
            .calls
            .iter()
            .find(|c| c.function_name == "New")
            .unwrap();
        assert_eq!(sha_call.import_path, Some("crypto/sha256".to_string()));

        let pbkdf2_call = result
            .calls
            .iter()
            .find(|c| c.function_name == "Key")
            .unwrap();
        assert_eq!(
            pbkdf2_call.import_path,
            Some("golang.org/x/crypto/pbkdf2".to_string())
        );
    }

    #[test]
    fn test_import_tracking_python() {
        let source = r#"
import hashlib

key = hashlib.pbkdf2_hmac('sha256', password, salt, 100000)
"#;
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(source, None).unwrap();

        let scanner = Scanner::new();
        let result = scanner.scan_tree(&tree, source.as_bytes(), "test.py", "python");

        assert_eq!(result.call_count(), 1);
        let call = &result.calls[0];
        assert_eq!(call.package, Some("hashlib".to_string()));
        assert_eq!(call.import_path, Some("hashlib".to_string()));
    }

    #[test]
    fn test_import_tracking_python_from() {
        let source = r#"
from cryptography.hazmat.primitives import hashes

h = hashes.SHA256()
"#;
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(source, None).unwrap();

        let scanner = Scanner::new();
        let result = scanner.scan_tree(&tree, source.as_bytes(), "test.py", "python");

        assert_eq!(result.call_count(), 1);
        let call = &result.calls[0];
        assert_eq!(call.package, Some("hashes".to_string()));
        assert_eq!(
            call.import_path,
            Some("cryptography.hazmat.primitives.hashes".to_string())
        );
    }
}
