/// Crypto Extractor
///
/// A multi-language cryptographic parameter extractor that uses Tree-sitter
/// for parsing and a resolution engine that works across multiple languages.
pub mod cli;
pub mod discovery;
pub mod engine;
pub mod mappings;
pub mod scanner;

pub use engine::{Context, Resolver, Value};
pub use scanner::{CryptoCall, CryptoMatcher, PatternMatcher, ScanResult, Scanner};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
