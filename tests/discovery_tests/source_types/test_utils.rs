//! Shared test utilities for source type tagging tests

use argflow::discovery::SourceFile;

pub fn assert_all_user_code_tagged(files: &[SourceFile]) {
    for file in files {
        assert!(
            matches!(file.source_type, argflow::discovery::SourceType::UserCode),
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
                argflow::discovery::SourceType::Dependency { .. }
                    | argflow::discovery::SourceType::Stdlib
            ),
            "All dependency files should be tagged as Dependency or Stdlib: {}",
            file.path.display()
        );
    }
}
