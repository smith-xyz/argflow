use crate::engine::{Context, Strategy, UnresolvedSource, Value};
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
        Value::unextractable(UnresolvedSource::NotImplemented)
    }
}
