use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use crate::cli::Language;
use crate::discovery::filter::{CryptoFileFilter, FilterError};
use serde::Deserialize;

use super::config::*;

#[derive(Debug, Deserialize)]
struct MappingsFile {
    mappings: std::collections::HashMap<String, std::collections::HashMap<String, String>>,
}

static CRYPTO_IMPORT_PATTERNS: OnceLock<Result<Vec<String>, String>> = OnceLock::new();

fn get_crypto_import_patterns() -> Result<&'static Vec<String>, &'static String> {
    CRYPTO_IMPORT_PATTERNS
        .get_or_init(load_import_patterns_from_mappings)
        .as_ref()
}

fn load_import_patterns_from_mappings() -> Result<Vec<String>, String> {
    let rules_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("classifier-rules");
    let mappings_path = rules_dir.join("python").join("mappings.json");

    if !mappings_path.exists() {
        return Err(format!(
            "mappings.json not found at {}",
            mappings_path.display()
        ));
    }

    let content = fs::read_to_string(&mappings_path)
        .map_err(|e| format!("Failed to read mappings.json: {e}"))?;
    let file: MappingsFile = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse mappings.json: {e}"))?;

    let mut patterns = HashSet::new();
    for import_path in file.mappings.keys() {
        patterns.insert(format!("import {import_path}"));
        patterns.insert(format!("from {import_path}"));
        patterns.insert(format!("import {import_path} as"));
        patterns.insert(format!("from {import_path} import"));
    }

    Ok(patterns.into_iter().collect())
}

pub struct PythonCryptoFilter;

impl CryptoFileFilter for PythonCryptoFilter {
    fn has_crypto_usage(&self, file_path: &Path) -> Result<bool, FilterError> {
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

        self.check_crypto_imports(&content)
    }

    fn language(&self) -> Language {
        Language::Python
    }
}

impl PythonCryptoFilter {
    fn check_crypto_imports(&self, content: &str) -> Result<bool, FilterError> {
        let patterns = get_crypto_import_patterns().map_err(|e| {
            FilterError::FileRead(format!(
                "Failed to load crypto import patterns from classifier-rules: {e}. \
                 Ensure classifier-rules/python/mappings.json exists and is valid."
            ))
        })?;

        Ok(patterns.iter().any(|pattern| content.contains(pattern)))
    }
}
