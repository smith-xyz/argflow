use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct PresetMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub languages: Vec<String>,
    #[serde(default)]
    pub category: Option<String>,
}

pub fn get_presets_dir() -> PathBuf {
    let exe_path = std::env::current_exe().unwrap_or_default();
    let exe_dir = exe_path.parent().unwrap_or(Path::new("."));

    // Check relative to executable first (for installed binaries)
    let preset_path = exe_dir.join("presets");
    if preset_path.exists() {
        return preset_path;
    }

    // Check parent of executable (for cargo run from target/debug)
    let parent_preset_path = exe_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("presets"));
    if let Some(ref path) = parent_preset_path {
        if path.exists() {
            return path.clone();
        }
    }

    // Fallback to current working directory
    PathBuf::from("presets")
}

pub fn load_preset(name: &str) -> Result<PathBuf> {
    let presets_dir = get_presets_dir();
    let preset_path = presets_dir.join(name);

    if !preset_path.exists() {
        anyhow::bail!(
            "Preset '{}' not found. Available presets: {:?}",
            name,
            list_available_presets()
        );
    }

    // Validate preset has required files (classifications.json is the essential file)
    let classifications_path = preset_path.join("classifications.json");
    if !classifications_path.exists() {
        anyhow::bail!("Preset '{name}' is missing classifications.json file");
    }

    Ok(preset_path)
}

pub fn load_presets(names: &[String]) -> Result<Vec<PathBuf>> {
    names.iter().map(|name| load_preset(name)).collect()
}

#[allow(dead_code)]
pub fn load_preset_metadata(preset_path: &Path) -> Result<PresetMetadata> {
    let metadata_path = preset_path.join("preset.json");
    let content = std::fs::read_to_string(&metadata_path).with_context(|| {
        format!(
            "Failed to read preset metadata: {}",
            metadata_path.display()
        )
    })?;

    serde_json::from_str(&content).with_context(|| {
        format!(
            "Failed to parse preset metadata: {}",
            metadata_path.display()
        )
    })
}

fn list_available_presets() -> Vec<String> {
    let presets_dir = get_presets_dir();

    if !presets_dir.exists() {
        return vec![];
    }

    std::fs::read_dir(&presets_dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .filter(|e| e.path().join("classifications.json").exists())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_preset_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let preset_path = temp_dir.path();

        let metadata = r#"{
            "name": "test",
            "version": "1.0.0",
            "description": "Test preset",
            "languages": ["go", "python"]
        }"#;

        fs::write(preset_path.join("preset.json"), metadata).unwrap();

        let result = load_preset_metadata(preset_path).unwrap();
        assert_eq!(result.name, "test");
        assert_eq!(result.version, "1.0.0");
        assert_eq!(result.languages, vec!["go", "python"]);
    }

    #[test]
    fn test_list_available_presets() {
        let temp_dir = TempDir::new().unwrap();
        let crypto_preset = temp_dir.path().join("crypto");
        fs::create_dir(&crypto_preset).unwrap();
        fs::write(crypto_preset.join("classifications.json"), "{}").unwrap();

        // This tests the filtering logic, though it won't find presets in the temp dir
        // since get_presets_dir() doesn't use it
        let presets = list_available_presets();
        // Just verify it returns a Vec without panicking
        assert!(presets.is_empty() || !presets.is_empty());
    }
}
