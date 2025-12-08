use crate::engine::Context;
use tree_sitter::Node;

pub fn get_selector<'a>(_node: &Node<'a>, _ctx: &Context<'a>) -> Option<(Node<'a>, String)> {
    let object = _node.child_by_field_name("object")?;
    let property = _node.child_by_field_name("property")?;
    Some((object, _ctx.get_node_text(&property)))
}
