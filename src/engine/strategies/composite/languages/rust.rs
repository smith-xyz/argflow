use crate::engine::Context;
use tree_sitter::Node;

use super::super::CompositeStrategy;
use crate::engine::Value;

pub fn resolve_array<'a>(
    strategy: &CompositeStrategy,
    node: &Node<'a>,
    ctx: &Context<'a>,
) -> Value {
    strategy.collect_array_elements(node, ctx)
}

pub fn resolve_struct<'a>(
    strategy: &CompositeStrategy,
    node: &Node<'a>,
    ctx: &Context<'a>,
) -> Value {
    if let Some(body) = node.child_by_field_name("body") {
        return strategy.collect_struct_fields(&body, ctx);
    }
    strategy.collect_struct_fields(node, ctx)
}
