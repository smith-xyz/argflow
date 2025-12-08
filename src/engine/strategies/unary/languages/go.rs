use crate::engine::{Context, UnaryOp};
use tree_sitter::Node;

pub fn get_unary<'a>(_node: &Node<'a>, _ctx: &Context<'a>) -> Option<(String, Node<'a>)> {
    let operand = _node.child_by_field_name("operand")?;

    let mut cursor = _node.walk();
    for child in _node.children(&mut cursor) {
        if !child.is_named() {
            let op_text = _ctx.get_node_text(&child);
            if UnaryOp::parse(&op_text).is_some() || op_text == "&" || op_text == "*" {
                return Some((op_text, operand));
            }
        }
    }
    None
}
