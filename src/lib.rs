/// Argflow
///
/// Argument flow analyzer - traces where function arguments come from across
/// multi-language codebases using Tree-sitter for parsing and a resolution
/// engine that works across multiple languages.
pub mod classifier;
pub mod cli;
pub mod discovery;
pub mod engine;
pub mod error;
pub mod logging;
pub mod mappings;
pub mod output;
pub mod presets;
pub mod query;
pub mod scanner;
pub mod utils;

pub use classifier::{
    classify_call, Classification, ClassifiedCall, Classifier, ClassifierError, RulesClassifier,
};
pub use engine::{Context, Resolver, Value};
pub use error::{Error, IoError, ParserError, QueryError};
pub use logging::Verbosity;
pub use output::{ConfigFinding, Finding, JsonOutput, OutputFormatter};
pub use presets::{load_preset, load_presets, PresetMetadata};
pub use scanner::{CallMatcher, ImportMap, PatternMatcher, ScanResult, Scanner};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
