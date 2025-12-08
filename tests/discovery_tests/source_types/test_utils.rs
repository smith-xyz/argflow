//! Shared test utilities for source type tagging tests

use crypto_extractor_core::discovery::SourceFile;

pub fn assert_all_user_code_tagged(files: &[SourceFile]) {
    for file in files {
        assert!(
            matches!(
                file.source_type,
                crypto_extractor_core::discovery::SourceType::UserCode
            ),
            "All user code files should be tagged as UserCode: {}",
            file.path.display()
        );
    }
}

pub fn assert_all_dependencies_tagged(files: &[SourceFile]) {
    for file in files {
        assert!(
            matches!(
                file.source_type,
                crypto_extractor_core::discovery::SourceType::Dependency { .. }
                    | crypto_extractor_core::discovery::SourceType::Stdlib
            ),
            "All dependency files should be tagged as Dependency or Stdlib: {}",
            file.path.display()
        );
    }
}
