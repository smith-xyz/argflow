use crate::engine::Context;
use tree_sitter::{Node, TreeCursor};

use super::super::IdentifierStrategy;

pub fn find_declaration<'a>(
    strategy: &IdentifierStrategy,
    name: &str,
    scope_node: Node<'a>,
    use_position: usize,
    ctx: &Context<'a>,
) -> Option<Node<'a>> {
    let mut cursor = scope_node.walk();
    search_declarations(strategy, &mut cursor, scope_node, name, use_position, ctx)
}

pub fn find_file_level_const<'a>(
    strategy: &IdentifierStrategy,
    name: &str,
    root: Node<'a>,
    use_position: usize,
    ctx: &Context<'a>,
) -> Option<Node<'a>> {
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.start_byte() >= use_position {
            continue;
        }

        match child.kind() {
            "const_declaration" => {
                if let Some(value) = extract_const_decl(strategy, child, name, ctx) {
                    return Some(value);
                }
            }
            "var_declaration" => {
                if let Some(value) = extract_var_decl(strategy, child, name, ctx) {
                    return Some(value);
                }
            }
            _ => {}
        }
    }
    None
}

fn search_declarations<'a>(
    _strategy: &IdentifierStrategy,
    cursor: &mut TreeCursor<'a>,
    node: Node<'a>,
    name: &str,
    use_position: usize,
    ctx: &Context<'a>,
) -> Option<Node<'a>> {
    let mut result: Option<Node<'a>> = None;

    for child in node.children(cursor) {
        if child.start_byte() >= use_position {
            continue;
        }

        match child.kind() {
            "short_var_declaration" => {
                if let Some(value) = extract_short_var(_strategy, child, name, ctx) {
                    result = Some(value);
                }
            }
            "var_declaration" => {
                if let Some(value) = extract_var_decl(_strategy, child, name, ctx) {
                    result = Some(value);
                }
            }
            "const_declaration" => {
                if let Some(value) = extract_const_decl(_strategy, child, name, ctx) {
                    result = Some(value);
                }
            }
            "assignment_statement" => {
                if let Some(value) = extract_assignment(_strategy, child, name, ctx) {
                    result = Some(value);
                }
            }
            "block" | "function_body" | "statement_list" => {
                let mut inner_cursor = child.walk();
                if let Some(found) = search_declarations(
                    _strategy,
                    &mut inner_cursor,
                    child,
                    name,
                    use_position,
                    ctx,
                ) {
                    result = Some(found);
                }
            }
            _ => {}
        }
    }
    result
}

fn extract_short_var<'a>(
    _strategy: &IdentifierStrategy,
    node: Node<'a>,
    name: &str,
    ctx: &Context<'a>,
) -> Option<Node<'a>> {
    let left = node.child_by_field_name("left")?;
    let right = node.child_by_field_name("right")?;

    let mut cursor = left.walk();
    let names: Vec<_> = left
        .children(&mut cursor)
        .filter(|c| c.is_named())
        .collect();

    for (i, name_node) in names.iter().enumerate() {
        if ctx.get_node_text(name_node) == name {
            let mut value_cursor = right.walk();
            let values: Vec<_> = right
                .children(&mut value_cursor)
                .filter(|c| c.is_named())
                .collect();
            return values.get(i).copied();
        }
    }
    None
}

fn extract_var_decl<'a>(
    _strategy: &IdentifierStrategy,
    node: Node<'a>,
    name: &str,
    ctx: &Context<'a>,
) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "var_spec" {
            if let Some(found) = extract_spec(_strategy, child, name, ctx) {
                return Some(found);
            }
        }
    }
    None
}

fn extract_const_decl<'a>(
    _strategy: &IdentifierStrategy,
    node: Node<'a>,
    name: &str,
    ctx: &Context<'a>,
) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "const_spec" {
            if let Some(found) = extract_spec(_strategy, child, name, ctx) {
                return Some(found);
            }
        }
    }
    None
}

fn extract_spec<'a>(
    _strategy: &IdentifierStrategy,
    spec: Node<'a>,
    name: &str,
    ctx: &Context<'a>,
) -> Option<Node<'a>> {
    let mut names: Vec<Node> = Vec::new();
    let mut values: Vec<Node> = Vec::new();

    let mut cursor = spec.walk();
    for child in spec.children(&mut cursor) {
        if child.kind() == "identifier" {
            names.push(child);
        }
    }

    if let Some(value_node) = spec.child_by_field_name("value") {
        let mut value_cursor = value_node.walk();
        for child in value_node.children(&mut value_cursor) {
            if child.is_named() {
                values.push(child);
            }
        }
    }

    for (i, name_node) in names.iter().enumerate() {
        if ctx.get_node_text(name_node) == name {
            return values.get(i).copied();
        }
    }

    None
}

fn extract_assignment<'a>(
    _strategy: &IdentifierStrategy,
    node: Node<'a>,
    name: &str,
    ctx: &Context<'a>,
) -> Option<Node<'a>> {
    let left = node.child_by_field_name("left")?;
    let right = node.child_by_field_name("right")?;

    let mut cursor = left.walk();
    let names: Vec<_> = left
        .children(&mut cursor)
        .filter(|c| c.is_named())
        .collect();

    for (i, name_node) in names.iter().enumerate() {
        if ctx.get_node_text(name_node) == name {
            let mut value_cursor = right.walk();
            let values: Vec<_> = right
                .children(&mut value_cursor)
                .filter(|c| c.is_named())
                .collect();
            return values.get(i).copied();
        }
    }
    None
}
