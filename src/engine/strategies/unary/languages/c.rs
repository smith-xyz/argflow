use crate::engine::Context;
use tree_sitter::Node;

pub fn get_unary<'a>(_node: &Node<'a>, _ctx: &Context<'a>) -> Option<(String, Node<'a>)> {
    let operand = _node.child_by_field_name("argument")?;
    let op = _node.child_by_field_name("operator")?;
    let op_text = _ctx.get_node_text(&op);
    Some((op_text, operand))
}
