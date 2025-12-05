use std::collections::HashMap;
use tree_sitter::Node;

#[derive(Debug, Clone)]
pub struct ScopeEntry {
    pub decl_start: usize,
    pub decl_end: usize,
}

impl ScopeEntry {
    pub fn new(decl_start: usize, decl_end: usize) -> Self {
        Self {
            decl_start,
            decl_end,
        }
    }

    pub fn from_node(node: &Node) -> Self {
        Self {
            decl_start: node.start_byte(),
            decl_end: node.end_byte(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Scope {
    pub variables: HashMap<String, ScopeEntry>,
}

impl Scope {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_entry_new() {
        let entry = ScopeEntry::new(10, 20);
        assert_eq!(entry.decl_start, 10);
        assert_eq!(entry.decl_end, 20);
    }

    #[test]
    fn test_scope_new_is_empty() {
        let scope = Scope::new();
        assert!(scope.variables.is_empty());
    }

    #[test]
    fn test_scope_add_variable() {
        let mut scope = Scope::new();
        scope
            .variables
            .insert("x".to_string(), ScopeEntry::new(0, 10));
        assert!(scope.variables.contains_key("x"));
    }
}
