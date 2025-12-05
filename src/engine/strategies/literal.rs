use crate::engine::{Context, Strategy, Value};
/// Literal Strategy - Extract literal values from any language.
///
/// Stub implementation - to be fully implemented later.
use tree_sitter::Node;

pub struct LiteralStrategy;

impl Default for LiteralStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl LiteralStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Strategy for LiteralStrategy {
    fn can_handle<'a>(&self, _node: &Node<'a>, _ctx: &Context<'a>) -> bool {
        false
    }

    fn resolve<'a>(&self, _node: &Node<'a>, _ctx: &Context<'a>) -> Value {
        Value::unextractable("not_implemented")
    }
}
