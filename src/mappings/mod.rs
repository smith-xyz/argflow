/// Language mappings and configuration
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Deserialize)]
pub struct LanguageMapping {
    pub language: String,
    pub tree_sitter_package: String,
    pub node_types: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub field_names: HashMap<String, HashMap<String, String>>,
}

impl LanguageMapping {
    /// Convert node_types Vec<String> to HashSet<String> for fast lookup
    pub fn to_node_type_sets(&self) -> HashMap<String, HashSet<String>> {
        self.node_types
            .iter()
            .map(|(k, v)| (k.clone(), v.iter().cloned().collect()))
            .collect()
    }
}

pub fn load_mapping(language: &str) -> Result<LanguageMapping, anyhow::Error> {
    let mapping_path = format!("mappings/{language}.yaml");
    let contents = std::fs::read_to_string(&mapping_path)?;
    let mapping: LanguageMapping = serde_yaml::from_str(&contents)?;
    Ok(mapping)
}
