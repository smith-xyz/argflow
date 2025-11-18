/// Resolution engine for extracting cryptographic parameters.
pub mod context;
pub mod strategies;
pub mod value;

pub use context::Context;
pub use value::Value;

use strategies::LiteralStrategy;
use tree_sitter::Node;

/// Main resolver that orchestrates strategy execution
pub struct Resolver {
    strategies: Vec<Box<dyn Strategy>>,
}

/// Trait that all resolution strategies must implement
pub trait Strategy {
    fn can_handle<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> bool;
    fn resolve<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value;
}

impl Resolver {
    /// Create a new resolver with default strategies
    pub fn new() -> Self {
        Self {
            strategies: vec![Box::new(LiteralStrategy::new())],
        }
    }

    /// Resolve an expression node to a value
    pub fn resolve<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        if let Some(cached) = ctx.get_cached_value(node) {
            return cached;
        }

        if ctx.has_visited(node) {
            return Value::unextractable("cycle_detected");
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
            resolved_value.unwrap_or_else(|| Value::unextractable("not_implemented"))
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
