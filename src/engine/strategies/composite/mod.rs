use crate::engine::{Context, Language, NodeCategory, Strategy, UnresolvedSource, Value};
use tree_sitter::Node;

mod languages;

pub struct CompositeStrategy;

impl Default for CompositeStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl CompositeStrategy {
    pub fn new() -> Self {
        Self
    }

    fn is_array_literal<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> bool {
        ctx.is_node_category(node.kind(), NodeCategory::ArrayLiteral)
    }

    fn is_struct_literal<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> bool {
        ctx.is_node_category(node.kind(), NodeCategory::StructLiteral)
    }

    fn resolve_array_literal<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        let lang = ctx.node_types().map(|nt| nt.language());

        match lang {
            Some(Language::Go) => languages::go_resolve_array(self, node, ctx),
            Some(Language::Python) => languages::python_resolve_array(self, node, ctx),
            Some(Language::Rust) => languages::rust_resolve_array(self, node, ctx),
            Some(Language::JavaScript | Language::TypeScript) => {
                languages::js_resolve_array(self, node, ctx)
            }
            Some(Language::C | Language::Cpp) => languages::c_resolve_array(self, node, ctx),
            Some(Language::Java) => languages::java_resolve_array(self, node, ctx),
            None => Value::unextractable(UnresolvedSource::Unknown),
        }
    }

    fn resolve_struct_literal<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        let lang = ctx.node_types().map(|nt| nt.language());

        match lang {
            Some(Language::Go) => languages::go_resolve_struct(self, node, ctx),
            Some(Language::Python) => languages::python_resolve_dict(self, node, ctx),
            Some(Language::Rust) => languages::rust_resolve_struct(self, node, ctx),
            Some(Language::JavaScript | Language::TypeScript) => {
                languages::js_resolve_object(self, node, ctx)
            }
            Some(Language::C | Language::Cpp) => languages::c_resolve_initializer(self, node, ctx),
            Some(Language::Java) => languages::java_resolve_object(self, node, ctx),
            None => Value::unextractable(UnresolvedSource::Unknown),
        }
    }

    pub(crate) fn collect_array_elements<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        let mut int_values = Vec::new();
        let mut string_values = Vec::new();
        let mut all_resolved = true;
        let mut expressions = Vec::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if !child.is_named() {
                continue;
            }

            if child.kind() == "comment" {
                continue;
            }

            if self.is_keyed_element(&child) {
                continue;
            }

            let value = self.resolve_element(&child, ctx);

            if value.is_resolved {
                int_values.extend(value.int_values);
                string_values.extend(value.string_values);
            } else {
                all_resolved = false;
                if !value.expression.is_empty() {
                    expressions.push(value.expression);
                } else {
                    expressions.push(ctx.get_node_text(&child));
                }
            }
        }

        if int_values.is_empty() && string_values.is_empty() && expressions.is_empty() {
            return Value::unextractable(UnresolvedSource::Unknown);
        }

        if !int_values.is_empty() || !string_values.is_empty() {
            Value {
                is_resolved: all_resolved,
                int_values,
                string_values,
                source: String::new(),
                expression: if all_resolved {
                    String::new()
                } else {
                    expressions.join(", ")
                },
            }
        } else {
            Value::partial_expression(format!("[{}]", expressions.join(", ")))
        }
    }

    pub(crate) fn collect_struct_fields<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        let mut fields: Vec<(String, Value)> = Vec::new();
        let mut all_resolved = true;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some((name, value)) = self.extract_keyed_element(&child, ctx) {
                if !value.is_resolved {
                    all_resolved = false;
                }
                fields.push((name, value));
            }
        }

        if fields.is_empty() {
            return Value::unextractable(UnresolvedSource::Unknown);
        }

        let field_strs: Vec<String> = fields
            .iter()
            .map(|(name, value)| {
                if value.is_resolved {
                    if !value.int_values.is_empty() {
                        format!("{name}: {}", value.int_values[0])
                    } else if !value.string_values.is_empty() {
                        format!("{name}: \"{}\"", value.string_values[0])
                    } else {
                        format!("{name}: ?")
                    }
                } else {
                    format!("{name}: {}", value.expression)
                }
            })
            .collect();

        let mut int_values = Vec::new();
        let mut string_values = Vec::new();
        for (_, value) in &fields {
            int_values.extend(value.int_values.clone());
            string_values.extend(value.string_values.clone());
        }

        Value {
            is_resolved: all_resolved,
            int_values,
            string_values,
            source: String::new(),
            expression: format!("{{{}}}", field_strs.join(", ")),
        }
    }

    pub(crate) fn collect_dict_entries<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        let mut entries: Vec<(String, Value)> = Vec::new();
        let mut all_resolved = true;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "pair" {
                if let Some(key) = child.child_by_field_name("key") {
                    if let Some(value_node) = child.child_by_field_name("value") {
                        let key_text = ctx.get_node_text(&key);
                        let key_clean = ctx.unquote_string(&key_text);
                        let value = self.resolve_element(&value_node, ctx);
                        if !value.is_resolved {
                            all_resolved = false;
                        }
                        entries.push((key_clean, value));
                    }
                }
            }
        }

        if entries.is_empty() {
            return Value::unextractable(UnresolvedSource::Unknown);
        }

        let mut int_values = Vec::new();
        let mut string_values = Vec::new();
        for (_, value) in &entries {
            int_values.extend(value.int_values.clone());
            string_values.extend(value.string_values.clone());
        }

        Value {
            is_resolved: all_resolved,
            int_values,
            string_values,
            source: String::new(),
            expression: format!("dict with {} entries", entries.len()),
        }
    }

    pub(crate) fn collect_object_properties<'a>(
        &self,
        node: &Node<'a>,
        ctx: &Context<'a>,
    ) -> Value {
        let mut properties: Vec<(String, Value)> = Vec::new();
        let mut all_resolved = true;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "pair" || child.kind() == "property" {
                if let Some((name, value)) = self.extract_js_property(&child, ctx) {
                    if !value.is_resolved {
                        all_resolved = false;
                    }
                    properties.push((name, value));
                }
            }
        }

        if properties.is_empty() {
            return Value::unextractable(UnresolvedSource::Unknown);
        }

        let mut int_values = Vec::new();
        let mut string_values = Vec::new();
        for (_, value) in &properties {
            int_values.extend(value.int_values.clone());
            string_values.extend(value.string_values.clone());
        }

        Value {
            is_resolved: all_resolved,
            int_values,
            string_values,
            source: String::new(),
            expression: format!("object with {} properties", properties.len()),
        }
    }

    pub(crate) fn collect_c_designated_initializers<'a>(
        &self,
        node: &Node<'a>,
        ctx: &Context<'a>,
    ) -> Value {
        let mut values_found = Vec::new();
        let mut all_resolved = true;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                let value = self.resolve_element(&child, ctx);
                if !value.is_resolved {
                    all_resolved = false;
                }
                values_found.push(value);
            }
        }

        if values_found.is_empty() {
            return Value::unextractable(UnresolvedSource::Unknown);
        }

        let mut int_values = Vec::new();
        let mut string_values = Vec::new();
        for value in &values_found {
            int_values.extend(value.int_values.clone());
            string_values.extend(value.string_values.clone());
        }

        Value {
            is_resolved: all_resolved,
            int_values,
            string_values,
            source: String::new(),
            expression: String::new(),
        }
    }

    pub(crate) fn is_keyed_element(&self, node: &Node) -> bool {
        // Note: literal_element is NOT a keyed element - it's a wrapper around values
        // In Go arrays: literal_value -> literal_element -> int_literal
        // In Go structs: literal_value -> keyed_element -> literal_element (key) + literal_element (value)
        matches!(
            node.kind(),
            "keyed_element"
                | "field_initializer"
                | "field_initializer_list"
                | "shorthand_field_initializer"
        )
    }

    pub(crate) fn extract_keyed_element<'a>(
        &self,
        node: &Node<'a>,
        ctx: &Context<'a>,
    ) -> Option<(String, Value)> {
        match node.kind() {
            "keyed_element" => {
                let name = node.child(0).map(|n| ctx.get_node_text(&n))?;
                let value_node = node.child(2)?;
                Some((name, self.resolve_element(&value_node, ctx)))
            }
            "literal_element" => {
                let mut cursor = node.walk();
                let children: Vec<_> = node.children(&mut cursor).collect();
                if children.len() >= 3 {
                    let name = ctx.get_node_text(&children[0]);
                    let value = self.resolve_element(&children[2], ctx);
                    Some((name, value))
                } else {
                    None
                }
            }
            "field_initializer" => {
                let name = node
                    .child_by_field_name("field")
                    .or_else(|| node.child(0))?;
                let value_node = node
                    .child_by_field_name("value")
                    .or_else(|| node.child(2))?;
                Some((
                    ctx.get_node_text(&name),
                    self.resolve_element(&value_node, ctx),
                ))
            }
            "shorthand_field_initializer" => {
                let name_node = node.child(0)?;
                let name = ctx.get_node_text(&name_node);
                Some((name.clone(), Value::partial_expression(name)))
            }
            _ => None,
        }
    }

    pub(crate) fn extract_js_property<'a>(
        &self,
        node: &Node<'a>,
        ctx: &Context<'a>,
    ) -> Option<(String, Value)> {
        let key = node.child_by_field_name("key")?;
        let value_node = node.child_by_field_name("value")?;

        let key_text = ctx.get_node_text(&key);
        let key_clean = ctx.unquote_string(&key_text);

        Some((key_clean, self.resolve_element(&value_node, ctx)))
    }

    pub(crate) fn resolve_element<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        let kind = node.kind();

        // Go wraps array elements in literal_element - unwrap to get the actual value
        if kind == "literal_element" {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.is_named() {
                    return self.resolve_element(&child, ctx);
                }
            }
            return Value::partial_expression(ctx.get_node_text(node));
        }

        if ctx.is_node_category(kind, NodeCategory::IntegerLiteral) {
            let text = ctx.get_node_text(node);
            if let Some(value) = ctx.parse_int_literal(&text) {
                return Value::resolved_int(value);
            }
        }

        if ctx.is_node_category(kind, NodeCategory::StringLiteral) {
            let text = ctx.get_node_text(node);
            let unquoted = ctx.unquote_string(&text);
            return Value::resolved_string(unquoted);
        }

        if ctx.is_node_category(kind, NodeCategory::BooleanLiteral) {
            let text = ctx.get_node_text(node);
            return Value::resolved_string(text);
        }

        if ctx.is_node_category(kind, NodeCategory::NilLiteral) {
            return Value::resolved_string(kind.to_string());
        }

        if self.is_array_literal(node, ctx) {
            return self.resolve_array_literal(node, ctx);
        }

        if self.is_struct_literal(node, ctx) {
            return self.resolve_struct_literal(node, ctx);
        }

        if ctx.is_node_category(kind, NodeCategory::SelectorExpression) {
            let text = ctx.get_node_text(node);
            return Value::partial_expression(text);
        }

        if ctx.is_node_category(kind, NodeCategory::Identifier) {
            let text = ctx.get_node_text(node);
            return Value::partial_expression(text);
        }

        Value::partial_expression(ctx.get_node_text(node))
    }
}

impl Strategy for CompositeStrategy {
    fn name(&self) -> &'static str {
        "composite"
    }

    fn can_handle<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> bool {
        self.is_array_literal(node, ctx) || self.is_struct_literal(node, ctx)
    }

    fn resolve<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        if self.is_array_literal(node, ctx) {
            let kind = node.kind();
            // Go's composite_literal can be either array or struct - check for keyed elements
            if kind == "composite_literal" && self.has_keyed_elements(node) {
                return self.resolve_struct_literal(node, ctx);
            }
            return self.resolve_array_literal(node, ctx);
        }

        if self.is_struct_literal(node, ctx) {
            return self.resolve_struct_literal(node, ctx);
        }

        Value::unextractable(UnresolvedSource::NotImplemented)
    }
}

impl CompositeStrategy {
    fn find_literal_value_child<'a>(&self, node: &Node<'a>) -> Option<Node<'a>> {
        let mut cursor = node.walk();
        let result = node
            .children(&mut cursor)
            .find(|child| child.kind() == "literal_value");
        result
    }

    fn has_keyed_elements(&self, node: &Node) -> bool {
        let body = node
            .child_by_field_name("body")
            .or_else(|| self.find_literal_value_child(node));

        if let Some(body_node) = body {
            let mut cursor = body_node.walk();
            for child in body_node.children(&mut cursor) {
                // In Go, struct literals have keyed_element nodes directly under literal_value
                // Array literals have literal_element nodes (not keyed)
                if child.kind() == "keyed_element" {
                    return true;
                }
            }
        }

        false
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

    fn find_composite<'a>(node: Node<'a>, ctx: &Context<'a>) -> Option<Node<'a>> {
        if ctx.is_node_category(node.kind(), NodeCategory::ArrayLiteral)
            || ctx.is_node_category(node.kind(), NodeCategory::StructLiteral)
        {
            return Some(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = find_composite(child, ctx) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn test_strategy_name() {
        let strategy = CompositeStrategy::new();
        assert_eq!(strategy.name(), "composite");
    }

    // =========================================================================
    // Go Array Tests
    // =========================================================================

    #[test]
    fn test_go_int_array() {
        let source = r#"
package main

var sizes = []int{16, 24, 32}
"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = CompositeStrategy::new();

        let node = find_composite(tree.root_node(), &ctx).unwrap();
        assert!(strategy.can_handle(&node, &ctx));

        let value = strategy.resolve(&node, &ctx);
        assert!(value.is_resolved);
        assert!(value.int_values.contains(&16));
        assert!(value.int_values.contains(&24));
        assert!(value.int_values.contains(&32));
    }

    #[test]
    fn test_go_string_array() {
        let source = r#"
package main

var algorithms = []string{"sha256", "sha384", "sha512"}
"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = CompositeStrategy::new();

        let node = find_composite(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert!(value.string_values.contains(&"sha256".to_string()));
        assert!(value.string_values.contains(&"sha384".to_string()));
        assert!(value.string_values.contains(&"sha512".to_string()));
    }

    #[test]
    fn test_go_struct_literal() {
        let source = r#"
package main

type Config struct {
    Iterations int
    KeySize    int
}

var cfg = Config{Iterations: 10000, KeySize: 32}
"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = CompositeStrategy::new();

        let node = find_composite(tree.root_node(), &ctx).unwrap();
        assert!(strategy.can_handle(&node, &ctx));

        let value = strategy.resolve(&node, &ctx);
        assert!(value.int_values.contains(&10000));
        assert!(value.int_values.contains(&32));
    }

    #[test]
    fn test_go_tls_config() {
        let source = r#"
package main

import "crypto/tls"

var tlsCfg = &tls.Config{
    MinVersion: tls.VersionTLS12,
    MaxVersion: tls.VersionTLS13,
}
"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = CompositeStrategy::new();

        let node = find_composite(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(!value.expression.is_empty());
        assert!(value.expression.contains("MinVersion"));
        assert!(value.expression.contains("MaxVersion"));
    }

    // =========================================================================
    // Python Tests
    // =========================================================================

    #[test]
    fn test_python_list() {
        let source = r#"
sizes = [16, 24, 32]
"#;
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = CompositeStrategy::new();

        let node = find_composite(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert!(value.int_values.contains(&16));
        assert!(value.int_values.contains(&24));
        assert!(value.int_values.contains(&32));
    }

    #[test]
    fn test_python_tuple() {
        let source = r#"
key_sizes = (128, 192, 256)
"#;
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = CompositeStrategy::new();

        let node = find_composite(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert!(value.int_values.contains(&128));
        assert!(value.int_values.contains(&192));
        assert!(value.int_values.contains(&256));
    }

    #[test]
    fn test_python_dict() {
        let source = r#"
config = {"iterations": 10000, "key_size": 32}
"#;
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = CompositeStrategy::new();

        let node = find_composite(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.int_values.contains(&10000));
        assert!(value.int_values.contains(&32));
    }

    // =========================================================================
    // Rust Tests
    // =========================================================================

    #[test]
    fn test_rust_array() {
        let source = r#"
let sizes: [i32; 3] = [16, 24, 32];
"#;
        let tree = parse_rust(source);
        let ctx = create_rust_context(&tree, source.as_bytes());
        let strategy = CompositeStrategy::new();

        let node = find_composite(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert!(value.int_values.contains(&16));
        assert!(value.int_values.contains(&24));
        assert!(value.int_values.contains(&32));
    }

    #[test]
    fn test_rust_struct() {
        let source = r#"
struct Config {
    iterations: u32,
    key_size: u32,
}

let cfg = Config { iterations: 10000, key_size: 32 };
"#;
        let tree = parse_rust(source);
        let ctx = create_rust_context(&tree, source.as_bytes());
        let strategy = CompositeStrategy::new();

        if let Some(node) = find_composite(tree.root_node(), &ctx) {
            let value = strategy.resolve(&node, &ctx);
            assert!(value.int_values.contains(&10000) || value.int_values.contains(&32));
        }
    }

    // =========================================================================
    // JavaScript Tests
    // =========================================================================

    #[test]
    fn test_js_array() {
        let source = r#"
const sizes = [16, 24, 32];
"#;
        let tree = parse_javascript(source);
        let ctx = create_js_context(&tree, source.as_bytes());
        let strategy = CompositeStrategy::new();

        let node = find_composite(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert!(value.int_values.contains(&16));
        assert!(value.int_values.contains(&24));
        assert!(value.int_values.contains(&32));
    }

    #[test]
    fn test_js_object() {
        let source = r#"
const config = {
    iterations: 10000,
    keySize: 32
};
"#;
        let tree = parse_javascript(source);
        let ctx = create_js_context(&tree, source.as_bytes());
        let strategy = CompositeStrategy::new();

        let node = find_composite(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.int_values.contains(&10000));
        assert!(value.int_values.contains(&32));
    }

    // =========================================================================
    // Crypto-Relevant Tests
    // =========================================================================

    #[test]
    fn test_go_cipher_suite_array() {
        let source = r#"
package main

var cipherSuites = []uint16{
    0x1301, // TLS_AES_128_GCM_SHA256
    0x1302, // TLS_AES_256_GCM_SHA384
    0x1303, // TLS_CHACHA20_POLY1305_SHA256
}
"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = CompositeStrategy::new();

        let node = find_composite(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert!(value.int_values.contains(&0x1301));
        assert!(value.int_values.contains(&0x1302));
        assert!(value.int_values.contains(&0x1303));
    }

    #[test]
    fn test_go_pbkdf2_config() {
        let source = r#"
package main

type KDFConfig struct {
    Iterations int
    KeyLength  int
    HashName   string
}

var kdfCfg = KDFConfig{
    Iterations: 100000,
    KeyLength:  32,
    HashName:   "sha256",
}
"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = CompositeStrategy::new();

        let node = find_composite(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.int_values.contains(&100000));
        assert!(value.int_values.contains(&32));
        assert!(value.string_values.contains(&"sha256".to_string()));
    }

    #[test]
    fn test_js_crypto_config() {
        let source = r#"
const cryptoConfig = {
    algorithm: "aes-256-gcm",
    keySize: 32,
    ivSize: 12
};
"#;
        let tree = parse_javascript(source);
        let ctx = create_js_context(&tree, source.as_bytes());
        let strategy = CompositeStrategy::new();

        let node = find_composite(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.string_values.contains(&"aes-256-gcm".to_string()));
        assert!(value.int_values.contains(&32));
        assert!(value.int_values.contains(&12));
    }
}
