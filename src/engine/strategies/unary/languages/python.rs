use crate::engine::Context;
use tree_sitter::Node;

pub fn get_unary<'a>(_node: &Node<'a>, _ctx: &Context<'a>) -> Option<(String, Node<'a>)> {
    let kind = _node.kind();

    if kind == "not_operator" {
        let operand = _node.child_by_field_name("argument")?;
        return Some(("not".to_string(), operand));
    }

    if kind == "unary_operator" {
        let operand = _node.child_by_field_name("argument")?;
        let op = _node.child_by_field_name("operator")?;
        let op_text = _ctx.get_node_text(&op);
        return Some((op_text, operand));
    }

    None
}
