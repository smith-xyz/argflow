use crate::engine::{Context, NodeCategory, Strategy, UnresolvedSource, Value};
use tree_sitter::Node;

pub struct LiteralStrategy;

impl Default for LiteralStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl LiteralStrategy {
    pub fn new() -> Self {
        Self
    }

    fn resolve_integer(&self, node: &Node, ctx: &Context) -> Value {
        let text = ctx.get_node_text(node);
        match ctx.parse_int_literal(&text) {
            Some(value) => Value::resolved_int(value),
            None => Value::unextractable(UnresolvedSource::Unknown),
        }
    }

    fn resolve_float(&self, node: &Node, ctx: &Context) -> Value {
        let text = ctx.get_node_text(node).replace('_', "");
        match text.parse::<f64>() {
            Ok(value) => {
                if value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64 {
                    Value::resolved_int(value as i64)
                } else {
                    Value::resolved_string(text)
                }
            }
            Err(_) => Value::unextractable(UnresolvedSource::Unknown),
        }
    }

    fn resolve_string(&self, node: &Node, ctx: &Context) -> Value {
        let text = ctx.get_node_text(node);
        let unquoted = ctx.unquote_string(&text);
        Value::resolved_string(unquoted)
    }

    fn resolve_boolean(&self, node: &Node, _ctx: &Context) -> Value {
        let kind = node.kind();
        if kind.eq_ignore_ascii_case("true") {
            Value::resolved_string("true".to_string())
        } else if kind.eq_ignore_ascii_case("false") {
            Value::resolved_string("false".to_string())
        } else {
            Value::unextractable(UnresolvedSource::Unknown)
        }
    }

    fn resolve_nil(&self, node: &Node, _ctx: &Context) -> Value {
        let kind = node.kind();
        Value::resolved_string(kind.to_string())
    }
}

impl Strategy for LiteralStrategy {
    fn name(&self) -> &'static str {
        "literal"
    }

    fn can_handle<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> bool {
        let kind = node.kind();
        ctx.is_node_category(kind, NodeCategory::IntegerLiteral)
            || ctx.is_node_category(kind, NodeCategory::FloatLiteral)
            || ctx.is_node_category(kind, NodeCategory::StringLiteral)
            || ctx.is_node_category(kind, NodeCategory::BooleanLiteral)
            || ctx.is_node_category(kind, NodeCategory::NilLiteral)
    }

    fn resolve<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        let kind = node.kind();

        if ctx.is_node_category(kind, NodeCategory::IntegerLiteral) {
            return self.resolve_integer(node, ctx);
        }

        if ctx.is_node_category(kind, NodeCategory::FloatLiteral) {
            return self.resolve_float(node, ctx);
        }

        if ctx.is_node_category(kind, NodeCategory::StringLiteral) {
            return self.resolve_string(node, ctx);
        }

        if ctx.is_node_category(kind, NodeCategory::BooleanLiteral) {
            return self.resolve_boolean(node, ctx);
        }

        if ctx.is_node_category(kind, NodeCategory::NilLiteral) {
            return self.resolve_nil(node, ctx);
        }

        Value::unextractable(UnresolvedSource::NotImplemented)
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

    fn create_context<'a>(tree: &'a Tree, source: &'a [u8]) -> Context<'a> {
        Context::new(
            tree,
            source,
            "test.go".to_string(),
            "go".to_string(),
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

    #[test]
    fn test_strategy_name() {
        let strategy = LiteralStrategy::new();
        assert_eq!(strategy.name(), "literal");
    }

    #[test]
    fn test_integer_decimal() {
        let source = "package main\nconst x = 10000";
        let tree = parse_go(source);
        let ctx = create_context(&tree, source.as_bytes());
        let strategy = LiteralStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "int_literal").unwrap();
        assert!(strategy.can_handle(&node, &ctx));

        let value = strategy.resolve(&node, &ctx);
        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![10000]);
    }

    #[test]
    fn test_integer_hex() {
        let source = "package main\nconst x = 0xFF";
        let tree = parse_go(source);
        let ctx = create_context(&tree, source.as_bytes());
        let strategy = LiteralStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "int_literal").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![255]);
    }

    #[test]
    fn test_integer_octal() {
        let source = "package main\nconst x = 0o755";
        let tree = parse_go(source);
        let ctx = create_context(&tree, source.as_bytes());
        let strategy = LiteralStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "int_literal").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![493]);
    }

    #[test]
    fn test_integer_binary() {
        let source = "package main\nconst x = 0b1010";
        let tree = parse_go(source);
        let ctx = create_context(&tree, source.as_bytes());
        let strategy = LiteralStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "int_literal").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![10]);
    }

    #[test]
    fn test_string_interpreted() {
        let source = "package main\nconst x = \"hello\"";
        let tree = parse_go(source);
        let ctx = create_context(&tree, source.as_bytes());
        let strategy = LiteralStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "interpreted_string_literal").unwrap();
        assert!(strategy.can_handle(&node, &ctx));

        let value = strategy.resolve(&node, &ctx);
        assert!(value.is_resolved);
        assert_eq!(value.string_values, vec!["hello"]);
    }

    #[test]
    fn test_string_raw() {
        let source = "package main\nconst x = `raw string`";
        let tree = parse_go(source);
        let ctx = create_context(&tree, source.as_bytes());
        let strategy = LiteralStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "raw_string_literal").unwrap();
        assert!(strategy.can_handle(&node, &ctx));

        let value = strategy.resolve(&node, &ctx);
        assert!(value.is_resolved);
        assert_eq!(value.string_values, vec!["raw string"]);
    }

    #[test]
    fn test_boolean_true() {
        let source = "package main\nvar x = true";
        let tree = parse_go(source);
        let ctx = create_context(&tree, source.as_bytes());
        let strategy = LiteralStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "true").unwrap();
        assert!(strategy.can_handle(&node, &ctx));

        let value = strategy.resolve(&node, &ctx);
        assert!(value.is_resolved);
        assert_eq!(value.string_values, vec!["true"]);
    }

    #[test]
    fn test_boolean_false() {
        let source = "package main\nvar x = false";
        let tree = parse_go(source);
        let ctx = create_context(&tree, source.as_bytes());
        let strategy = LiteralStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "false").unwrap();
        assert!(strategy.can_handle(&node, &ctx));

        let value = strategy.resolve(&node, &ctx);
        assert!(value.is_resolved);
        assert_eq!(value.string_values, vec!["false"]);
    }

    #[test]
    fn test_nil() {
        let source = "package main\nvar x = nil";
        let tree = parse_go(source);
        let ctx = create_context(&tree, source.as_bytes());
        let strategy = LiteralStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "nil").unwrap();
        assert!(strategy.can_handle(&node, &ctx));

        let value = strategy.resolve(&node, &ctx);
        assert!(value.is_resolved);
        assert_eq!(value.string_values, vec!["nil"]);
    }

    #[test]
    fn test_cannot_handle_identifier() {
        let source = "package main\nvar x = someVar";
        let tree = parse_go(source);
        let ctx = create_context(&tree, source.as_bytes());
        let strategy = LiteralStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "identifier").unwrap();
        assert!(!strategy.can_handle(&node, &ctx));
    }

    #[test]
    fn test_float_literal() {
        let source = "package main\nconst x = 3.14";
        let tree = parse_go(source);
        let ctx = create_context(&tree, source.as_bytes());
        let strategy = LiteralStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "float_literal").unwrap();
        assert!(strategy.can_handle(&node, &ctx));

        let value = strategy.resolve(&node, &ctx);
        assert!(value.is_resolved);
        assert_eq!(value.string_values, vec!["3.14"]);
    }

    #[test]
    fn test_float_whole_number() {
        let source = "package main\nconst x = 100.0";
        let tree = parse_go(source);
        let ctx = create_context(&tree, source.as_bytes());
        let strategy = LiteralStrategy::new();

        let node = find_first_node_of_kind(tree.root_node(), "float_literal").unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![100]);
    }
}
