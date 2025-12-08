use crate::engine::{Context, Language, NodeCategory, Strategy, UnresolvedSource, Value};
use tree_sitter::Node;

mod languages;

pub struct IndexStrategy;

impl Default for IndexStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexStrategy {
    pub fn new() -> Self {
        Self
    }

    fn get_object_and_index<'a>(
        &self,
        node: &Node<'a>,
        ctx: &Context<'a>,
    ) -> Option<(Node<'a>, Node<'a>)> {
        let lang = ctx.node_types()?.language();

        match lang {
            Language::Go => languages::go_get_object_index(node),
            Language::Python => languages::python_get_object_index(node),
            Language::Rust => languages::rust_get_object_index(node),
            Language::JavaScript | Language::TypeScript => languages::js_get_object_index(node),
            Language::C | Language::Cpp => languages::c_get_object_index(node),
            Language::Java => languages::java_get_object_index(node),
        }
    }

    fn resolve_index_value<'a>(index_node: &Node<'a>, ctx: &Context<'a>) -> Option<i64> {
        let kind = index_node.kind();

        if ctx.is_node_category(kind, NodeCategory::IntegerLiteral) {
            let text = ctx.get_node_text(index_node);
            return ctx.parse_int_literal(&text);
        }

        if kind == "parenthesized_expression" {
            if let Some(inner) = index_node.named_child(0) {
                return Self::resolve_index_value(&inner, ctx);
            }
        }

        None
    }

    fn resolve_string_index<'a>(&self, index_node: &Node<'a>, ctx: &Context<'a>) -> Option<String> {
        let kind = index_node.kind();

        if ctx.is_node_category(kind, NodeCategory::StringLiteral) {
            let text = ctx.get_node_text(index_node);
            return Some(ctx.unquote_string(&text));
        }

        None
    }

    fn extract_array_elements<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Option<Vec<Value>> {
        let lang = ctx.node_types()?.language();
        let kind = node.kind();

        if !self.is_array_like(kind, lang) {
            return None;
        }

        match lang {
            Language::Go => self.extract_go_array_elements(node, ctx),
            _ => self.extract_generic_array_elements(node, ctx, lang),
        }
    }

    fn extract_go_array_elements<'a>(
        &self,
        node: &Node<'a>,
        ctx: &Context<'a>,
    ) -> Option<Vec<Value>> {
        let literal_value = node.child_by_field_name("body")?;
        let mut elements = Vec::new();
        let mut cursor = literal_value.walk();

        for child in literal_value.children(&mut cursor) {
            if child.kind() == "literal_element" {
                if let Some(value_node) = child.named_child(0) {
                    elements.push(self.resolve_element(&value_node, ctx));
                }
            }
        }

        Some(elements)
    }

    fn extract_generic_array_elements<'a>(
        &self,
        node: &Node<'a>,
        ctx: &Context<'a>,
        lang: Language,
    ) -> Option<Vec<Value>> {
        let mut elements = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.is_named() && self.is_element_node(&child, lang) {
                elements.push(self.resolve_element(&child, ctx));
            }
        }

        Some(elements)
    }

    fn is_array_like(&self, kind: &str, lang: Language) -> bool {
        match lang {
            Language::Go => kind == "composite_literal",
            Language::Python => kind == "list" || kind == "tuple",
            Language::Rust => kind == "array_expression",
            Language::JavaScript | Language::TypeScript => kind == "array",
            Language::C | Language::Cpp => kind == "initializer_list",
            Language::Java => kind == "array_initializer",
        }
    }

    fn is_element_node(&self, node: &Node, lang: Language) -> bool {
        let kind = node.kind();
        match lang {
            Language::Go => {
                kind != "type_identifier" && kind != "slice_type" && kind != "array_type"
            }
            Language::Python => true,
            Language::Rust => true,
            Language::JavaScript | Language::TypeScript => true,
            Language::C | Language::Cpp => true,
            Language::Java => true,
        }
    }

    fn resolve_element<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        let kind = node.kind();

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
            let text = if kind == "true" || kind == "True" {
                "true"
            } else {
                "false"
            };
            return Value::resolved_string(text.to_string());
        }

        Value::partial_expression(ctx.get_node_text(node))
    }

    fn extract_map_value<'a>(
        &self,
        node: &Node<'a>,
        key: &str,
        ctx: &Context<'a>,
    ) -> Option<Value> {
        let lang = ctx.node_types()?.language();
        let kind = node.kind();

        match lang {
            Language::Python if kind == "dictionary" => {
                self.extract_python_dict_value(node, key, ctx)
            }
            Language::JavaScript | Language::TypeScript if kind == "object" => {
                self.extract_js_object_value(node, key, ctx)
            }
            Language::Go if kind == "composite_literal" => {
                self.extract_go_map_value(node, key, ctx)
            }
            _ => None,
        }
    }

    fn extract_python_dict_value<'a>(
        &self,
        node: &Node<'a>,
        key: &str,
        ctx: &Context<'a>,
    ) -> Option<Value> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "pair" {
                let k = child.child_by_field_name("key")?;
                let v = child.child_by_field_name("value")?;

                let key_text = ctx.get_node_text(&k);
                let key_unquoted = ctx.unquote_string(&key_text);

                if key_unquoted == key {
                    return Some(self.resolve_element(&v, ctx));
                }
            }
        }
        None
    }

    fn extract_js_object_value<'a>(
        &self,
        node: &Node<'a>,
        key: &str,
        ctx: &Context<'a>,
    ) -> Option<Value> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "pair" {
                let k = child.child_by_field_name("key")?;
                let v = child.child_by_field_name("value")?;

                let key_text = ctx.get_node_text(&k);
                let key_unquoted = ctx.unquote_string(&key_text);

                if key_unquoted == key || key_text == key {
                    return Some(self.resolve_element(&v, ctx));
                }
            }
        }
        None
    }

    fn extract_go_map_value<'a>(
        &self,
        node: &Node<'a>,
        key: &str,
        ctx: &Context<'a>,
    ) -> Option<Value> {
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "keyed_element" {
                    let k = child.named_child(0)?;
                    let v = child.named_child(1)?;

                    let key_text = ctx.get_node_text(&k);
                    let key_unquoted = ctx.unquote_string(&key_text);

                    if key_unquoted == key {
                        return Some(self.resolve_element(&v, ctx));
                    }
                }
            }
        }
        None
    }
}

impl Strategy for IndexStrategy {
    fn name(&self) -> &'static str {
        "index"
    }

    fn can_handle<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> bool {
        ctx.is_node_category(node.kind(), NodeCategory::IndexExpression)
    }

    fn resolve<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        let (object_node, index_node) = match self.get_object_and_index(node, ctx) {
            Some((o, i)) => (o, i),
            None => return Value::unextractable(UnresolvedSource::Unknown),
        };

        if let Some(string_key) = self.resolve_string_index(&index_node, ctx) {
            if let Some(value) = self.extract_map_value(&object_node, &string_key, ctx) {
                return value;
            }
        }

        if let Some(index) = Self::resolve_index_value(&index_node, ctx) {
            if index < 0 {
                return Value::partial_expression(ctx.get_node_text(node));
            }

            if let Some(elements) = self.extract_array_elements(&object_node, ctx) {
                let idx = index as usize;
                if idx < elements.len() {
                    return elements[idx].clone();
                }
            }
        }

        Value::partial_expression(ctx.get_node_text(node))
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

    fn create_javascript_context<'a>(tree: &'a Tree, source: &'a [u8]) -> Context<'a> {
        Context::new(
            tree,
            source,
            "test.js".to_string(),
            "javascript".to_string(),
            HashMap::new(),
        )
    }

    fn find_first_index_expression<'a>(node: Node<'a>, ctx: &Context<'a>) -> Option<Node<'a>> {
        if ctx.is_node_category(node.kind(), NodeCategory::IndexExpression) {
            return Some(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = find_first_index_expression(child, ctx) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn test_strategy_name() {
        let strategy = IndexStrategy::new();
        assert_eq!(strategy.name(), "index");
    }

    #[test]
    fn test_go_array_index_first() {
        let source = "package main\nvar x = []int{10, 20, 30}[0]";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = IndexStrategy::new();

        let node = find_first_index_expression(tree.root_node(), &ctx).unwrap();
        assert!(strategy.can_handle(&node, &ctx));

        let value = strategy.resolve(&node, &ctx);
        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![10]);
    }

    #[test]
    fn test_go_array_index_middle() {
        let source = "package main\nvar x = []int{10, 20, 30}[1]";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = IndexStrategy::new();

        let node = find_first_index_expression(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![20]);
    }

    #[test]
    fn test_go_array_index_last() {
        let source = "package main\nvar x = []int{10, 20, 30}[2]";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = IndexStrategy::new();

        let node = find_first_index_expression(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![30]);
    }

    #[test]
    fn test_python_list_index() {
        let source = "x = [10, 20, 30][1]";
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = IndexStrategy::new();

        let node = find_first_index_expression(tree.root_node(), &ctx).unwrap();
        assert!(strategy.can_handle(&node, &ctx));

        let value = strategy.resolve(&node, &ctx);
        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![20]);
    }

    #[test]
    fn test_python_tuple_index() {
        let source = "x = (10, 20, 30)[2]";
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = IndexStrategy::new();

        let node = find_first_index_expression(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![30]);
    }

    #[test]
    fn test_python_dict_string_key() {
        let source = r#"x = {"key": 100}["key"]"#;
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = IndexStrategy::new();

        let node = find_first_index_expression(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![100]);
    }

    #[test]
    fn test_rust_array_index() {
        let source = "fn main() { let x = [10, 20, 30][1]; }";
        let tree = parse_rust(source);
        let ctx = create_rust_context(&tree, source.as_bytes());
        let strategy = IndexStrategy::new();

        let node = find_first_index_expression(tree.root_node(), &ctx).unwrap();
        assert!(strategy.can_handle(&node, &ctx));

        let value = strategy.resolve(&node, &ctx);
        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![20]);
    }

    #[test]
    fn test_javascript_array_index() {
        let source = "const x = [10, 20, 30][1];";
        let tree = parse_javascript(source);
        let ctx = create_javascript_context(&tree, source.as_bytes());
        let strategy = IndexStrategy::new();

        let node = find_first_index_expression(tree.root_node(), &ctx).unwrap();
        assert!(strategy.can_handle(&node, &ctx));

        let value = strategy.resolve(&node, &ctx);
        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![20]);
    }

    #[test]
    fn test_javascript_object_bracket_access() {
        let source = r#"const x = {key: 100}["key"];"#;
        let tree = parse_javascript(source);
        let ctx = create_javascript_context(&tree, source.as_bytes());
        let strategy = IndexStrategy::new();

        let node = find_first_index_expression(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![100]);
    }

    #[test]
    fn test_go_string_array_index() {
        let source = r#"package main
var x = []string{"sha256", "sha512"}[0]"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = IndexStrategy::new();

        let node = find_first_index_expression(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.string_values, vec!["sha256"]);
    }

    #[test]
    fn test_out_of_bounds_index() {
        let source = "package main\nvar x = []int{10, 20}[5]";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = IndexStrategy::new();

        let node = find_first_index_expression(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(!value.is_resolved);
        assert!(!value.expression.is_empty());
    }

    #[test]
    fn test_non_literal_index() {
        let source = "package main\nvar i = 1\nvar x = []int{10, 20}[i]";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = IndexStrategy::new();

        let node = find_first_index_expression(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(!value.is_resolved);
    }

    #[test]
    fn test_cannot_handle_literal() {
        let source = "package main\nconst x = 10000";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = IndexStrategy::new();

        fn find_int_literal<'a>(node: Node<'a>) -> Option<Node<'a>> {
            if node.kind() == "int_literal" {
                return Some(node);
            }
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if let Some(found) = find_int_literal(child) {
                    return Some(found);
                }
            }
            None
        }

        let node = find_int_literal(tree.root_node()).unwrap();
        assert!(!strategy.can_handle(&node, &ctx));
    }
}
