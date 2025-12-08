use crate::engine::Context;
use tree_sitter::Node;

pub fn get_unary<'a>(_node: &Node<'a>, _ctx: &Context<'a>) -> Option<(String, Node<'a>)> {
    let kind = _node.kind();

    if kind == "reference_expression" {
        let operand = _node.child_by_field_name("value")?;
        return Some(("&".to_string(), operand));
    }

    if kind == "dereference_expression" {
        let operand = _node.child_by_field_name("value")?;
        return Some(("*".to_string(), operand));
    }

    if kind == "unary_expression" {
        let operand = _node.child(1)?;

        if let Some(op_node) = _node.child(0) {
            if !op_node.is_named() {
                let op_text = _ctx.get_node_text(&op_node);
                return Some((op_text, operand));
            }
        }
    }

    None
}
