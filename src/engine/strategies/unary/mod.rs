use crate::engine::{Context, Language, NodeCategory, Strategy, UnresolvedSource, Value};
use tree_sitter::Node;

mod languages;

pub struct UnaryStrategy;

impl Default for UnaryStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl UnaryStrategy {
    pub fn new() -> Self {
        Self
    }

    fn get_operator_and_operand<'a>(
        &self,
        node: &Node<'a>,
        ctx: &Context<'a>,
    ) -> Option<(String, Node<'a>)> {
        let lang = ctx.node_types()?.language();

        match lang {
            Language::Go => languages::go_get_unary(node, ctx),
            Language::Python => languages::python_get_unary(node, ctx),
            Language::Rust => languages::rust_get_unary(node, ctx),
            Language::JavaScript | Language::TypeScript => languages::js_get_unary(node, ctx),
            Language::C | Language::Cpp => languages::c_get_unary(node, ctx),
            Language::Java => languages::java_get_unary(node, ctx),
        }
    }

    fn resolve_operand<'a>(&self, operand: &Node<'a>, ctx: &Context<'a>) -> Value {
        let kind = operand.kind();

        if ctx.is_node_category(kind, NodeCategory::IntegerLiteral) {
            let text = ctx.get_node_text(operand);
            if let Some(value) = ctx.parse_int_literal(&text) {
                return Value::resolved_int(value);
            }
        }

        if ctx.is_node_category(kind, NodeCategory::FloatLiteral) {
            let text = ctx.get_node_text(operand).replace('_', "");
            if let Ok(value) = text.parse::<f64>() {
                if value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64 {
                    return Value::resolved_int(value as i64);
                }
            }
        }

        if ctx.is_node_category(kind, NodeCategory::BooleanLiteral) {
            return self.resolve_boolean(operand, ctx);
        }

        if ctx.is_node_category(kind, NodeCategory::UnaryExpression) {
            return self.resolve(operand, ctx);
        }

        if kind == "parenthesized_expression" {
            if let Some(inner) = operand.named_child(0) {
                return self.resolve_operand(&inner, ctx);
            }
        }

        Value::unextractable(UnresolvedSource::NotImplemented)
    }

    fn resolve_boolean<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        let kind = node.kind();

        if kind.eq_ignore_ascii_case("true") {
            return Value::resolved_int(1);
        }

        if kind.eq_ignore_ascii_case("false") {
            return Value::resolved_int(0);
        }

        if kind == "boolean_literal" {
            let text = ctx.get_node_text(node);
            if text.eq_ignore_ascii_case("true") {
                return Value::resolved_int(1);
            } else if text.eq_ignore_ascii_case("false") {
                return Value::resolved_int(0);
            }
        }

        Value::unextractable(UnresolvedSource::Unknown)
    }
}

impl Strategy for UnaryStrategy {
    fn name(&self) -> &'static str {
        "unary"
    }

    fn can_handle<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> bool {
        let kind = node.kind();
        ctx.is_node_category(kind, NodeCategory::UnaryExpression)
    }

    fn resolve<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        let (op_text, operand) = match self.get_operator_and_operand(node, ctx) {
            Some(result) => result,
            None => return Value::unextractable(UnresolvedSource::Unknown),
        };

        if op_text == "&" || op_text == "*" {
            let operand_text = ctx.get_node_text(&operand);
            return Value::partial_expression(format!("{op_text}{operand_text}"));
        }

        if op_text == "not" {
            let operand_value = self.resolve_operand(&operand, ctx);
            return Value::unary_op("!", &operand_value);
        }

        let operand_value = self.resolve_operand(&operand, ctx);
        Value::unary_op(&op_text, &operand_value)
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

    fn find_first_node_of_kind<'a>(
        node: tree_sitter::Node<'a>,
        kind: &str,
    ) -> Option<tree_sitter::Node<'a>> {
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

    fn find_first_unary_expression<'a>(
        node: tree_sitter::Node<'a>,
        ctx: &Context<'a>,
    ) -> Option<tree_sitter::Node<'a>> {
        let kind = node.kind();
        if ctx.is_node_category(kind, NodeCategory::UnaryExpression) {
            return Some(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = find_first_unary_expression(child, ctx) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn test_strategy_name() {
        let strategy = UnaryStrategy::new();
        assert_eq!(strategy.name(), "unary");
    }

    #[test]
    fn test_go_negative_integer() {
        let source = "package main\nconst x = -10000";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "unary_expression").unwrap();
        assert!(strategy.can_handle(&node, &ctx));

        let value = strategy.resolve(&node, &ctx);
        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![-10000]);
    }

    #[test]
    fn test_go_bitwise_not() {
        let source = "package main\nconst x = ^255";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "unary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![-256]); // ^255 = -256 in two's complement
    }

    #[test]
    fn test_go_logical_not_true() {
        let source = "package main\nvar x = !true";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "unary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![0]); // !true = false = 0
    }

    #[test]
    fn test_go_logical_not_false() {
        let source = "package main\nvar x = !false";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "unary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![1]); // !false = true = 1
    }

    #[test]
    fn test_go_positive_integer() {
        let source = "package main\nconst x = +42";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "unary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![42]); // +42 = 42
    }

    #[test]
    fn test_go_address_of() {
        let source = "package main\nvar x = &someVar";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "unary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(!value.is_resolved);
        assert_eq!(value.expression, "&someVar");
    }

    #[test]
    fn test_go_double_negative() {
        let source = "package main\nconst x = --42";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "unary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![42]); // --42 = 42
    }

    #[test]
    fn test_go_nested_unary() {
        let source = "package main\nconst x = -(-100)";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "unary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![100]); // -(-100) = 100
    }

    #[test]
    fn test_python_negative() {
        let source = "x = -10000";
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_unary_expression(tree.root_node(), &ctx).unwrap();
        assert!(strategy.can_handle(&node, &ctx));

        let value = strategy.resolve(&node, &ctx);
        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![-10000]);
    }

    #[test]
    fn test_python_bitwise_not() {
        let source = "x = ~255";
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_unary_expression(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![-256]); // ~255 = -256
    }

    #[test]
    fn test_python_not_true() {
        let source = "x = not True";
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_unary_expression(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![0]); // not True = False = 0
    }

    #[test]
    fn test_python_not_false() {
        let source = "x = not False";
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_unary_expression(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![1]); // not False = True = 1
    }

    #[test]
    fn test_rust_negative() {
        let source = "const X: i32 = -10000;";
        let tree = parse_rust(source);
        let ctx = create_rust_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_unary_expression(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![-10000]);
    }

    #[test]
    fn test_rust_logical_not() {
        let source = "let x = !true;";
        let tree = parse_rust(source);
        let ctx = create_rust_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_unary_expression(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![0]); // !true = false = 0
    }

    #[test]
    fn test_rust_reference() {
        let source = "let x = &some_var;";
        let tree = parse_rust(source);
        let ctx = create_rust_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_unary_expression(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(!value.is_resolved);
        assert_eq!(value.expression, "&some_var");
    }

    #[test]
    fn test_rust_dereference() {
        let source = "let x = *some_ptr;";
        let tree = parse_rust(source);
        let ctx = create_rust_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_unary_expression(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(!value.is_resolved);
        assert_eq!(value.expression, "*some_ptr");
    }

    #[test]
    fn test_js_negative() {
        let source = "const x = -10000;";
        let tree = parse_javascript(source);
        let ctx = create_js_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "unary_expression").unwrap();
        assert!(strategy.can_handle(&node, &ctx));

        let value = strategy.resolve(&node, &ctx);
        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![-10000]);
    }

    #[test]
    fn test_js_logical_not() {
        let source = "const x = !true;";
        let tree = parse_javascript(source);
        let ctx = create_js_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "unary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![0]);
    }

    #[test]
    fn test_js_bitwise_not() {
        let source = "const x = ~255;";
        let tree = parse_javascript(source);
        let ctx = create_js_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "unary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![-256]);
    }

    #[test]
    fn test_cannot_handle_literal() {
        let source = "package main\nconst x = 10000";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "int_literal").unwrap();
        assert!(!strategy.can_handle(&node, &ctx));
    }

    #[test]
    fn test_go_hex_negative() {
        let source = "package main\nconst x = -0xFF";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "unary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![-255]);
    }

    #[test]
    fn test_go_bitwise_not_zero() {
        let source = "package main\nconst x = ^0";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "unary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![-1]); // ^0 = -1 (all bits set)
    }

    #[test]
    fn test_unresolved_identifier_operand() {
        let source = "package main\nvar x = -someVar";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = UnaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "unary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(!value.is_resolved);
        assert_eq!(value.expression, "-<not_implemented>");
    }
}
