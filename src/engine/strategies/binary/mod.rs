use crate::engine::{
    BinaryOp, Context, Language, NodeCategory, Strategy, UnaryOp, UnresolvedSource, Value,
};
use tree_sitter::Node;

pub struct BinaryStrategy;

impl Default for BinaryStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl BinaryStrategy {
    pub fn new() -> Self {
        Self
    }

    fn get_operator<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Option<String> {
        if let Some(op_node) = node.child_by_field_name("operator") {
            return Some(ctx.get_node_text(&op_node));
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if !child.is_named() {
                let text = ctx.get_node_text(&child);
                if is_binary_operator(&text) {
                    return Some(text);
                }
            }
        }

        None
    }

    fn get_left_operand<'a>(&self, node: &Node<'a>) -> Option<Node<'a>> {
        if let Some(left) = node.child_by_field_name("left") {
            return Some(left);
        }

        let mut cursor = node.walk();
        let result = node.children(&mut cursor).find(|child| child.is_named());
        result
    }

    fn get_right_operand<'a>(&self, node: &Node<'a>) -> Option<Node<'a>> {
        if let Some(right) = node.child_by_field_name("right") {
            return Some(right);
        }

        let mut named_children: Vec<Node<'a>> = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                named_children.push(child);
            }
        }

        if named_children.len() >= 2 {
            return Some(named_children[1]);
        }

        None
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
            return self.resolve_unary(operand, ctx);
        }

        if ctx.is_node_category(kind, NodeCategory::BinaryExpression) {
            return self.resolve(operand, ctx);
        }

        if kind == "parenthesized_expression" {
            if let Some(inner) = operand.named_child(0) {
                return self.resolve_operand(&inner, ctx);
            }
        }

        Value::partial_expression(ctx.get_node_text(operand))
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

    fn resolve_unary<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        let (op_text, operand) = match self.get_unary_parts(node, ctx) {
            Some(result) => result,
            None => return Value::unextractable(UnresolvedSource::Unknown),
        };

        let operand_value = self.resolve_operand(&operand, ctx);
        Value::unary_op(&op_text, &operand_value)
    }

    fn get_unary_parts<'a>(
        &self,
        node: &Node<'a>,
        ctx: &Context<'a>,
    ) -> Option<(String, Node<'a>)> {
        let lang = ctx.node_types()?.language();

        match lang {
            Language::Go => {
                let operand = node.child_by_field_name("operand")?;
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if !child.is_named() {
                        let op_text = ctx.get_node_text(&child);
                        if is_unary_operator(&op_text) {
                            return Some((op_text, operand));
                        }
                    }
                }
                None
            }
            Language::Python => {
                if node.kind() == "not_operator" {
                    let operand = node.child_by_field_name("argument")?;
                    return Some(("!".to_string(), operand));
                }
                if node.kind() == "unary_operator" {
                    let operand = node.child_by_field_name("argument")?;
                    let op = node.child_by_field_name("operator")?;
                    return Some((ctx.get_node_text(&op), operand));
                }
                None
            }
            Language::Rust => {
                if node.kind() == "reference_expression" || node.kind() == "dereference_expression"
                {
                    let operand = node.child_by_field_name("value")?;
                    let op = if node.kind() == "reference_expression" {
                        "&"
                    } else {
                        "*"
                    };
                    return Some((op.to_string(), operand));
                }
                if node.kind() == "unary_expression" {
                    let operand = node.child(1)?;
                    if let Some(op_node) = node.child(0) {
                        if !op_node.is_named() {
                            return Some((ctx.get_node_text(&op_node), operand));
                        }
                    }
                }
                None
            }
            Language::JavaScript | Language::TypeScript => {
                let operand = node.child_by_field_name("argument")?;
                let op = node.child_by_field_name("operator")?;
                Some((ctx.get_node_text(&op), operand))
            }
            Language::C | Language::Cpp | Language::Java => {
                let operand = node
                    .child_by_field_name("operand")
                    .or_else(|| node.child_by_field_name("argument"))?;
                let op = node.child_by_field_name("operator")?;
                Some((ctx.get_node_text(&op), operand))
            }
        }
    }
}

fn is_binary_operator(s: &str) -> bool {
    BinaryOp::parse(s).is_some()
}

fn is_unary_operator(s: &str) -> bool {
    UnaryOp::parse(s).is_some() || matches!(s, "&" | "*" | "not")
}

impl Strategy for BinaryStrategy {
    fn name(&self) -> &'static str {
        "binary"
    }

    fn can_handle<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> bool {
        ctx.is_node_category(node.kind(), NodeCategory::BinaryExpression)
    }

    fn resolve<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        let left_node = match self.get_left_operand(node) {
            Some(n) => n,
            None => return Value::unextractable(UnresolvedSource::Unknown),
        };

        let right_node = match self.get_right_operand(node) {
            Some(n) => n,
            None => return Value::unextractable(UnresolvedSource::Unknown),
        };

        let operator = match self.get_operator(node, ctx) {
            Some(op) => op,
            None => return Value::unextractable(UnresolvedSource::Unknown),
        };

        let left_value = self.resolve_operand(&left_node, ctx);
        let right_value = self.resolve_operand(&right_node, ctx);

        Value::binary_op(&left_value, &operator, &right_value)
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

    fn find_first_binary_expression<'a>(node: Node<'a>, ctx: &Context<'a>) -> Option<Node<'a>> {
        let kind = node.kind();
        if ctx.is_node_category(kind, NodeCategory::BinaryExpression) {
            return Some(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = find_first_binary_expression(child, ctx) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn test_strategy_name() {
        let strategy = BinaryStrategy::new();
        assert_eq!(strategy.name(), "binary");
    }

    // =============================================================================
    // Addition Tests
    // =============================================================================

    #[test]
    fn test_go_addition() {
        let source = "package main\nconst x = 100 + 50";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        assert!(strategy.can_handle(&node, &ctx));

        let value = strategy.resolve(&node, &ctx);
        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![150]);
    }

    #[test]
    fn test_python_addition() {
        let source = "x = 100 + 50";
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_binary_expression(tree.root_node(), &ctx).unwrap();
        assert!(strategy.can_handle(&node, &ctx));

        let value = strategy.resolve(&node, &ctx);
        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![150]);
    }

    #[test]
    fn test_rust_addition() {
        let source = "const X: i32 = 100 + 50;";
        let tree = parse_rust(source);
        let ctx = create_rust_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        assert!(strategy.can_handle(&node, &ctx));

        let value = strategy.resolve(&node, &ctx);
        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![150]);
    }

    #[test]
    fn test_javascript_addition() {
        let source = "const x = 100 + 50;";
        let tree = parse_javascript(source);
        let ctx = create_javascript_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        assert!(strategy.can_handle(&node, &ctx));

        let value = strategy.resolve(&node, &ctx);
        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![150]);
    }

    // =============================================================================
    // Subtraction Tests
    // =============================================================================

    #[test]
    fn test_go_subtraction() {
        let source = "package main\nconst x = 100 - 30";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![70]);
    }

    #[test]
    fn test_python_subtraction() {
        let source = "x = 100 - 30";
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_binary_expression(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![70]);
    }

    // =============================================================================
    // Multiplication Tests
    // =============================================================================

    #[test]
    fn test_go_multiplication() {
        let source = "package main\nconst x = 32 * 8";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![256]);
    }

    #[test]
    fn test_python_multiplication() {
        let source = "x = 32 * 8";
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_binary_expression(tree.root_node(), &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![256]);
    }

    #[test]
    fn test_rust_multiplication() {
        let source = "const X: i32 = 32 * 8;";
        let tree = parse_rust(source);
        let ctx = create_rust_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![256]);
    }

    #[test]
    fn test_javascript_multiplication() {
        let source = "const x = 32 * 8;";
        let tree = parse_javascript(source);
        let ctx = create_javascript_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![256]);
    }

    // =============================================================================
    // Division Tests
    // =============================================================================

    #[test]
    fn test_go_division() {
        let source = "package main\nconst x = 100 / 4";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![25]);
    }

    #[test]
    fn test_go_division_by_zero() {
        let source = "package main\nconst x = 100 / 0";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(!value.is_resolved);
        assert_eq!(value.expression, "100 / 0");
    }

    // =============================================================================
    // Bitwise Shift Tests
    // =============================================================================

    #[test]
    fn test_go_shift_left() {
        let source = "package main\nconst x = 1 << 4";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![16]);
    }

    #[test]
    fn test_go_shift_right() {
        let source = "package main\nconst x = 16 >> 2";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![4]);
    }

    #[test]
    fn test_rust_shift() {
        let source = "const X: i32 = 1 << 8;";
        let tree = parse_rust(source);
        let ctx = create_rust_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![256]);
    }

    // =============================================================================
    // Bitwise Operation Tests
    // =============================================================================

    #[test]
    fn test_go_bitwise_and() {
        let source = "package main\nconst x = 0xFF & 0x0F";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![15]);
    }

    #[test]
    fn test_go_bitwise_or() {
        let source = "package main\nconst x = 0xF0 | 0x0F";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![255]);
    }

    #[test]
    fn test_go_bitwise_xor() {
        let source = "package main\nconst x = 0xFF ^ 0x0F";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![240]);
    }

    // =============================================================================
    // Comparison Tests
    // =============================================================================

    #[test]
    fn test_go_comparison_equal() {
        let source = "package main\nvar x = 10 == 10";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![1]);
    }

    #[test]
    fn test_go_comparison_not_equal() {
        let source = "package main\nvar x = 10 != 20";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![1]);
    }

    #[test]
    fn test_go_comparison_less() {
        let source = "package main\nvar x = 5 < 10";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![1]);
    }

    #[test]
    fn test_go_comparison_greater() {
        let source = "package main\nvar x = 10 > 5";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![1]);
    }

    #[test]
    fn test_go_comparison_less_equal() {
        let source = "package main\nvar x = 10 <= 10";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![1]);
    }

    #[test]
    fn test_go_comparison_greater_equal() {
        let source = "package main\nvar x = 10 >= 10";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![1]);
    }

    // =============================================================================
    // Logical Operator Tests
    // =============================================================================

    #[test]
    fn test_go_logical_and_true() {
        let source = "package main\nvar x = 1 != 0 && 2 != 0";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let nodes: Vec<_> = find_all_binary_expressions(tree.root_node(), &ctx);
        let outer = nodes.iter().find(|n| ctx.get_node_text(n).contains("&&"));

        if let Some(node) = outer {
            let value = strategy.resolve(node, &ctx);
            assert!(value.is_resolved);
            assert_eq!(value.int_values, vec![1]);
        }
    }

    #[test]
    fn test_go_logical_and_false() {
        let source = "package main\nvar x = 1 != 0 && 0 != 0";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let nodes: Vec<_> = find_all_binary_expressions(tree.root_node(), &ctx);
        let outer = nodes.iter().find(|n| ctx.get_node_text(n).contains("&&"));

        if let Some(node) = outer {
            let value = strategy.resolve(node, &ctx);
            assert!(value.is_resolved);
            assert_eq!(value.int_values, vec![0]);
        }
    }

    #[test]
    fn test_go_logical_or_true() {
        let source = "package main\nvar x = 0 == 0 || 1 == 0";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let nodes: Vec<_> = find_all_binary_expressions(tree.root_node(), &ctx);
        let outer = nodes.iter().find(|n| ctx.get_node_text(n).contains("||"));

        if let Some(node) = outer {
            let value = strategy.resolve(node, &ctx);
            assert!(value.is_resolved);
            assert_eq!(value.int_values, vec![1]);
        }
    }

    #[test]
    fn test_go_logical_or_false() {
        let source = "package main\nvar x = 1 == 0 || 2 == 0";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let nodes: Vec<_> = find_all_binary_expressions(tree.root_node(), &ctx);
        let outer = nodes.iter().find(|n| ctx.get_node_text(n).contains("||"));

        if let Some(node) = outer {
            let value = strategy.resolve(node, &ctx);
            assert!(value.is_resolved);
            assert_eq!(value.int_values, vec![0]);
        }
    }

    // =============================================================================
    // Nested Expression Tests
    // =============================================================================

    #[test]
    fn test_go_nested_addition() {
        let source = "package main\nconst x = 10 + 20 + 30";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let nodes: Vec<_> = find_all_binary_expressions(tree.root_node(), &ctx);
        let outer = nodes.iter().max_by_key(|n| n.byte_range().len());

        if let Some(node) = outer {
            let value = strategy.resolve(node, &ctx);
            assert!(value.is_resolved);
            assert_eq!(value.int_values, vec![60]);
        }
    }

    #[test]
    fn test_go_mixed_operations() {
        let source = "package main\nconst x = 100 + 50 * 2";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let nodes: Vec<_> = find_all_binary_expressions(tree.root_node(), &ctx);
        let outer = nodes.iter().max_by_key(|n| n.byte_range().len());

        if let Some(node) = outer {
            let value = strategy.resolve(node, &ctx);
            assert!(value.is_resolved);
            assert_eq!(value.int_values, vec![200]); // Go parses as (100 + 50) * 2 = 300 or 100 + (50 * 2) = 200
        }
    }

    #[test]
    fn test_go_with_unary_operand() {
        let source = "package main\nconst x = 100 + -50";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![50]);
    }

    #[test]
    fn test_go_parenthesized_expression() {
        let source = "package main\nconst x = (100 + 50) * 2";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let nodes: Vec<_> = find_all_binary_expressions(tree.root_node(), &ctx);
        let outer = nodes.iter().max_by_key(|n| n.byte_range().len());

        if let Some(node) = outer {
            let value = strategy.resolve(node, &ctx);
            assert!(value.is_resolved);
            assert_eq!(value.int_values, vec![300]);
        }
    }

    // =============================================================================
    // Crypto-Relevant Tests
    // =============================================================================

    #[test]
    fn test_go_pbkdf2_iteration_calculation() {
        let source = "package main\nconst BASE_ITER = 100000\nconst x = 100000 + 10000";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![110000]);
    }

    #[test]
    fn test_go_key_size_bytes_to_bits() {
        let source = "package main\nconst x = 32 * 8";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![256]); // 32 bytes = 256 bits
    }

    #[test]
    fn test_go_aes_key_calculation() {
        let source = "package main\nconst x = 256 / 8";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![32]); // 256 bits = 32 bytes
    }

    // =============================================================================
    // Partial Resolution Tests
    // =============================================================================

    #[test]
    fn test_go_unresolved_identifier() {
        let source = "package main\nvar x = iterations + 10000";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(!value.is_resolved);
        assert_eq!(value.expression, "iterations + 10000");
    }

    #[test]
    fn test_go_one_resolved_one_unresolved() {
        let source = "package main\nvar x = 100000 + extra";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "binary_expression").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(!value.is_resolved);
        assert_eq!(value.expression, "100000 + extra");
    }

    // =============================================================================
    // Cannot Handle Tests
    // =============================================================================

    #[test]
    fn test_cannot_handle_literal() {
        let source = "package main\nconst x = 10000";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "int_literal").unwrap();
        assert!(!strategy.can_handle(&node, &ctx));
    }

    #[test]
    fn test_cannot_handle_unary() {
        let source = "package main\nconst x = -10000";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = BinaryStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "unary_expression").unwrap();
        assert!(!strategy.can_handle(&node, &ctx));
    }

    fn find_all_binary_expressions<'a>(node: Node<'a>, ctx: &Context<'a>) -> Vec<Node<'a>> {
        let mut result = Vec::new();
        let kind = node.kind();
        if ctx.is_node_category(kind, NodeCategory::BinaryExpression) {
            result.push(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            result.extend(find_all_binary_expressions(child, ctx));
        }
        result
    }
}
