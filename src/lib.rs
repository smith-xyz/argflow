/// Crypto Extractor
///
/// A multi-language cryptographic parameter extractor that uses Tree-sitter
/// for parsing and a resolution engine that works across multiple languages.
pub mod cli;
pub mod discovery;
pub mod engine;
pub mod mappings;

pub use engine::{Context, Resolver, Value};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
