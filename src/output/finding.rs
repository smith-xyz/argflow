use serde::Serialize;
use std::collections::HashMap;

use crate::classifier::RulesClassifier;
use crate::engine::Value;
use crate::scanner::{CryptoCall, CryptoConfig};

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub function: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_path: Option<String>,
    pub full_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finding_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primitive: Option<String>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub raw_text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigFinding {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub struct_type: String,
    pub full_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_path: Option<String>,
    pub fields: Vec<ConfigFieldValue>,
    pub raw_text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigFieldValue {
    pub field_name: String,
    pub value: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classification_key: Option<String>,
}

impl Finding {
    pub fn from_crypto_call(call: &CryptoCall, classifier: &RulesClassifier) -> Self {
        let classification = crate::classifier::classify_call(call, classifier);

        let parameters = call
            .arguments
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let name = format!("arg{i}");
                let param_value = value_to_json(v);
                (name, param_value)
            })
            .collect();

        Finding {
            file: call.file_path.clone(),
            line: call.line,
            column: call.column,
            function: call.function_name.clone(),
            package: call.package.clone(),
            import_path: call.import_path.clone(),
            full_name: call.full_name(),
            algorithm: classification.algorithm,
            finding_type: if classification.finding_type.is_empty() {
                None
            } else {
                Some(classification.finding_type)
            },
            operation: if classification.operation.is_empty() {
                None
            } else {
                Some(classification.operation)
            },
            primitive: classification.primitive,
            parameters,
            raw_text: call.raw_text.clone(),
        }
    }
}

impl ConfigFinding {
    pub fn from_crypto_config(config: &CryptoConfig) -> Self {
        let fields = config
            .fields
            .iter()
            .map(|f| ConfigFieldValue {
                field_name: f.field_name.clone(),
                value: value_to_json(&f.value),
                classification_key: f.classification_key.clone(),
            })
            .collect();

        ConfigFinding {
            file: config.file_path.clone(),
            line: config.line,
            column: config.column,
            struct_type: config.struct_type.clone(),
            full_type: config.full_type(),
            package: config.package.clone(),
            import_path: config.import_path.clone(),
            fields,
            raw_text: config.raw_text.clone(),
        }
    }
}

fn value_to_json(value: &Value) -> serde_json::Value {
    // Resolved: return direct value
    if !value.int_values.is_empty() {
        if value.int_values.len() == 1 {
            serde_json::Value::Number(value.int_values[0].into())
        } else {
            serde_json::Value::Array(
                value
                    .int_values
                    .iter()
                    .map(|&v| serde_json::Value::Number(v.into()))
                    .collect(),
            )
        }
    } else if !value.string_values.is_empty() {
        if value.string_values.len() == 1 {
            serde_json::Value::String(value.string_values[0].clone())
        } else {
            serde_json::Value::Array(
                value
                    .string_values
                    .iter()
                    .map(|s| serde_json::Value::String(s.clone()))
                    .collect(),
            )
        }
    // Partial: return value with source
    } else if !value.expression.is_empty() {
        serde_json::json!({
            "value": value.expression,
            "source": "partial_expression"
        })
    // Unresolved: return source only
    } else if !value.source.is_empty() {
        serde_json::json!({
            "source": value.source,
            "value": null
        })
    } else {
        serde_json::json!({
            "source": "unknown",
            "value": null
        })
    }
}
