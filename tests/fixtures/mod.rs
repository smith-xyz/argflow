use std::path::PathBuf;

pub fn get_test_fixture_path(language: &str, fixture_name: Option<&str>) -> PathBuf {
    if let Some(fixture_name) = fixture_name {
        return PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(language)
            .join(fixture_name);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(language)
}
