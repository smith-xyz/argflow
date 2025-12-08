use tree_sitter::Node;

pub fn get_object_index<'a>(_node: &Node<'a>) -> Option<(Node<'a>, Node<'a>)> {
    let array = _node.child_by_field_name("array")?;
    let index = _node.child_by_field_name("index")?;
    Some((array, index))
}
