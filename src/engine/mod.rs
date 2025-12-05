pub mod context;
pub mod file_cache;
pub mod operators;
pub mod scope;
pub mod sources;
pub mod strategies;
pub mod value;

pub use context::Context;
pub use file_cache::{CachedFileEntry, FileCache, FunctionInfo};
pub use operators::{BinaryOp, UnaryOp};
pub use scope::{Scope, ScopeEntry};
pub use sources::UnresolvedSource;
pub use value::Value;

use strategies::LiteralStrategy;
use tree_sitter::Node;

pub trait Strategy {
    fn can_handle<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> bool;
    fn resolve<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value;
}

pub struct Resolver {
    strategies: Vec<Box<dyn Strategy>>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            strategies: vec![Box::new(LiteralStrategy::new())],
        }
    }

    pub fn resolve<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        if let Some(cached) = ctx.get_cached_value(node) {
            return cached;
        }

        if ctx.has_visited(node) {
            return Value::unextractable(UnresolvedSource::CycleDetected);
        }
        ctx.mark_visited(node);

        let result = {
            let mut resolved_value = None;
            for strategy in &self.strategies {
                if strategy.can_handle(node, ctx) {
                    let value = strategy.resolve(node, ctx);
                    resolved_value = Some(value);
                    break;
                }
            }
            resolved_value.unwrap_or_else(|| Value::unextractable(UnresolvedSource::NotImplemented))
        };

        ctx.cache_value(node, result.clone());
        ctx.unmark_visited(node);
        result
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}
