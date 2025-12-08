use crate::engine::Context;
use tree_sitter::Node;

pub fn get_selector<'a>(_node: &Node<'a>, _ctx: &Context<'a>) -> Option<(Node<'a>, String)> {
    let operand = _node.child_by_field_name("operand")?;
    let field = _node.child_by_field_name("field")?;
    Some((operand, _ctx.get_node_text(&field)))
}
