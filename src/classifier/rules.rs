use super::Classification;
use crate::error::ClassifierError;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, trace};

pub trait Classifier: Send + Sync {
    fn lookup(&self, import_path: &str, function: &str) -> Classification;

    fn lookup_with_fallback(
        &self,
        import_path: Option<&str>,
        package: &str,
        function: &str,
    ) -> Classification {
        // Try full import path first
        if let Some(path) = import_path {
            let result = self.lookup(path, function);
            if !result.is_unclassified() {
                return result;
            }
        }

        // Fallback to package name
        let result = self.lookup(package, function);
        if !result.is_unclassified() {
            return result;
        }

        Classification::unclassified()
    }
}

#[derive(Debug, Deserialize)]
struct ClassificationsFile {
    #[allow(dead_code)]
    version: String,
    classifications: HashMap<String, Classification>,
}

#[derive(Debug, Deserialize)]
struct MappingsFile {
    #[allow(dead_code)]
    version: String,
    #[allow(dead_code)]
    language: String,
    mappings: HashMap<String, HashMap<String, String>>,
    #[serde(default)]
    struct_fields: HashMap<String, HashMap<String, String>>,
    #[serde(default)]
    constants: HashMap<String, HashMap<String, ConstantValue>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConstantValue {
    pub value: i64,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}

type ImportMap = HashMap<String, HashMap<String, String>>;
type StructFieldMap = HashMap<String, HashMap<String, String>>;
type ConstantsMap = HashMap<String, HashMap<String, ConstantValue>>;

pub struct RulesClassifier {
    classifications: HashMap<String, Classification>,
    mappings: ImportMap,
    struct_fields: StructFieldMap,
    constants: ConstantsMap,
}

impl RulesClassifier {
    pub fn new() -> Self {
        Self {
            classifications: HashMap::new(),
            mappings: HashMap::new(),
            struct_fields: HashMap::new(),
            constants: HashMap::new(),
        }
    }

    pub fn load_classifications<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ClassifierError> {
        let path = path.as_ref();
        trace!(path = %path.display(), "loading classifications");

        let content = fs::read_to_string(path)
            .map_err(|e| ClassifierError::rules_file_read_error(path, e.to_string()))?;

        let file: ClassificationsFile = serde_json::from_str(&content)
            .map_err(|e| ClassifierError::rules_parse_error(path, e.to_string()))?;

        self.classifications = file.classifications;
        debug!(count = self.classifications.len(), "loaded classifications");
        Ok(())
    }

    pub fn load_mappings<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ClassifierError> {
        let path = path.as_ref();
        trace!(path = %path.display(), "loading mappings");

        let content = fs::read_to_string(path)
            .map_err(|e| ClassifierError::rules_file_read_error(path, e.to_string()))?;

        let file: MappingsFile = serde_json::from_str(&content)
            .map_err(|e| ClassifierError::rules_parse_error(path, e.to_string()))?;

        // Parse nested mappings format: { "import_path": { "function": "key" } }
        let mut count = 0;
        for (import_path, functions) in file.mappings {
            let import_lower = import_path.to_lowercase();
            let entry = self.mappings.entry(import_lower).or_default();
            for (func, key) in functions {
                entry.insert(func.to_lowercase(), key);
                count += 1;
            }
        }

        // Load struct field mappings
        for (struct_type, fields) in file.struct_fields {
            let type_lower = struct_type.to_lowercase();
            let entry = self.struct_fields.entry(type_lower).or_default();
            for (field, key) in fields {
                entry.insert(field.to_lowercase(), key);
            }
        }

        // Load constant values
        for (package, constants) in file.constants {
            let pkg_lower = package.to_lowercase();
            let entry = self.constants.entry(pkg_lower).or_default();
            for (name, value) in constants {
                entry.insert(name.to_lowercase(), value);
            }
        }

        debug!(count, "loaded mappings");
        Ok(())
    }

    pub fn load_user_rules<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ClassifierError> {
        let path = path.as_ref();
        debug!(path = %path.display(), "loading user rules");

        let content = fs::read_to_string(path)
            .map_err(|e| ClassifierError::rules_file_read_error(path, e.to_string()))?;

        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match extension {
            "json" => self.parse_user_rules_json(&content),
            "yaml" | "yml" => self.parse_user_rules_yaml(&content),
            _ => Err(ClassifierError::unsupported_format(extension)),
        }
    }

    fn parse_user_rules_json(&mut self, content: &str) -> Result<(), ClassifierError> {
        let rules: UserRulesFile =
            serde_json::from_str(content).map_err(|e| ClassifierError::RulesParseError {
                path: "user_rules".into(),
                message: e.to_string(),
            })?;
        self.merge_user_rules(rules);
        Ok(())
    }

    fn parse_user_rules_yaml(&mut self, content: &str) -> Result<(), ClassifierError> {
        let rules: UserRulesFile =
            serde_yaml::from_str(content).map_err(|e| ClassifierError::RulesParseError {
                path: "user_rules".into(),
                message: e.to_string(),
            })?;
        self.merge_user_rules(rules);
        Ok(())
    }

    fn merge_user_rules(&mut self, rules: UserRulesFile) {
        if let Some(classifications) = rules.classifications {
            for (key, classification) in classifications {
                self.classifications.insert(key, classification);
            }
        }
        if let Some(mappings) = rules.mappings {
            for (import_path, functions) in mappings {
                let import_lower = import_path.to_lowercase();
                let entry = self.mappings.entry(import_lower).or_default();
                for (func, key) in functions {
                    entry.insert(func.to_lowercase(), key);
                }
            }
        }
    }

    pub fn from_bundled() -> Result<Self, ClassifierError> {
        debug!("loading bundled classifier rules");
        let mut classifier = Self::new();

        let rules_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("classifier-rules");
        trace!(path = %rules_dir.display(), "rules directory");

        classifier.load_classifications(rules_dir.join("classifications.json"))?;

        for lang in &["go", "python", "rust", "javascript"] {
            let mappings_path = rules_dir.join(lang).join("mappings.json");
            if mappings_path.exists() {
                classifier.load_mappings(&mappings_path)?;
            }
        }

        debug!(
            classifications = classifier.classification_count(),
            mappings = classifier.mapping_count(),
            "bundled rules loaded"
        );
        Ok(classifier)
    }

    pub fn from_bundled_for_language(language: &str) -> Result<Self, ClassifierError> {
        debug!(language, "loading bundled classifier rules for language");
        let mut classifier = Self::new();

        let rules_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("classifier-rules");

        classifier.load_classifications(rules_dir.join("classifications.json"))?;

        let mappings_path = rules_dir.join(language).join("mappings.json");
        if mappings_path.exists() {
            classifier.load_mappings(mappings_path)?;
        }

        Ok(classifier)
    }

    pub fn classification_count(&self) -> usize {
        self.classifications.len()
    }

    pub fn mapping_count(&self) -> usize {
        self.mappings.values().map(|m| m.len()).sum()
    }

    pub fn get_mappings(&self) -> &HashMap<String, HashMap<String, String>> {
        &self.mappings
    }

    pub fn get_struct_fields(&self) -> &StructFieldMap {
        &self.struct_fields
    }

    pub fn get_constants(&self) -> &ConstantsMap {
        &self.constants
    }

    pub fn lookup_struct_field(&self, struct_type: &str, field_name: &str) -> Option<&str> {
        let type_lower = struct_type.to_lowercase();
        let field_lower = field_name.to_lowercase();
        self.struct_fields
            .get(&type_lower)
            .and_then(|fields| fields.get(&field_lower))
            .map(|s| s.as_str())
    }

    pub fn is_crypto_struct(&self, struct_type: &str) -> bool {
        let type_lower = struct_type.to_lowercase();
        self.struct_fields.contains_key(&type_lower)
    }

    pub fn lookup_constant(&self, package: &str, constant_name: &str) -> Option<&ConstantValue> {
        let pkg_lower = package.to_lowercase();
        let const_lower = constant_name.to_lowercase();
        self.constants
            .get(&pkg_lower)
            .and_then(|constants| constants.get(&const_lower))
    }
}

impl Default for RulesClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl Classifier for RulesClassifier {
    fn lookup(&self, import_path: &str, function: &str) -> Classification {
        let import_lower = import_path.to_lowercase();
        let func_lower = function.to_lowercase();

        if let Some(functions) = self.mappings.get(&import_lower) {
            if let Some(key) = functions.get(&func_lower) {
                if let Some(classification) = self.classifications.get(key) {
                    return classification.clone();
                }
            }
        }

        Classification::unclassified()
    }
}

#[derive(Debug, Deserialize)]
struct UserRulesFile {
    classifications: Option<HashMap<String, Classification>>,
    mappings: Option<HashMap<String, HashMap<String, String>>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rules_classifier_new() {
        let classifier = RulesClassifier::new();
        assert_eq!(classifier.classification_count(), 0);
        assert_eq!(classifier.mapping_count(), 0);
    }

    #[test]
    fn test_lookup_unclassified() {
        let classifier = RulesClassifier::new();
        let result = classifier.lookup("unknown", "function");
        assert!(result.is_unclassified());
    }

    #[test]
    fn test_load_bundled_classifications() {
        let classifier = RulesClassifier::from_bundled();
        assert!(classifier.is_ok());
        let classifier = classifier.unwrap();
        assert!(classifier.classification_count() > 0);
        assert!(classifier.mapping_count() > 0);
    }

    #[test]
    fn test_lookup_go_pbkdf2() {
        let classifier = RulesClassifier::from_bundled().unwrap();
        let result = classifier.lookup("golang.org/x/crypto/pbkdf2", "Key");
        assert!(!result.is_unclassified());
        assert_eq!(result.algorithm, Some("PBKDF2".to_string()));
        assert_eq!(result.finding_type, "kdf");
    }

    #[test]
    fn test_lookup_go_sha256() {
        let classifier = RulesClassifier::from_bundled().unwrap();
        let result = classifier.lookup("crypto/sha256", "New");
        assert!(!result.is_unclassified());
        assert_eq!(result.algorithm, Some("SHA-256".to_string()));
        assert_eq!(result.finding_type, "hash");
    }

    #[test]
    fn test_lookup_with_fallback() {
        let classifier = RulesClassifier::from_bundled().unwrap();

        let result =
            classifier.lookup_with_fallback(Some("golang.org/x/crypto/pbkdf2"), "pbkdf2", "Key");
        assert!(!result.is_unclassified());
        assert_eq!(result.algorithm, Some("PBKDF2".to_string()));
    }

    #[test]
    fn test_lookup_go_aes_gcm() {
        let classifier = RulesClassifier::from_bundled().unwrap();
        let result = classifier.lookup("crypto/cipher", "NewGCM");
        assert!(!result.is_unclassified());
        assert_eq!(result.algorithm, Some("AES-GCM".to_string()));
        assert_eq!(result.mode, Some("GCM".to_string()));
    }

    #[test]
    fn test_lookup_python_hashlib() {
        let classifier = RulesClassifier::from_bundled().unwrap();
        let result = classifier.lookup("hashlib", "pbkdf2_hmac");
        assert!(!result.is_unclassified());
        assert_eq!(result.algorithm, Some("PBKDF2".to_string()));
        assert_eq!(result.finding_type, "kdf");
    }

    #[test]
    fn test_lookup_rust_ring() {
        let classifier = RulesClassifier::from_bundled().unwrap();
        let result = classifier.lookup("ring::pbkdf2", "derive");
        assert!(!result.is_unclassified());
        assert_eq!(result.algorithm, Some("PBKDF2".to_string()));
    }

    #[test]
    fn test_lookup_javascript_crypto() {
        let classifier = RulesClassifier::from_bundled().unwrap();
        let result = classifier.lookup("crypto", "pbkdf2");
        assert!(!result.is_unclassified());
        assert_eq!(result.algorithm, Some("PBKDF2".to_string()));
    }

    #[test]
    fn test_bundled_loads_all_languages() {
        let classifier = RulesClassifier::from_bundled().unwrap();
        assert!(
            classifier.mapping_count() > 200,
            "Expected 200+ mappings from all languages, got {}",
            classifier.mapping_count()
        );
    }

    #[test]
    fn test_bundled_for_single_language() {
        let go_only = RulesClassifier::from_bundled_for_language("go").unwrap();
        let all = RulesClassifier::from_bundled().unwrap();

        assert!(go_only.mapping_count() < all.mapping_count());
        assert!(go_only.mapping_count() > 50);
    }

    #[test]
    fn test_case_insensitive_lookup() {
        let classifier = RulesClassifier::from_bundled().unwrap();

        // Function name case shouldn't matter
        let result1 = classifier.lookup("crypto/sha256", "New");
        let result2 = classifier.lookup("crypto/sha256", "new");
        let result3 = classifier.lookup("crypto/sha256", "NEW");

        assert!(!result1.is_unclassified());
        assert!(!result2.is_unclassified());
        assert!(!result3.is_unclassified());
    }
}
