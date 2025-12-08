use crate::engine::{Context, Value};
use tree_sitter::Node;

use super::super::CallStrategy;

pub fn extract_return<'a>(
    strategy: &CallStrategy,
    node: &Node<'a>,
    ctx: &Context<'a>,
) -> Option<Value> {
    let mut cursor = node.walk();
    let children: Vec<_> = node
        .children(&mut cursor)
        .filter(|c| c.is_named())
        .collect();

    if children.is_empty() {
        return None;
    }

    if children.len() == 1 {
        let child = children[0];
        if child.kind() == "expression_list" {
            return Some(strategy.resolve_expression_list(&child, ctx));
        }
        return Some(strategy.resolve_value_node(child, ctx));
    }

    Some(strategy.resolve_multiple_values(&children, ctx))
}
