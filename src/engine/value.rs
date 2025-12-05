use serde::{Deserialize, Serialize};

use super::operators::{BinaryOp, UnaryOp};
use super::sources::{self, UnresolvedSource};

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
            source: UnresolvedSource::PartiallyResolved.to_string(),
            expression: expression.into(),
        }
    }

    pub fn display(&self) -> String {
        if self.is_resolved {
            if !self.int_values.is_empty() {
                return Self::format_int_list(&self.int_values);
            }
            if !self.string_values.is_empty() {
                return Self::format_string_list(&self.string_values);
            }
        }

        if !self.expression.is_empty() {
            return self.expression.clone();
        }

        self.format_unresolved_tag()
    }

    fn format_int_list(values: &[i64]) -> String {
        match values {
            [single] => single.to_string(),
            multiple => format!(
                "[{}]",
                multiple
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }

    fn format_string_list(values: &[String]) -> String {
        match values {
            [single] => format!("\"{single}\""),
            multiple => format!(
                "[{}]",
                multiple
                    .iter()
                    .map(|v| format!("\"{v}\""))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }

    fn format_unresolved_tag(&self) -> String {
        let tag = if self.source.is_empty() {
            sources::UNRESOLVED
        } else {
            &self.source
        };
        format!("<{tag}>")
    }

    pub fn as_int(&self) -> Option<i64> {
        if self.is_resolved && self.int_values.len() == 1 {
            Some(self.int_values[0])
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        if self.is_resolved && self.string_values.len() == 1 {
            Some(&self.string_values[0])
        } else {
            None
        }
    }

    pub fn binary_op(left: &Value, op: &str, right: &Value) -> Value {
        if let (Some(l), Some(r)) = (left.as_int(), right.as_int()) {
            if let Some(binary_op) = BinaryOp::parse(op) {
                if let Some(result) = binary_op.evaluate(l, r) {
                    return Value::resolved_int(result);
                }
            }
        }

        Value::partial_expression(format!("{} {} {}", left.display(), op, right.display()))
    }

    pub fn unary_op(op: &str, operand: &Value) -> Value {
        if let Some(v) = operand.as_int() {
            if let Some(unary_op) = UnaryOp::parse(op) {
                if let Some(result) = unary_op.evaluate(v) {
                    return Value::resolved_int(result);
                }
            }
        }

        Value::partial_expression(format!("{}{}", op, operand.display()))
    }

    pub fn merge(values: Vec<Value>) -> Value {
        let mut all_ints: Vec<i64> = Vec::new();
        let mut all_strings: Vec<String> = Vec::new();
        let mut all_resolved = true;

        for val in values {
            if val.is_resolved {
                all_ints.extend(val.int_values);
                all_strings.extend(val.string_values);
            } else {
                all_resolved = false;
            }
        }

        if !all_resolved {
            return Value::unextractable(UnresolvedSource::MixedResolution);
        }

        if !all_ints.is_empty() && all_strings.is_empty() {
            all_ints.sort();
            all_ints.dedup();
            return Value::resolved_ints(all_ints);
        }

        if !all_strings.is_empty() && all_ints.is_empty() {
            all_strings.sort();
            all_strings.dedup();
            return Value::resolved_strings(all_strings);
        }

        Value::unextractable(UnresolvedSource::MixedTypes)
    }

    pub fn format_for_output(&self) -> serde_json::Value {
        if self.is_resolved {
            if !self.int_values.is_empty() {
                return Self::format_int_json(&self.int_values);
            }
            if !self.string_values.is_empty() {
                return Self::format_string_json(&self.string_values);
            }
        }

        if !self.expression.is_empty() {
            return serde_json::json!(&self.expression);
        }

        self.format_unresolved_json()
    }

    fn format_int_json(values: &[i64]) -> serde_json::Value {
        match values {
            [single] => serde_json::json!(single),
            multiple => serde_json::json!(multiple),
        }
    }

    fn format_string_json(values: &[String]) -> serde_json::Value {
        match values {
            [single] => serde_json::json!(single),
            multiple => serde_json::json!(multiple),
        }
    }

    fn format_unresolved_json(&self) -> serde_json::Value {
        let source = if self.source.is_empty() {
            sources::NOT_RESOLVED
        } else {
            &self.source
        };
        serde_json::json!({
            "value": null,
            "source": source
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

    #[test]
    fn test_display_resolved_int() {
        assert_eq!(Value::resolved_int(10000).display(), "10000");
        assert_eq!(Value::resolved_ints(vec![16, 32]).display(), "[16, 32]");
    }

    #[test]
    fn test_display_resolved_string() {
        assert_eq!(
            Value::resolved_string("sha256".to_string()).display(),
            "\"sha256\""
        );
        assert_eq!(
            Value::resolved_strings(vec!["a".to_string(), "b".to_string()]).display(),
            "[\"a\", \"b\"]"
        );
    }

    #[test]
    fn test_display_partial_expression() {
        let val = Value::partial_expression("BASE + 10000");
        assert_eq!(val.display(), "BASE + 10000");
    }

    #[test]
    fn test_display_unextractable() {
        let val = Value::unextractable("function_parameter");
        assert_eq!(val.display(), "<function_parameter>");
    }

    #[test]
    fn test_as_int() {
        assert_eq!(Value::resolved_int(42).as_int(), Some(42));
        assert_eq!(Value::resolved_ints(vec![1, 2]).as_int(), None);
        assert_eq!(Value::resolved_string("x".to_string()).as_int(), None);
    }

    #[test]
    fn test_as_string() {
        assert_eq!(
            Value::resolved_string("test".to_string()).as_string(),
            Some("test")
        );
        assert_eq!(Value::resolved_int(42).as_string(), None);
    }

    #[test]
    fn test_binary_op_addition() {
        let left = Value::resolved_int(100000);
        let right = Value::resolved_int(10000);
        let result = Value::binary_op(&left, "+", &right);

        assert!(result.is_resolved);
        assert_eq!(result.as_int(), Some(110000));
    }

    #[test]
    fn test_binary_op_subtraction() {
        let left = Value::resolved_int(100);
        let right = Value::resolved_int(30);
        let result = Value::binary_op(&left, "-", &right);

        assert_eq!(result.as_int(), Some(70));
    }

    #[test]
    fn test_binary_op_multiplication() {
        let left = Value::resolved_int(32);
        let right = Value::resolved_int(8);
        let result = Value::binary_op(&left, "*", &right);

        assert_eq!(result.as_int(), Some(256));
    }

    #[test]
    fn test_binary_op_division() {
        let left = Value::resolved_int(100);
        let right = Value::resolved_int(4);
        let result = Value::binary_op(&left, "/", &right);

        assert_eq!(result.as_int(), Some(25));
    }

    #[test]
    fn test_binary_op_division_by_zero() {
        let left = Value::resolved_int(100);
        let right = Value::resolved_int(0);
        let result = Value::binary_op(&left, "/", &right);

        assert!(!result.is_resolved);
        assert_eq!(result.expression, "100 / 0");
    }

    #[test]
    fn test_binary_op_bitwise() {
        let left = Value::resolved_int(1);
        let right = Value::resolved_int(4);
        let result = Value::binary_op(&left, "<<", &right);

        assert_eq!(result.as_int(), Some(16));
    }

    #[test]
    fn test_binary_op_partial() {
        let left = Value::resolved_int(100000);
        let right = Value::unextractable("config_value");
        let result = Value::binary_op(&left, "+", &right);

        assert!(!result.is_resolved);
        assert_eq!(result.expression, "100000 + <config_value>");
    }

    #[test]
    fn test_unary_op_negation() {
        let operand = Value::resolved_int(42);
        let result = Value::unary_op("-", &operand);

        assert_eq!(result.as_int(), Some(-42));
    }

    #[test]
    fn test_unary_op_bitwise_not() {
        let operand = Value::resolved_int(0);
        let result = Value::unary_op("^", &operand);

        assert_eq!(result.as_int(), Some(-1));
    }

    #[test]
    fn test_unary_op_partial() {
        let operand = Value::unextractable("unknown");
        let result = Value::unary_op("-", &operand);

        assert!(!result.is_resolved);
        assert_eq!(result.expression, "-<unknown>");
    }

    #[test]
    fn test_merge_integers() {
        let values = vec![
            Value::resolved_int(16),
            Value::resolved_int(32),
            Value::resolved_int(16),
        ];
        let result = Value::merge(values);

        assert!(result.is_resolved);
        assert_eq!(result.int_values, vec![16, 32]);
    }

    #[test]
    fn test_merge_strings() {
        let values = vec![
            Value::resolved_string("a".to_string()),
            Value::resolved_string("b".to_string()),
        ];
        let result = Value::merge(values);

        assert!(result.is_resolved);
        assert_eq!(result.string_values, vec!["a", "b"]);
    }

    #[test]
    fn test_merge_with_unresolved() {
        let values = vec![Value::resolved_int(16), Value::unextractable("runtime")];
        let result = Value::merge(values);

        assert!(!result.is_resolved);
        assert_eq!(result.source, "mixed_resolution");
    }

    #[test]
    fn test_partial_expression() {
        let val = Value::partial_expression("iterations + 10000");
        assert!(!val.is_resolved);
        assert_eq!(val.source, "partially_resolved");
        assert_eq!(val.expression, "iterations + 10000");
    }

    #[test]
    fn test_format_for_output_resolved() {
        let val = Value::resolved_int(10000);
        assert_eq!(val.format_for_output(), serde_json::json!(10000));
    }

    #[test]
    fn test_format_for_output_partial() {
        let val = Value::partial_expression("BASE + 10000");
        assert_eq!(val.format_for_output(), serde_json::json!("BASE + 10000"));
    }

    #[test]
    fn test_format_for_output_unresolved() {
        let val = Value::unextractable("function_parameter");
        let output = val.format_for_output();
        assert_eq!(output["source"], "function_parameter");
        assert!(output["value"].is_null());
    }
}
