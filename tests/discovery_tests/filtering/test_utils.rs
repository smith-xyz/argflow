//! Shared test utilities for crypto filtering tests

use crypto_extractor_core::discovery::filter::CryptoFileFilter;
use crypto_extractor_core::discovery::SourceFile;

pub fn filter_crypto_files<F: CryptoFileFilter>(
    files: Vec<SourceFile>,
    filter: &F,
) -> Vec<SourceFile> {
    files
        .into_iter()
        .filter_map(|file| {
            filter
                .has_crypto_usage(&file.path)
                .ok()
                .and_then(|has_crypto| has_crypto.then_some(file))
        })
        .collect()
}
