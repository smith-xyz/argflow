//! Shared test utilities for import filtering tests

use argflow::discovery::filter::ImportFileFilter;
use argflow::discovery::SourceFile;

pub fn filter_matching_files<F: ImportFileFilter>(
    files: Vec<SourceFile>,
    filter: &F,
) -> Vec<SourceFile> {
    files
        .into_iter()
        .filter_map(|file| {
            filter
                .has_matching_imports(&file.path)
                .ok()
                .and_then(|has_match| has_match.then_some(file))
        })
        .collect()
}
