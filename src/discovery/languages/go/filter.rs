use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::Language;
use crate::discovery::filter::{FilterError, ImportFileFilter};
use serde::Deserialize;

use super::config::*;

#[derive(Debug, Deserialize)]
struct MappingsFile {
    mappings: std::collections::HashMap<String, std::collections::HashMap<String, String>>,
}

pub struct GoImportFilter {
    import_patterns: Vec<String>,
}

impl GoImportFilter {
    pub fn new(preset_paths: &[PathBuf]) -> Result<Self, FilterError> {
        let import_patterns = load_import_patterns_from_presets(preset_paths, "go")?;
        Ok(Self { import_patterns })
    }

    pub fn from_bundled() -> Result<Self, FilterError> {
        let preset_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("presets")
            .join("crypto");
        Self::new(&[preset_dir])
    }
}

impl ImportFileFilter for GoImportFilter {
    fn has_matching_imports(&self, file_path: &Path) -> Result<bool, FilterError> {
        let metadata = fs::metadata(file_path).map_err(|e| {
            FilterError::FileRead(format!(
                "Failed to read metadata for {}: {}",
                file_path.display(),
                e
            ))
        })?;

        if metadata.len() > MAX_FILE_SIZE {
            return Err(FilterError::FileRead(format!(
                "File too large: {} bytes (max: {} bytes)",
                metadata.len(),
                MAX_FILE_SIZE
            )));
        }

        let content = fs::read_to_string(file_path).map_err(|e| {
            FilterError::FileRead(format!(
                "Failed to read file {}: {}",
                file_path.display(),
                e
            ))
        })?;

        Ok(self
            .import_patterns
            .iter()
            .any(|pattern| content.contains(pattern)))
    }

    fn language(&self) -> Language {
        Language::Go
    }
}

fn load_import_patterns_from_presets(
    preset_paths: &[PathBuf],
    language: &str,
) -> Result<Vec<String>, FilterError> {
    let mut all_patterns = HashSet::new();

    for preset_path in preset_paths {
        let mappings_path = preset_path.join(language).join("mappings.json");
        if mappings_path.exists() {
            let patterns = load_import_patterns_from_file(&mappings_path)?;
            all_patterns.extend(patterns);
        }
    }

    if all_patterns.is_empty() {
        return Err(FilterError::FileRead(format!(
            "No {language} mappings found in any preset. Checked: {preset_paths:?}"
        )));
    }

    Ok(all_patterns.into_iter().collect())
}

fn load_import_patterns_from_file(mappings_path: &Path) -> Result<Vec<String>, FilterError> {
    let content = fs::read_to_string(mappings_path).map_err(|e| {
        FilterError::FileRead(format!("Failed to read {}: {}", mappings_path.display(), e))
    })?;

    let file: MappingsFile = serde_json::from_str(&content).map_err(|e| {
        FilterError::FileRead(format!(
            "Failed to parse {}: {}",
            mappings_path.display(),
            e
        ))
    })?;

    let mut patterns = HashSet::new();
    for import_path in file.mappings.keys() {
        patterns.insert(format!("\"{import_path}\""));
        patterns.insert(format!("`{import_path}`"));
        patterns.insert(import_path.to_string());
    }

    Ok(patterns.into_iter().collect())
}
