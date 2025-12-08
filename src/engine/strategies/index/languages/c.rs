use tree_sitter::Node;

pub fn get_object_index<'a>(_node: &Node<'a>) -> Option<(Node<'a>, Node<'a>)> {
    let argument = _node.child_by_field_name("argument")?;
    let index = _node.child_by_field_name("index")?;
    Some((argument, index))
}
