use crate::engine::{Context, UnresolvedSource, Value};
use tree_sitter::Node;

use super::super::CompositeStrategy;

pub fn resolve_array<'a>(
    strategy: &CompositeStrategy,
    node: &Node<'a>,
    ctx: &Context<'a>,
) -> Value {
    strategy.collect_array_elements(node, ctx)
}

pub fn resolve_object<'a>(
    _strategy: &CompositeStrategy,
    _node: &Node<'a>,
    _ctx: &Context<'a>,
) -> Value {
    Value::unextractable(UnresolvedSource::NotImplemented)
}
