/// Value representation for resolved expressions.
///
/// This matches the Value type from the Go implementation but works with
/// Tree-sitter nodes instead of language-specific AST nodes.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value {
    /// Resolved integer values
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub int_values: Vec<i64>,
    
    /// Resolved string values
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub string_values: Vec<String>,
    
    /// Is this value fully resolved?
    pub is_resolved: bool,
    
    /// Source classification for unresolved values
    #[serde(skip_serializing_if = "String::is_empty")]
    pub source: String,
    
    /// Partially resolved expression (e.g., "iterations + 10000")
    #[serde(skip_serializing_if = "String::is_empty")]
    pub expression: String,
}

impl Value {
    /// Create a resolved integer value
    pub fn resolved_int(value: i64) -> Self {
        Self {
            int_values: vec![value],
            string_values: vec![],
            is_resolved: true,
            source: String::new(),
            expression: String::new(),
        }
    }
    
    /// Create a resolved value with multiple integers
    pub fn resolved_ints(values: Vec<i64>) -> Self {
        Self {
            int_values: values,
            string_values: vec![],
            is_resolved: true,
            source: String::new(),
            expression: String::new(),
        }
    }
    
    /// Create a resolved string value
    pub fn resolved_string(value: String) -> Self {
        Self {
            int_values: vec![],
            string_values: vec![value],
            is_resolved: true,
            source: String::new(),
            expression: String::new(),
        }
    }
    
    /// Create a resolved value with multiple strings
    pub fn resolved_strings(values: Vec<String>) -> Self {
        Self {
            int_values: vec![],
            string_values: values,
            is_resolved: true,
            source: String::new(),
            expression: String::new(),
        }
    }
    
    /// Create an unextractable value with source classification
    pub fn unextractable(source: impl Into<String>) -> Self {
        Self {
            int_values: vec![],
            string_values: vec![],
            is_resolved: false,
            source: source.into(),
            expression: String::new(),
        }
    }
    
    /// Create a partially resolved expression
    pub fn partial_expression(expression: impl Into<String>) -> Self {
        Self {
            int_values: vec![],
            string_values: vec![],
            is_resolved: false,
            source: "partially_resolved".to_string(),
            expression: expression.into(),
        }
    }
    
    /// Format value for JSON output
    pub fn format_for_output(&self) -> serde_json::Value {
        if self.is_resolved {
            if !self.int_values.is_empty() {
                if self.int_values.len() == 1 {
                    return serde_json::json!(self.int_values[0]);
                }
                return serde_json::json!(&self.int_values);
            }
            if !self.string_values.is_empty() {
                if self.string_values.len() == 1 {
                    return serde_json::json!(&self.string_values[0]);
                }
                return serde_json::json!(&self.string_values);
            }
        }
        
        if !self.expression.is_empty() {
            return serde_json::json!(&self.expression);
        }
        
        // Unresolved
        serde_json::json!({
            "value": null,
            "source": if self.source.is_empty() { "not_resolved" } else { &self.source }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resolved_int() {
        let val = Value::resolved_int(10000);
        assert!(val.is_resolved);
        assert_eq!(val.int_values, vec![10000]);
    }
    
    #[test]
    fn test_resolved_string() {
        let val = Value::resolved_string("sha256".to_string());
        assert!(val.is_resolved);
        assert_eq!(val.string_values, vec!["sha256"]);
    }
    
    #[test]
    fn test_unextractable() {
        let val = Value::unextractable("function_parameter");
        assert!(!val.is_resolved);
        assert_eq!(val.source, "function_parameter");
    }
}

