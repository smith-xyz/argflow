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

pub fn resolve_initializer<'a>(
    strategy: &CompositeStrategy,
    node: &Node<'a>,
    ctx: &Context<'a>,
) -> Value {
    strategy.collect_c_designated_initializers(node, ctx)
}
