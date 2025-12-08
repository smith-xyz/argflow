use crate::engine::{Context, Language, NodeCategory, Resolver, Strategy, UnresolvedSource, Value};
use tree_sitter::Node;

mod languages;

pub struct SelectorStrategy {
    resolver: Option<Resolver>,
}

impl Default for SelectorStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectorStrategy {
    pub fn new() -> Self {
        Self { resolver: None }
    }

    pub fn with_resolver(resolver: Resolver) -> Self {
        Self {
            resolver: Some(resolver),
        }
    }

    fn get_object_and_field<'a>(
        &self,
        node: &Node<'a>,
        ctx: &Context<'a>,
    ) -> Option<(Node<'a>, String)> {
        let lang = ctx.node_types()?.language();

        match lang {
            Language::Go => languages::go_get_selector(node, ctx),
            Language::Python => languages::python_get_selector(node, ctx),
            Language::Rust => languages::rust_get_selector(node, ctx),
            Language::JavaScript | Language::TypeScript => languages::js_get_selector(node, ctx),
            Language::C | Language::Cpp => languages::c_get_selector(node, ctx),
            Language::Java => languages::java_get_selector(node, ctx),
        }
    }

    fn is_package_identifier<'a>(&self, object: &Node<'a>, ctx: &Context<'a>) -> bool {
        if object.kind() == "identifier" {
            let name = ctx.get_node_text(object);
            return self.looks_like_package_name(&name, ctx);
        }
        false
    }

    fn looks_like_package_name<'a>(&self, name: &str, ctx: &Context<'a>) -> bool {
        let lang = ctx.node_types().map(|nt| nt.language());

        match lang {
            Some(Language::Go) => {
                // Go packages are lowercase identifiers
                !name.is_empty() && name.chars().next().is_some_and(|c| c.is_lowercase())
            }
            Some(Language::Python) => {
                // Python modules are typically lowercase
                !name.is_empty() && name.chars().next().is_some_and(|c| c.is_lowercase())
            }
            Some(Language::Rust) => {
                // Rust modules/crates are lowercase with underscores
                !name.is_empty() && name.chars().all(|c| c.is_lowercase() || c == '_')
            }
            Some(Language::JavaScript | Language::TypeScript) => {
                // JS modules - check if it matches common patterns
                !name.is_empty()
            }
            _ => false,
        }
    }

    fn resolve_package_constant<'a>(
        &self,
        _package: &Node<'a>,
        field_name: &str,
        ctx: &Context<'a>,
    ) -> Value {
        // Try to find cross-file constant with this name
        if let Some(value) = ctx.find_cross_file_constant(field_name) {
            return value;
        }

        // Return partial expression preserving the selector
        let package_name = ctx.get_node_text(_package);
        Value::partial_expression(format!("{package_name}.{field_name}"))
    }

    fn resolve_field_access<'a>(
        &self,
        object: &Node<'a>,
        field_name: &str,
        ctx: &Context<'a>,
    ) -> Value {
        // Try to resolve the object using the full resolver chain
        let object_value = self.resolve_object(object, ctx);

        if object_value.is_resolved {
            // Check if this is tuple/array index access (e.g., cfg.0, cfg.1)
            if let Ok(index) = field_name.parse::<usize>() {
                // Tuple/array index access - try to extract the element
                if !object_value.int_values.is_empty() {
                    if let Some(&val) = object_value.int_values.get(index) {
                        return Value::resolved_int(val);
                    }
                }
                if !object_value.string_values.is_empty() {
                    if let Some(val) = object_value.string_values.get(index) {
                        return Value::resolved_string(val.clone());
                    }
                }
            }

            // Object resolved but we can't access named fields on primitive values
            // Return partial expression
            let object_text = ctx.get_node_text(object);
            return Value::partial_expression(format!("{object_text}.{field_name}"));
        }

        // For struct field access, we'd need type information
        // Return partial expression for now
        let object_text = ctx.get_node_text(object);
        Value::partial_expression(format!("{object_text}.{field_name}"))
    }

    fn resolve_object<'a>(&self, object: &Node<'a>, ctx: &Context<'a>) -> Value {
        // Use resolver if available for full strategy chain
        if let Some(ref resolver) = self.resolver {
            return resolver.resolve(object, ctx);
        }

        // Create a new resolver for full strategy chain resolution
        // This enables proper chaining through identifier -> composite strategies
        let resolver = Resolver::new();
        resolver.resolve(object, ctx)
    }
}

impl Strategy for SelectorStrategy {
    fn name(&self) -> &'static str {
        "selector"
    }

    fn can_handle<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> bool {
        ctx.is_node_category(node.kind(), NodeCategory::SelectorExpression)
    }

    fn resolve<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        let (object, field_name) = match self.get_object_and_field(node, ctx) {
            Some(result) => result,
            None => return Value::unextractable(UnresolvedSource::Unknown),
        };

        // Check if this looks like a package-qualified constant (pkg.Constant)
        if self.is_package_identifier(&object, ctx) {
            // Check if the field name looks like a constant (starts with uppercase in Go)
            let looks_like_constant = field_name.chars().next().is_some_and(|c| c.is_uppercase());

            if looks_like_constant {
                return self.resolve_package_constant(&object, &field_name, ctx);
            }
        }

        // Otherwise treat as field access (obj.field)
        self.resolve_field_access(&object, &field_name, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tree_sitter::Tree;

    fn parse_go(source: &str) -> Tree {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    fn parse_python(source: &str) -> Tree {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    fn parse_rust(source: &str) -> Tree {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    fn parse_javascript(source: &str) -> Tree {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    fn create_go_context<'a>(tree: &'a Tree, source: &'a [u8]) -> Context<'a> {
        Context::new(
            tree,
            source,
            "test.go".to_string(),
            "go".to_string(),
            HashMap::new(),
        )
    }

    fn create_python_context<'a>(tree: &'a Tree, source: &'a [u8]) -> Context<'a> {
        Context::new(
            tree,
            source,
            "test.py".to_string(),
            "python".to_string(),
            HashMap::new(),
        )
    }

    fn create_rust_context<'a>(tree: &'a Tree, source: &'a [u8]) -> Context<'a> {
        Context::new(
            tree,
            source,
            "test.rs".to_string(),
            "rust".to_string(),
            HashMap::new(),
        )
    }

    fn create_js_context<'a>(tree: &'a Tree, source: &'a [u8]) -> Context<'a> {
        Context::new(
            tree,
            source,
            "test.js".to_string(),
            "javascript".to_string(),
            HashMap::new(),
        )
    }

    fn find_first_node_of_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
        if node.kind() == kind {
            return Some(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = find_first_node_of_kind(child, kind) {
                return Some(found);
            }
        }
        None
    }

    fn find_first_selector<'a>(node: Node<'a>, ctx: &Context<'a>) -> Option<Node<'a>> {
        let kind = node.kind();
        if ctx.is_node_category(kind, NodeCategory::SelectorExpression) {
            return Some(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = find_first_selector(child, ctx) {
                return Some(found);
            }
        }
        None
    }

    fn find_all_selectors<'a>(node: Node<'a>, ctx: &Context<'a>) -> Vec<Node<'a>> {
        let mut result = Vec::new();
        let kind = node.kind();
        if ctx.is_node_category(kind, NodeCategory::SelectorExpression) {
            result.push(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            result.extend(find_all_selectors(child, ctx));
        }
        result
    }

    #[test]
    fn test_strategy_name() {
        let strategy = SelectorStrategy::new();
        assert_eq!(strategy.name(), "selector");
    }

    // =============================================================================
    // Go Tests
    // =============================================================================

    #[test]
    fn test_go_package_constant() {
        let source = r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() { pbkdf2.Key(p, s, crypto.DefaultIterations, 32, h) }
"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = SelectorStrategy::new();

        let selectors = find_all_selectors(tree.root_node(), &ctx);
        // Find selector that's not pbkdf2.Key (which is a call)
        let selector = selectors
            .iter()
            .find(|s| ctx.get_node_text(s).contains("DefaultIterations"));

        if let Some(node) = selector {
            assert!(strategy.can_handle(node, &ctx));
            let value = strategy.resolve(node, &ctx);
            // Package constant - returns partial expression
            assert!(!value.is_resolved);
            assert!(value.expression.contains("DefaultIterations"));
        }
    }

    #[test]
    fn test_go_field_access() {
        let source = r#"
package main
func main() { 
    cfg := Config{Iterations: 10000}
    use(cfg.Iterations) 
}
"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = SelectorStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "selector_expression").unwrap();
        assert!(strategy.can_handle(&node, &ctx));

        let value = strategy.resolve(&node, &ctx);
        // Field access on identifier - returns partial expression
        assert!(!value.is_resolved);
        assert_eq!(value.expression, "cfg.Iterations");
    }

    #[test]
    fn test_go_chained_selector() {
        let source = r#"
package main
func main() { use(a.b.c) }
"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = SelectorStrategy::new();

        let selectors = find_all_selectors(tree.root_node(), &ctx);
        let outer = selectors.iter().max_by_key(|s| s.byte_range().len());

        if let Some(node) = outer {
            let value = strategy.resolve(node, &ctx);
            assert!(!value.is_resolved);
            assert_eq!(value.expression, "a.b.c");
        }
    }

    #[test]
    fn test_go_lowercase_package() {
        let source = r#"
package main
import "crypto/aes"
func main() { aes.NewCipher(key) }
"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = SelectorStrategy::new();

        // This is a call_expression, not a selector, but let's test the selector part
        if let Some(selector) = find_first_selector(tree.root_node(), &ctx) {
            let value = strategy.resolve(&selector, &ctx);
            // aes.NewCipher - NewCipher starts with uppercase
            assert!(!value.is_resolved);
        }
    }

    // =============================================================================
    // Python Tests
    // =============================================================================

    #[test]
    fn test_python_attribute_access() {
        let source = r#"
class Config:
    iterations = 10000

cfg = Config()
print(cfg.iterations)
"#;
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = SelectorStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "attribute").unwrap();
        assert!(strategy.can_handle(&node, &ctx));

        let value = strategy.resolve(&node, &ctx);
        assert!(!value.is_resolved);
        assert_eq!(value.expression, "cfg.iterations");
    }

    #[test]
    fn test_python_module_constant() {
        let source = r#"
from hashlib import sha256
result = hashlib.sha256(data)
"#;
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = SelectorStrategy::new();

        if let Some(node) = find_first_selector(tree.root_node(), &ctx) {
            let value = strategy.resolve(&node, &ctx);
            assert!(!value.is_resolved);
        }
    }

    #[test]
    fn test_python_chained_attribute() {
        let source = r#"
result = a.b.c
"#;
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = SelectorStrategy::new();

        let selectors = find_all_selectors(tree.root_node(), &ctx);
        let outer = selectors.iter().max_by_key(|s| s.byte_range().len());

        if let Some(node) = outer {
            let value = strategy.resolve(node, &ctx);
            assert!(!value.is_resolved);
            assert_eq!(value.expression, "a.b.c");
        }
    }

    // =============================================================================
    // Rust Tests
    // =============================================================================

    #[test]
    fn test_rust_field_access() {
        let source = r#"
struct Config {
    iterations: u32,
}

fn main() {
    let cfg = Config { iterations: 10000 };
    println!("{}", cfg.iterations);
}
"#;
        let tree = parse_rust(source);
        let ctx = create_rust_context(&tree, source.as_bytes());
        let strategy = SelectorStrategy::new();

        if let Some(node) = find_first_node_of_kind(tree.root_node(), "field_expression") {
            assert!(strategy.can_handle(&node, &ctx));
            let value = strategy.resolve(&node, &ctx);
            assert!(!value.is_resolved);
            assert_eq!(value.expression, "cfg.iterations");
        }
    }

    #[test]
    fn test_rust_chained_field() {
        let source = r#"
fn main() {
    let x = a.b.c;
}
"#;
        let tree = parse_rust(source);
        let ctx = create_rust_context(&tree, source.as_bytes());
        let strategy = SelectorStrategy::new();

        let selectors = find_all_selectors(tree.root_node(), &ctx);
        let outer = selectors.iter().max_by_key(|s| s.byte_range().len());

        if let Some(node) = outer {
            let value = strategy.resolve(node, &ctx);
            assert!(!value.is_resolved);
            assert_eq!(value.expression, "a.b.c");
        }
    }

    // =============================================================================
    // JavaScript Tests
    // =============================================================================

    #[test]
    fn test_js_property_access() {
        let source = r#"
const cfg = { iterations: 10000 };
console.log(cfg.iterations);
"#;
        let tree = parse_javascript(source);
        let ctx = create_js_context(&tree, source.as_bytes());
        let strategy = SelectorStrategy::new();

        let selectors = find_all_selectors(tree.root_node(), &ctx);
        let target = selectors
            .iter()
            .find(|s| ctx.get_node_text(s) == "cfg.iterations");

        if let Some(node) = target {
            assert!(strategy.can_handle(node, &ctx));
            let value = strategy.resolve(node, &ctx);
            assert!(!value.is_resolved);
            assert_eq!(value.expression, "cfg.iterations");
        }
    }

    #[test]
    fn test_js_module_export() {
        let source = r#"
const crypto = require('crypto');
crypto.pbkdf2(password, salt, 100000, 32, 'sha256');
"#;
        let tree = parse_javascript(source);
        let ctx = create_js_context(&tree, source.as_bytes());
        let strategy = SelectorStrategy::new();

        if let Some(node) = find_first_selector(tree.root_node(), &ctx) {
            let value = strategy.resolve(&node, &ctx);
            // crypto.pbkdf2 - pbkdf2 is lowercase so treated as field
            assert!(!value.is_resolved);
        }
    }

    #[test]
    fn test_js_chained_property() {
        let source = r#"
const x = a.b.c;
"#;
        let tree = parse_javascript(source);
        let ctx = create_js_context(&tree, source.as_bytes());
        let strategy = SelectorStrategy::new();

        let selectors = find_all_selectors(tree.root_node(), &ctx);
        let outer = selectors.iter().max_by_key(|s| s.byte_range().len());

        if let Some(node) = outer {
            let value = strategy.resolve(node, &ctx);
            assert!(!value.is_resolved);
            assert_eq!(value.expression, "a.b.c");
        }
    }

    // =============================================================================
    // Cannot Handle Tests
    // =============================================================================

    #[test]
    fn test_cannot_handle_identifier() {
        let source = "package main\nvar x = someVar";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = SelectorStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "identifier").unwrap();
        assert!(!strategy.can_handle(&node, &ctx));
    }

    #[test]
    fn test_cannot_handle_literal() {
        let source = "package main\nconst x = 10000";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = SelectorStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "int_literal").unwrap();
        assert!(!strategy.can_handle(&node, &ctx));
    }

    #[test]
    fn test_cannot_handle_binary() {
        let source = "package main\nconst x = 10 + 20";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = SelectorStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        assert!(!strategy.can_handle(&node, &ctx));
    }

    #[test]
    fn test_go_tls_version_constant() {
        let source = r#"package main
import "crypto/tls"
func main() {
    cfg := tls.Config{
        MinVersion: tls.VersionTLS12,
    }
    _ = cfg
}"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = SelectorStrategy::new();

        // Find tls.VersionTLS12 selector
        let selectors = find_all_selectors(tree.root_node(), &ctx);
        let tls_version = selectors
            .iter()
            .find(|s| ctx.get_node_text(s) == "tls.VersionTLS12");

        assert!(
            tls_version.is_some(),
            "Should find tls.VersionTLS12 selector"
        );
        let node = tls_version.unwrap();

        assert!(strategy.can_handle(node, &ctx));
        let value = strategy.resolve(node, &ctx);

        // Should return partial expression "tls.VersionTLS12"
        assert!(!value.is_resolved);
        assert_eq!(value.expression, "tls.VersionTLS12");
    }
}
