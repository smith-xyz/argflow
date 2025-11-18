/// Resolution context that carries state during AST traversal.
///
/// This provides the resolver with access to:
/// - The source file being analyzed
/// - Language-specific node type mappings
/// - Cached information for performance
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use tree_sitter::{Node, Tree};

const MAX_CACHE_SIZE: usize = 10_000;

pub struct Context<'a> {
    /// The Tree-sitter parse tree
    tree: &'a Tree,

    /// Source code bytes
    source_code: &'a [u8],

    /// File path being analyzed
    file_path: String,

    /// Language name (go, python, rust, etc.)
    language: String,

    /// Node type mappings for this language
    node_mappings: HashMap<String, HashSet<String>>,

    /// Cache of resolved values (optimization)
    value_cache: RefCell<HashMap<usize, crate::Value>>,

    /// Visited nodes (cycle detection)
    visited_nodes: RefCell<HashSet<usize>>,
}

impl<'a> Context<'a> {
    /// Create a new context
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
            value_cache: RefCell::new(HashMap::new()),
            visited_nodes: RefCell::new(HashSet::new()),
        }
    }

    /// Get the parse tree
    pub fn tree(&self) -> &Tree {
        self.tree
    }

    /// Get the source code bytes
    pub fn source_code(&self) -> &[u8] {
        self.source_code
    }

    /// Get the file path
    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    /// Get the language name
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Get node type mappings
    pub fn node_mappings(&self) -> &HashMap<String, HashSet<String>> {
        &self.node_mappings
    }

    /// Check if a tree-sitter node matches a semantic type
    ///
    /// Example:
    ///     is_node_type(node, "literal")
    ///     -> true if node.kind() in ["number_literal", "integer", "integer_literal"]
    pub fn is_node_type(&self, node: &Node, semantic_type: &str) -> bool {
        if let Some(types) = self.node_mappings.get(semantic_type) {
            types.contains(node.kind())
        } else {
            false
        }
    }

    /// Get the source code text for a node
    /// Uses lossy UTF-8 conversion to handle invalid sequences gracefully
    pub fn get_node_text(&self, node: &Node) -> String {
        let start = node.start_byte();
        let end = node.end_byte();
        String::from_utf8_lossy(&self.source_code[start..end]).to_string()
    }

    /// Check if we've already visited this node (cycle detection)
    pub fn has_visited(&self, node: &Node) -> bool {
        self.visited_nodes.borrow().contains(&node.id())
    }

    /// Mark a node as visited
    pub fn mark_visited(&self, node: &Node) {
        self.visited_nodes.borrow_mut().insert(node.id());
    }

    /// Unmark a node as visited (for cycle detection cleanup)
    pub fn unmark_visited(&self, node: &Node) {
        self.visited_nodes.borrow_mut().remove(&node.id());
    }

    /// Get a cached resolution result for a node
    pub fn get_cached_value(&self, node: &Node) -> Option<crate::Value> {
        self.value_cache.borrow().get(&node.id()).cloned()
    }

    /// Cache a resolution result for a node
    pub fn cache_value(&self, node: &Node, value: crate::Value) {
        let mut cache = self.value_cache.borrow_mut();
        if cache.len() >= MAX_CACHE_SIZE {
            let first_key = *cache.keys().next().unwrap();
            cache.remove(&first_key);
        }
        cache.insert(node.id(), value);
    }
}
