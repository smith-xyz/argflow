use tree_sitter::Node;

pub fn get_object_index<'a>(_node: &Node<'a>) -> Option<(Node<'a>, Node<'a>)> {
    let mut named_children = Vec::new();
    let mut cursor = _node.walk();
    for child in _node.children(&mut cursor) {
        if child.is_named() {
            named_children.push(child);
        }
    }
    if named_children.len() >= 2 {
        Some((named_children[0], named_children[1]))
    } else {
        None
    }
}
