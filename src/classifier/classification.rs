use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Classification {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub algorithm_family: Option<String>,

    pub finding_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_type: Option<String>,

    pub operation: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub crypto_functions: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub primitive: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_size: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_size: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce_size: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_size: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_set_identifier: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub curve: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub classical_security_level: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub nist_quantum_security_level: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_source: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_format: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<String>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Classification {
    pub fn unclassified() -> Self {
        Self {
            finding_type: "unknown".to_string(),
            operation: "unknown".to_string(),
            ..Default::default()
        }
    }

    pub fn is_unclassified(&self) -> bool {
        self.finding_type == "unknown" && self.algorithm.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_pbkdf2() {
        let json = r#"{
            "algorithm": "PBKDF2",
            "algorithmFamily": "PBKDF2",
            "findingType": "kdf",
            "assetType": "algorithm",
            "operation": "keyderive",
            "cryptoFunctions": ["keyderive"],
            "primitive": "kdf",
            "materialSource": "derived"
        }"#;

        let classification: Classification = serde_json::from_str(json).unwrap();
        assert_eq!(classification.algorithm, Some("PBKDF2".to_string()));
        assert_eq!(classification.finding_type, "kdf");
        assert_eq!(classification.operation, "keyderive");
        assert_eq!(classification.primitive, Some("kdf".to_string()));
    }

    #[test]
    fn test_deserialize_aes_gcm() {
        let json = r#"{
            "algorithm": "AES-GCM",
            "algorithmFamily": "AES",
            "findingType": "aead",
            "operation": "encrypt",
            "primitive": "aead",
            "mode": "GCM",
            "nonceSize": 12,
            "tagSize": 16
        }"#;

        let classification: Classification = serde_json::from_str(json).unwrap();
        assert_eq!(classification.algorithm, Some("AES-GCM".to_string()));
        assert_eq!(classification.mode, Some("GCM".to_string()));
        assert_eq!(classification.nonce_size, Some(12));
        assert_eq!(classification.tag_size, Some(16));
    }

    #[test]
    fn test_unclassified() {
        let classification = Classification::unclassified();
        assert!(classification.is_unclassified());
        assert_eq!(classification.finding_type, "unknown");
    }

    #[test]
    fn test_extra_fields() {
        let json = r#"{
            "findingType": "custom",
            "operation": "custom",
            "customField": "customValue",
            "anotherField": 42
        }"#;

        let classification: Classification = serde_json::from_str(json).unwrap();
        assert_eq!(classification.finding_type, "custom");
        assert!(classification.extra.contains_key("customField"));
    }
}
