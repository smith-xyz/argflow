use crate::engine::{Context, UnresolvedSource, Value};
use tree_sitter::Node;

use super::super::CompositeStrategy;

pub fn resolve_array<'a>(
    strategy: &CompositeStrategy,
    node: &Node<'a>,
    ctx: &Context<'a>,
) -> Value {
    if let Some(body) = node.child_by_field_name("body") {
        return strategy.collect_array_elements(&body, ctx);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "literal_value" {
            return strategy.collect_array_elements(&child, ctx);
        }
    }

    Value::unextractable(UnresolvedSource::Unknown)
}

pub fn resolve_struct<'a>(
    strategy: &CompositeStrategy,
    node: &Node<'a>,
    ctx: &Context<'a>,
) -> Value {
    if let Some(body) = node.child_by_field_name("body") {
        return strategy.collect_struct_fields(&body, ctx);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "literal_value" {
            return strategy.collect_struct_fields(&child, ctx);
        }
    }

    Value::unextractable(UnresolvedSource::Unknown)
}
