//! Shared test utilities for cache tests

use argflow::discovery::SourceFile;
use std::collections::HashSet;

pub fn assert_cache_consistency(files1: &[SourceFile], files2: &[SourceFile]) {
    assert_eq!(
        files1.len(),
        files2.len(),
        "Cached and non-cached results should have same file count"
    );

    let paths1: HashSet<_> = files1.iter().map(|f| &f.path).collect();
    let paths2: HashSet<_> = files2.iter().map(|f| &f.path).collect();

    assert_eq!(
        paths1, paths2,
        "Cached and non-cached results should contain same files"
    );
}
