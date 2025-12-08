pub mod context;
pub mod file_cache;
pub mod lang_features;
pub mod node_types;
pub mod operators;
pub mod scope;
pub mod sources;
pub mod strategies;
pub mod value;

pub use context::Context;
pub use file_cache::{CachedFileEntry, FileCache, FunctionInfo};
pub use node_types::{Language, NodeCategory, NodeTypes};
pub use operators::{BinaryOp, UnaryOp};
pub use scope::{Scope, ScopeEntry};
pub use sources::UnresolvedSource;
pub use value::Value;

use strategies::LiteralStrategy;
// TODO: Uncomment as strategies are implemented
// use strategies::UnaryStrategy;
// use strategies::BinaryStrategy;
// use strategies::IdentifierStrategy;
// use strategies::CallStrategy;
// use strategies::SelectorStrategy;
// use strategies::IndexStrategy;
// use strategies::CompositeStrategy;
use tree_sitter::Node;

const DEFAULT_MAX_DEPTH: usize = 50;

pub trait Strategy: Send + Sync {
    fn name(&self) -> &'static str;
    fn can_handle<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> bool;
    fn resolve<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value;
}

pub struct Resolver {
    strategies: Vec<Box<dyn Strategy>>,
    max_depth: usize,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            strategies: Self::default_strategies(),
            max_depth: DEFAULT_MAX_DEPTH,
        }
    }

    /// Returns the default strategy chain in order of complexity.
    /// Uncomment strategies as they are implemented.
    fn default_strategies() -> Vec<Box<dyn Strategy>> {
        vec![
            // Order matters: simpler strategies first
            Box::new(LiteralStrategy::new()),
            // Box::new(UnaryStrategy::new()),
            // Box::new(BinaryStrategy::new()),
            // Box::new(IdentifierStrategy::new()),
            // Box::new(CallStrategy::new()),
            // Box::new(SelectorStrategy::new()),
            // Box::new(IndexStrategy::new()),
            // Box::new(CompositeStrategy::new()),
        ]
    }

    pub fn builder() -> ResolverBuilder {
        ResolverBuilder::new()
    }

    pub fn resolve<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        self.resolve_with_depth(node, ctx, 0)
    }

    fn resolve_with_depth<'a>(&self, node: &Node<'a>, ctx: &Context<'a>, depth: usize) -> Value {
        if depth >= self.max_depth {
            return Value::unextractable(UnresolvedSource::CycleDetected);
        }

        if let Some(cached) = ctx.get_cached_value(node) {
            return cached;
        }

        if ctx.has_visited(node) {
            return Value::unextractable(UnresolvedSource::CycleDetected);
        }
        ctx.mark_visited(node);

        let result = self.try_strategies(node, ctx);

        ctx.cache_value(node, result.clone());
        ctx.unmark_visited(node);
        result
    }

    fn try_strategies<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        for strategy in &self.strategies {
            if strategy.can_handle(node, ctx) {
                return strategy.resolve(node, ctx);
            }
        }
        Value::unextractable(UnresolvedSource::NotImplemented)
    }

    pub fn strategy_count(&self) -> usize {
        self.strategies.len()
    }

    pub fn strategy_names(&self) -> Vec<&'static str> {
        self.strategies.iter().map(|s| s.name()).collect()
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ResolverBuilder {
    strategies: Vec<Box<dyn Strategy>>,
    max_depth: usize,
    include_defaults: bool,
}

impl ResolverBuilder {
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
            max_depth: DEFAULT_MAX_DEPTH,
            include_defaults: true,
        }
    }

    pub fn with_strategy<S: Strategy + 'static>(mut self, strategy: S) -> Self {
        self.strategies.push(Box::new(strategy));
        self
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn without_defaults(mut self) -> Self {
        self.include_defaults = false;
        self
    }

    pub fn build(mut self) -> Resolver {
        if self.include_defaults && self.strategies.is_empty() {
            self.strategies = Resolver::default_strategies();
        }

        Resolver {
            strategies: self.strategies,
            max_depth: self.max_depth,
        }
    }
}

impl Default for ResolverBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn parse_go(source: &str) -> tree_sitter::Tree {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    fn create_context<'a>(tree: &'a tree_sitter::Tree, source: &'a [u8]) -> Context<'a> {
        Context::new(
            tree,
            source,
            "test.go".to_string(),
            "go".to_string(),
            HashMap::new(),
        )
    }

    fn find_first_node_of_kind<'a>(
        node: tree_sitter::Node<'a>,
        kind: &str,
    ) -> Option<tree_sitter::Node<'a>> {
        if node.kind() == kind {
            return Some(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = find_first_node_of_kind(child, kind) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn test_resolver_default() {
        let resolver = Resolver::new();
        assert_eq!(resolver.strategy_count(), 1);
        assert_eq!(resolver.strategy_names(), vec!["literal"]);
    }

    #[test]
    fn test_resolver_builder_defaults() {
        let resolver = Resolver::builder().build();
        assert_eq!(resolver.strategy_count(), 1);
        assert_eq!(resolver.strategy_names(), vec!["literal"]);
    }

    #[test]
    fn test_resolver_builder_without_defaults() {
        let resolver = Resolver::builder().without_defaults().build();
        assert_eq!(resolver.strategy_count(), 0);
    }

    #[test]
    fn test_resolver_builder_custom_depth() {
        let resolver = Resolver::builder().with_max_depth(10).build();
        assert_eq!(resolver.max_depth, 10);
    }

    #[test]
    fn test_resolver_resolves_literal() {
        let source = "package main\nconst x = 10000";
        let tree = parse_go(source);
        let ctx = create_context(&tree, source.as_bytes());
        let resolver = Resolver::new();

        let node = find_first_node_of_kind(tree.root_node(), "int_literal").unwrap();
        let value = resolver.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![10000]);
    }

    #[test]
    fn test_resolver_caches_values() {
        let source = "package main\nconst x = 42";
        let tree = parse_go(source);
        let ctx = create_context(&tree, source.as_bytes());
        let resolver = Resolver::new();

        let node = find_first_node_of_kind(tree.root_node(), "int_literal").unwrap();

        // First resolve
        let value1 = resolver.resolve(&node, &ctx);
        assert!(value1.is_resolved);

        // Second resolve should hit cache
        let value2 = resolver.resolve(&node, &ctx);
        assert_eq!(value1.int_values, value2.int_values);
    }

    #[test]
    fn test_resolver_unhandled_node() {
        let source = "package main\nvar x = someIdentifier";
        let tree = parse_go(source);
        let ctx = create_context(&tree, source.as_bytes());
        let resolver = Resolver::new();

        let node = find_node_by_text(tree.root_node(), "identifier", "someIdentifier", &ctx)
            .expect("should find someIdentifier");
        let value = resolver.resolve(&node, &ctx);

        assert!(!value.is_resolved);
    }

    fn find_node_by_text<'a>(
        node: tree_sitter::Node<'a>,
        kind: &str,
        text: &str,
        ctx: &Context<'a>,
    ) -> Option<tree_sitter::Node<'a>> {
        if node.kind() == kind && ctx.get_node_text(&node) == text {
            return Some(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = find_node_by_text(child, kind, text, ctx) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn test_resolver_depth_limit() {
        let resolver = Resolver::builder().with_max_depth(0).build();

        let source = "package main\nconst x = 1";
        let tree = parse_go(source);
        let ctx = create_context(&tree, source.as_bytes());

        let node = find_first_node_of_kind(tree.root_node(), "int_literal").unwrap();
        let value = resolver.resolve(&node, &ctx);

        // With max_depth=0, should immediately return CycleDetected
        assert!(!value.is_resolved);
    }
}
