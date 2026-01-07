//! Shared test utilities for integration tests

use argflow::discovery::SourceFile;

pub fn combine_user_and_dependencies(
    user_files: Vec<SourceFile>,
    dep_files: Vec<SourceFile>,
) -> Vec<SourceFile> {
    user_files.into_iter().chain(dep_files).collect()
}
