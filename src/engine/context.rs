use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::rc::Rc;
use tree_sitter::{Node, Tree};

use super::file_cache::{FileCache, FunctionInfo};
use super::scope::{Scope, ScopeEntry};

const MAX_CACHE_SIZE: usize = 10_000;

pub struct Context<'a> {
    tree: &'a Tree,
    source_code: &'a [u8],
    file_path: String,
    language: String,
    node_mappings: HashMap<String, HashSet<String>>,
    scopes: RefCell<Vec<Scope>>,
    constants: RefCell<HashMap<String, ScopeEntry>>,
    file_cache: Option<Rc<RefCell<FileCache>>>,
    value_cache: RefCell<HashMap<usize, crate::Value>>,
    visited_nodes: RefCell<HashSet<usize>>,
}

impl<'a> Context<'a> {
    pub fn new(
        tree: &'a Tree,
        source_code: &'a [u8],
        file_path: String,
        language: String,
        node_mappings: HashMap<String, HashSet<String>>,
    ) -> Self {
        Self {
            tree,
            source_code,
            file_path,
            language,
            node_mappings,
            scopes: RefCell::new(Vec::new()),
            constants: RefCell::new(HashMap::new()),
            file_cache: None,
            value_cache: RefCell::new(HashMap::new()),
            visited_nodes: RefCell::new(HashSet::new()),
        }
    }

    pub fn with_file_cache(
        tree: &'a Tree,
        source_code: &'a [u8],
        file_path: String,
        language: String,
        node_mappings: HashMap<String, HashSet<String>>,
        file_cache: Rc<RefCell<FileCache>>,
    ) -> Self {
        Self {
            tree,
            source_code,
            file_path,
            language,
            node_mappings,
            scopes: RefCell::new(Vec::new()),
            constants: RefCell::new(HashMap::new()),
            file_cache: Some(file_cache),
            value_cache: RefCell::new(HashMap::new()),
            visited_nodes: RefCell::new(HashSet::new()),
        }
    }

    pub fn tree(&self) -> &Tree {
        self.tree
    }

    pub fn source_code(&self) -> &[u8] {
        self.source_code
    }

    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn node_mappings(&self) -> &HashMap<String, HashSet<String>> {
        &self.node_mappings
    }

    pub fn is_node_type(&self, node: &Node, semantic_type: &str) -> bool {
        if let Some(types) = self.node_mappings.get(semantic_type) {
            types.contains(node.kind())
        } else {
            false
        }
    }

    pub fn get_node_text(&self, node: &Node) -> String {
        let start = node.start_byte();
        let end = node.end_byte();

        if start > end || end > self.source_code.len() {
            return String::new();
        }

        String::from_utf8_lossy(&self.source_code[start..end]).to_string()
    }

    pub fn get_field_text(&self, node: &Node, field_name: &str) -> Option<String> {
        node.child_by_field_name(field_name)
            .map(|child| self.get_node_text(&child))
    }

    pub fn get_child_text(&self, node: &Node, index: usize) -> Option<String> {
        node.child(index).map(|child| self.get_node_text(&child))
    }

    pub fn unquote_string(&self, text: &str) -> String {
        let text = text.trim();

        if text.len() < 2 {
            return text.to_string();
        }

        let first = text.chars().next().unwrap();
        let last = text.chars().last().unwrap();

        let inner = if (first == '"' && last == '"')
            || (first == '\'' && last == '\'')
            || (first == '`' && last == '`')
        {
            &text[1..text.len() - 1]
        } else {
            text
        };

        inner
            .replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\r", "\r")
            .replace("\\\\", "\\")
            .replace("\\\"", "\"")
            .replace("\\'", "'")
    }

    pub fn parse_int_literal(&self, text: &str) -> Option<i64> {
        let text = text.trim().replace("_", "");

        if text.starts_with("0x") || text.starts_with("0X") {
            i64::from_str_radix(&text[2..], 16).ok()
        } else if text.starts_with("0o") || text.starts_with("0O") {
            i64::from_str_radix(&text[2..], 8).ok()
        } else if text.starts_with("0b") || text.starts_with("0B") {
            i64::from_str_radix(&text[2..], 2).ok()
        } else if text.starts_with("0") && text.len() > 1 && !text.contains('.') {
            // Go-style octal (0755)
            i64::from_str_radix(&text[1..], 8)
                .ok()
                .or_else(|| text.parse().ok())
        } else {
            text.parse().ok()
        }
    }

    pub fn get_named_children(&self, node: &Node<'a>) -> Vec<Node<'a>> {
        let mut children = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.is_named() {
                children.push(child);
            }
        }

        children
    }

    pub fn has_visited(&self, node: &Node) -> bool {
        self.visited_nodes.borrow().contains(&node.id())
    }

    pub fn mark_visited(&self, node: &Node) {
        self.visited_nodes.borrow_mut().insert(node.id());
    }

    pub fn unmark_visited(&self, node: &Node) {
        self.visited_nodes.borrow_mut().remove(&node.id());
    }

    pub fn get_cached_value(&self, node: &Node) -> Option<crate::Value> {
        self.value_cache.borrow().get(&node.id()).cloned()
    }

    pub fn cache_value(&self, node: &Node, value: crate::Value) {
        let mut cache = self.value_cache.borrow_mut();
        if cache.len() >= MAX_CACHE_SIZE {
            let first_key = *cache.keys().next().unwrap();
            cache.remove(&first_key);
        }
        cache.insert(node.id(), value);
    }

    pub fn push_scope(&self) {
        self.scopes.borrow_mut().push(Scope::new());
    }

    pub fn pop_scope(&self) {
        self.scopes.borrow_mut().pop();
    }

    pub fn scope_depth(&self) -> usize {
        self.scopes.borrow().len()
    }

    pub fn add_variable(&self, name: String, entry: ScopeEntry) {
        let mut scopes = self.scopes.borrow_mut();
        if let Some(scope) = scopes.last_mut() {
            scope.variables.insert(name, entry);
        }
    }

    pub fn find_variable(&self, name: &str) -> Option<ScopeEntry> {
        let scopes = self.scopes.borrow();
        for scope in scopes.iter().rev() {
            if let Some(entry) = scope.variables.get(name) {
                return Some(entry.clone());
            }
        }
        None
    }

    pub fn add_constant(&self, name: String, entry: ScopeEntry) {
        self.constants.borrow_mut().insert(name, entry);
    }

    pub fn find_constant(&self, name: &str) -> Option<ScopeEntry> {
        self.constants.borrow().get(name).cloned()
    }

    pub fn find_cross_file_constant(&self, name: &str) -> Option<crate::Value> {
        let cache = self.file_cache.as_ref()?;
        let cache = cache.borrow();

        if let Some(parent) = Path::new(&self.file_path).parent() {
            cache.find_constant_in_package(name, &parent.to_string_lossy())
        } else {
            cache.find_constant(name)
        }
    }

    pub fn find_cross_file_function(&self, name: &str) -> Option<FunctionInfo> {
        let cache = self.file_cache.as_ref()?;
        let cache = cache.borrow();
        cache.find_function(name).cloned()
    }

    pub fn has_file_cache(&self) -> bool {
        self.file_cache.is_some()
    }

    pub fn package_dir(&self) -> Option<String> {
        Path::new(&self.file_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
    }

    pub fn find_node_at_position(&self, start: usize, end: usize) -> Option<Node<'a>> {
        let root = self.tree.root_node();
        Self::find_node_recursive(root, start, end)
    }

    fn find_node_recursive(node: Node<'a>, start: usize, end: usize) -> Option<Node<'a>> {
        if node.start_byte() == start && node.end_byte() == end {
            return Some(node);
        }

        if node.start_byte() > end || node.end_byte() < start {
            return None;
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = Self::find_node_recursive(child, start, end) {
                return Some(found);
            }
        }

        None
    }

    pub fn find_declaration_node(&self, entry: &ScopeEntry) -> Option<Node<'a>> {
        self.find_node_at_position(entry.decl_start, entry.decl_end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::file_cache::CachedFileEntry;

    fn create_test_context<'a>(tree: &'a Tree, source: &'a [u8]) -> Context<'a> {
        Context::new(
            tree,
            source,
            "test.go".to_string(),
            "go".to_string(),
            HashMap::new(),
        )
    }

    fn parse_go_source(source: &str) -> Tree {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    #[test]
    fn test_scope_push_pop() {
        let source = b"package main";
        let tree = parse_go_source("package main");
        let ctx = create_test_context(&tree, source);

        assert_eq!(ctx.scope_depth(), 0);

        ctx.push_scope();
        assert_eq!(ctx.scope_depth(), 1);

        ctx.push_scope();
        assert_eq!(ctx.scope_depth(), 2);

        ctx.pop_scope();
        assert_eq!(ctx.scope_depth(), 1);

        ctx.pop_scope();
        assert_eq!(ctx.scope_depth(), 0);
    }

    #[test]
    fn test_variable_add_and_find_single_scope() {
        let source = b"package main";
        let tree = parse_go_source("package main");
        let ctx = create_test_context(&tree, source);

        ctx.push_scope();
        ctx.add_variable("iterations".to_string(), ScopeEntry::new(10, 20));

        let found = ctx.find_variable("iterations");
        assert!(found.is_some());
        let entry = found.unwrap();
        assert_eq!(entry.decl_start, 10);
        assert_eq!(entry.decl_end, 20);
    }

    #[test]
    fn test_variable_not_found() {
        let source = b"package main";
        let tree = parse_go_source("package main");
        let ctx = create_test_context(&tree, source);

        ctx.push_scope();

        let found = ctx.find_variable("nonexistent");
        assert!(found.is_none());
    }

    #[test]
    fn test_variable_requires_scope() {
        let source = b"package main";
        let tree = parse_go_source("package main");
        let ctx = create_test_context(&tree, source);

        ctx.add_variable("x".to_string(), ScopeEntry::new(0, 10));

        let found = ctx.find_variable("x");
        assert!(found.is_none());
    }

    #[test]
    fn test_inner_scope_shadows_outer() {
        let source = b"package main";
        let tree = parse_go_source("package main");
        let ctx = create_test_context(&tree, source);

        ctx.push_scope();
        ctx.add_variable("x".to_string(), ScopeEntry::new(10, 20));

        ctx.push_scope();
        ctx.add_variable("x".to_string(), ScopeEntry::new(30, 40));

        let found = ctx.find_variable("x").unwrap();
        assert_eq!(found.decl_start, 30);
        assert_eq!(found.decl_end, 40);

        ctx.pop_scope();

        let found = ctx.find_variable("x").unwrap();
        assert_eq!(found.decl_start, 10);
        assert_eq!(found.decl_end, 20);
    }

    #[test]
    fn test_search_traverses_scopes() {
        let source = b"package main";
        let tree = parse_go_source("package main");
        let ctx = create_test_context(&tree, source);

        ctx.push_scope();
        ctx.add_variable("outer".to_string(), ScopeEntry::new(0, 10));

        ctx.push_scope();
        ctx.add_variable("inner".to_string(), ScopeEntry::new(20, 30));

        assert!(ctx.find_variable("outer").is_some());
        assert!(ctx.find_variable("inner").is_some());
    }

    #[test]
    fn test_scope_isolation_after_pop() {
        let source = b"package main";
        let tree = parse_go_source("package main");
        let ctx = create_test_context(&tree, source);

        ctx.push_scope();

        ctx.push_scope();
        ctx.add_variable("inner_only".to_string(), ScopeEntry::new(0, 10));
        ctx.pop_scope();

        assert!(ctx.find_variable("inner_only").is_none());
    }

    #[test]
    fn test_constants_separate_from_variables() {
        let source = b"package main";
        let tree = parse_go_source("package main");
        let ctx = create_test_context(&tree, source);

        ctx.add_constant("MAX_ITERATIONS".to_string(), ScopeEntry::new(0, 20));

        let found = ctx.find_constant("MAX_ITERATIONS");
        assert!(found.is_some());

        assert!(ctx.find_variable("MAX_ITERATIONS").is_none());
    }

    #[test]
    fn test_find_node_at_position() {
        let source = "package main\nconst X = 10000";
        let tree = parse_go_source(source);
        let ctx = create_test_context(&tree, source.as_bytes());

        let root = tree.root_node();
        let const_decl = root.child(1).unwrap();
        let const_spec = const_decl.child(1).unwrap();
        let expr_list = const_spec.child(2).unwrap();
        let value_node = expr_list.child(0).unwrap();

        let start = value_node.start_byte();
        let end = value_node.end_byte();

        let found = ctx.find_node_at_position(start, end);
        assert!(found.is_some());

        let found_node = found.unwrap();
        assert_eq!(found_node.start_byte(), start);
        assert_eq!(found_node.end_byte(), end);
    }

    #[test]
    fn test_find_declaration_node() {
        let source = "package main\nconst X = 10000";
        let tree = parse_go_source(source);
        let ctx = create_test_context(&tree, source.as_bytes());

        let root = tree.root_node();
        let const_decl = root.child(1).unwrap();
        let const_spec = const_decl.child(1).unwrap();
        let expr_list = const_spec.child(2).unwrap();
        let value_node = expr_list.child(0).unwrap();

        let entry = ScopeEntry::from_node(&value_node);
        let found = ctx.find_declaration_node(&entry);

        assert!(found.is_some());
        let found_node = found.unwrap();
        assert_eq!(ctx.get_node_text(&found_node), "10000");
    }

    #[test]
    fn test_scope_entry_from_node() {
        let source = "package main\nconst X = 10000";
        let tree = parse_go_source(source);

        let root = tree.root_node();
        let const_decl = root.child(1).unwrap();
        let const_spec = const_decl.child(1).unwrap();
        let expr_list = const_spec.child(2).unwrap();
        let value_node = expr_list.child(0).unwrap();

        let entry = ScopeEntry::from_node(&value_node);

        assert_eq!(entry.decl_start, value_node.start_byte());
        assert_eq!(entry.decl_end, value_node.end_byte());
    }

    #[test]
    fn test_context_with_file_cache() {
        let source = b"package main";
        let tree = parse_go_source("package main");

        let mut file_cache = FileCache::new();
        let mut constants = HashMap::new();
        constants.insert(
            "CROSS_FILE_CONST".to_string(),
            crate::Value::resolved_int(42),
        );
        file_cache.add_file(
            "/pkg/other.go".to_string(),
            CachedFileEntry {
                constants,
                functions: HashMap::new(),
            },
        );

        let cache_rc = Rc::new(RefCell::new(file_cache));

        let ctx = Context::with_file_cache(
            &tree,
            source,
            "/pkg/main.go".to_string(),
            "go".to_string(),
            HashMap::new(),
            cache_rc,
        );

        assert!(ctx.has_file_cache());

        let found = ctx.find_cross_file_constant("CROSS_FILE_CONST");
        assert!(found.is_some());
        let value = found.unwrap();
        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![42]);
    }

    #[test]
    fn test_context_without_file_cache() {
        let source = b"package main";
        let tree = parse_go_source("package main");
        let ctx = create_test_context(&tree, source);

        assert!(!ctx.has_file_cache());
        assert!(ctx.find_cross_file_constant("ANY").is_none());
    }

    #[test]
    fn test_get_node_text_boundary_check() {
        let source = b"package main";
        let tree = parse_go_source("package main");
        let ctx = create_test_context(&tree, source);

        let root = tree.root_node();
        let text = ctx.get_node_text(&root);
        assert_eq!(text, "package main");
    }

    #[test]
    fn test_get_field_text() {
        let source = "package main\nconst X = 10000";
        let tree = parse_go_source(source);
        let ctx = create_test_context(&tree, source.as_bytes());

        let root = tree.root_node();
        let const_decl = root.child(1).unwrap();

        // const_declaration has no "name" field, but we can test field access pattern
        let specs = ctx.get_field_text(&const_decl, "specs");
        assert!(specs.is_none()); // This field doesn't exist in tree-sitter-go

        // Test with a node that does have the field structure
        let const_spec = const_decl.child(1).unwrap();
        let name_text = ctx.get_child_text(&const_spec, 0);
        assert_eq!(name_text, Some("X".to_string()));
    }

    #[test]
    fn test_get_child_text() {
        let source = "package main";
        let tree = parse_go_source(source);
        let ctx = create_test_context(&tree, source.as_bytes());

        let root = tree.root_node();
        let pkg_clause = root.child(0).unwrap();

        let keyword = ctx.get_child_text(&pkg_clause, 0);
        assert_eq!(keyword, Some("package".to_string()));

        let name = ctx.get_child_text(&pkg_clause, 1);
        assert_eq!(name, Some("main".to_string()));

        let nonexistent = ctx.get_child_text(&pkg_clause, 99);
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_unquote_string_double_quotes() {
        let source = b"package main";
        let tree = parse_go_source("package main");
        let ctx = create_test_context(&tree, source);

        assert_eq!(ctx.unquote_string("\"hello\""), "hello");
        assert_eq!(ctx.unquote_string("\"hello\\nworld\""), "hello\nworld");
        assert_eq!(ctx.unquote_string("\"hello\\tworld\""), "hello\tworld");
        assert_eq!(ctx.unquote_string("\"hello\\\"world\""), "hello\"world");
    }

    #[test]
    fn test_unquote_string_single_quotes() {
        let source = b"package main";
        let tree = parse_go_source("package main");
        let ctx = create_test_context(&tree, source);

        assert_eq!(ctx.unquote_string("'hello'"), "hello");
        assert_eq!(ctx.unquote_string("'it\\'s'"), "it's");
    }

    #[test]
    fn test_unquote_string_backticks() {
        let source = b"package main";
        let tree = parse_go_source("package main");
        let ctx = create_test_context(&tree, source);

        assert_eq!(ctx.unquote_string("`raw string`"), "raw string");
    }

    #[test]
    fn test_unquote_string_no_quotes() {
        let source = b"package main";
        let tree = parse_go_source("package main");
        let ctx = create_test_context(&tree, source);

        assert_eq!(ctx.unquote_string("no_quotes"), "no_quotes");
        assert_eq!(ctx.unquote_string("x"), "x");
        assert_eq!(ctx.unquote_string(""), "");
    }

    #[test]
    fn test_parse_int_literal_decimal() {
        let source = b"package main";
        let tree = parse_go_source("package main");
        let ctx = create_test_context(&tree, source);

        assert_eq!(ctx.parse_int_literal("10000"), Some(10000));
        assert_eq!(ctx.parse_int_literal("0"), Some(0));
        assert_eq!(ctx.parse_int_literal("-42"), Some(-42));
        assert_eq!(ctx.parse_int_literal("1_000_000"), Some(1000000));
    }

    #[test]
    fn test_parse_int_literal_hex() {
        let source = b"package main";
        let tree = parse_go_source("package main");
        let ctx = create_test_context(&tree, source);

        assert_eq!(ctx.parse_int_literal("0xFF"), Some(255));
        assert_eq!(ctx.parse_int_literal("0x10"), Some(16));
        assert_eq!(ctx.parse_int_literal("0X1A"), Some(26));
    }

    #[test]
    fn test_parse_int_literal_octal() {
        let source = b"package main";
        let tree = parse_go_source("package main");
        let ctx = create_test_context(&tree, source);

        assert_eq!(ctx.parse_int_literal("0o755"), Some(493));
        assert_eq!(ctx.parse_int_literal("0O10"), Some(8));
        // Go-style octal (leading zero)
        assert_eq!(ctx.parse_int_literal("0755"), Some(493));
    }

    #[test]
    fn test_parse_int_literal_binary() {
        let source = b"package main";
        let tree = parse_go_source("package main");
        let ctx = create_test_context(&tree, source);

        assert_eq!(ctx.parse_int_literal("0b1010"), Some(10));
        assert_eq!(ctx.parse_int_literal("0B1111"), Some(15));
    }

    #[test]
    fn test_parse_int_literal_invalid() {
        let source = b"package main";
        let tree = parse_go_source("package main");
        let ctx = create_test_context(&tree, source);

        assert_eq!(ctx.parse_int_literal("not_a_number"), None);
        assert_eq!(ctx.parse_int_literal("12.34"), None);
    }

    #[test]
    fn test_get_named_children() {
        let source = "package main\nconst X = 10000";
        let tree = parse_go_source(source);
        let ctx = create_test_context(&tree, source.as_bytes());

        let root = tree.root_node();
        let named_children = ctx.get_named_children(&root);

        // Should have package_clause and const_declaration
        assert_eq!(named_children.len(), 2);
        assert_eq!(named_children[0].kind(), "package_clause");
        assert_eq!(named_children[1].kind(), "const_declaration");
    }
}
