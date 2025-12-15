//! Shared test utilities for dependency discovery tests

use crypto_extractor_core::discovery::SourceFile;

pub fn get_dependency_files(files: &[SourceFile]) -> Vec<&SourceFile> {
    files
        .iter()
        .filter(|f| {
            matches!(
                f.source_type,
                crypto_extractor_core::discovery::SourceType::Dependency { .. }
            )
        })
        .collect()
}

pub fn get_stdlib_files(files: &[SourceFile]) -> Vec<&SourceFile> {
    files
        .iter()
        .filter(|f| {
            matches!(
                f.source_type,
                crypto_extractor_core::discovery::SourceType::Stdlib
            )
        })
        .collect()
}

pub fn get_user_code_files(files: &[SourceFile]) -> Vec<&SourceFile> {
    files
        .iter()
        .filter(|f| {
            matches!(
                f.source_type,
                crypto_extractor_core::discovery::SourceType::UserCode
            )
        })
        .collect()
}
