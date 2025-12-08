use crate::engine::{Context, Value};
use tree_sitter::Node;

use super::super::CallStrategy;

pub fn extract_return<'a>(
    strategy: &CallStrategy,
    node: &Node<'a>,
    ctx: &Context<'a>,
) -> Option<Value> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.is_named() {
            if child.kind() == "tuple_expression" {
                return Some(strategy.resolve_tuple(&child, ctx));
            }
            return Some(strategy.resolve_value_node(child, ctx));
        }
    }
    None
}
