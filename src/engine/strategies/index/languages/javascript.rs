use tree_sitter::Node;

pub fn get_object_index<'a>(_node: &Node<'a>) -> Option<(Node<'a>, Node<'a>)> {
    let object = _node.child_by_field_name("object")?;
    let index = _node.child_by_field_name("index")?;
    Some((object, index))
}
