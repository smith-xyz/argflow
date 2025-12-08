use crate::engine::Context;
use tree_sitter::Node;

pub fn get_selector<'a>(_node: &Node<'a>, _ctx: &Context<'a>) -> Option<(Node<'a>, String)> {
    let kind = _node.kind();

    if kind == "field_expression" {
        let value = _node.child_by_field_name("value")?;
        let field = _node.child_by_field_name("field")?;
        return Some((value, _ctx.get_node_text(&field)));
    }

    if kind == "scoped_identifier" {
        let path = _node.child_by_field_name("path")?;
        let name = _node.child_by_field_name("name")?;
        return Some((path, _ctx.get_node_text(&name)));
    }

    None
}
