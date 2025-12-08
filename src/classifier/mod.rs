mod classification;
mod rules;

pub use classification::Classification;
pub use rules::{Classifier, RulesClassifier};

pub use crate::error::ClassifierError;

use crate::scanner::CryptoCall;

#[derive(Debug, Clone)]
pub struct ClassifiedCall {
    pub call: CryptoCall,
    pub classification: Classification,
}

impl ClassifiedCall {
    pub fn new(call: CryptoCall, classification: Classification) -> Self {
        Self {
            call,
            classification,
        }
    }

    pub fn is_classified(&self) -> bool {
        !self.classification.is_unclassified()
    }
}

pub fn classify_call<C: Classifier>(call: &CryptoCall, classifier: &C) -> Classification {
    classifier.lookup_with_fallback(
        call.import_path.as_deref(),
        call.package.as_deref().unwrap_or(""),
        &call.function_name,
    )
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn make_call(
        import_path: Option<&str>,
        package: Option<&str>,
        function: &str,
        language: &str,
    ) -> CryptoCall {
        CryptoCall {
            file_path: format!("test.{language}"),
            line: 1,
            column: 1,
            function_name: function.to_string(),
            package: package.map(|s| s.to_string()),
            import_path: import_path.map(|s| s.to_string()),
            arguments: vec![],
            raw_text: format!("{function}()"),
            language: language.to_string(),
        }
    }

    #[test]
    fn test_classify_go_pbkdf2_with_import_path() {
        let classifier = RulesClassifier::from_bundled().unwrap();
        let call = make_call(
            Some("golang.org/x/crypto/pbkdf2"),
            Some("pbkdf2"),
            "Key",
            "go",
        );

        let result = classify_call(&call, &classifier);

        assert!(!result.is_unclassified());
        assert_eq!(result.algorithm, Some("PBKDF2".to_string()));
        assert_eq!(result.finding_type, "kdf");
        assert_eq!(result.operation, "keyderive");
        assert_eq!(result.primitive, Some("kdf".to_string()));
    }

    #[test]
    fn test_classify_go_stdlib_sha256() {
        let classifier = RulesClassifier::from_bundled().unwrap();
        let call = make_call(Some("crypto/sha256"), Some("sha256"), "New", "go");

        let result = classify_call(&call, &classifier);

        assert!(!result.is_unclassified());
        assert_eq!(result.algorithm, Some("SHA-256".to_string()));
        assert_eq!(result.algorithm_family, Some("SHA-2".to_string()));
        assert_eq!(result.finding_type, "hash");
    }

    #[test]
    fn test_classify_go_aes_gcm() {
        let classifier = RulesClassifier::from_bundled().unwrap();
        let call = make_call(Some("crypto/cipher"), Some("cipher"), "NewGCM", "go");

        let result = classify_call(&call, &classifier);

        assert!(!result.is_unclassified());
        assert_eq!(result.algorithm, Some("AES-GCM".to_string()));
        assert_eq!(result.mode, Some("GCM".to_string()));
        assert_eq!(result.nonce_size, Some(12));
        assert_eq!(result.tag_size, Some(16));
    }

    #[test]
    fn test_classify_python_hashlib_pbkdf2() {
        let classifier = RulesClassifier::from_bundled().unwrap();
        let call = make_call(Some("hashlib"), Some("hashlib"), "pbkdf2_hmac", "python");

        let result = classify_call(&call, &classifier);

        assert!(!result.is_unclassified());
        assert_eq!(result.algorithm, Some("PBKDF2".to_string()));
    }

    #[test]
    fn test_classify_python_cryptography() {
        let classifier = RulesClassifier::from_bundled().unwrap();
        let call = make_call(
            Some("cryptography.hazmat.primitives.ciphers.aead"),
            Some("AESGCM"),
            "AESGCM",
            "python",
        );

        let result = classify_call(&call, &classifier);

        assert!(!result.is_unclassified());
        assert_eq!(result.algorithm, Some("AES-GCM".to_string()));
    }

    #[test]
    fn test_classify_rust_ring_pbkdf2() {
        let classifier = RulesClassifier::from_bundled().unwrap();

        // Rust: import_path is "ring::pbkdf2", function is "derive"
        let call = make_call(Some("ring::pbkdf2"), Some("pbkdf2"), "derive", "rust");

        let result = classify_call(&call, &classifier);

        assert!(!result.is_unclassified());
        assert_eq!(result.algorithm, Some("PBKDF2".to_string()));
    }

    #[test]
    fn test_classify_javascript_crypto() {
        let classifier = RulesClassifier::from_bundled().unwrap();
        let call = make_call(Some("crypto"), Some("crypto"), "pbkdf2", "javascript");

        let result = classify_call(&call, &classifier);

        assert!(!result.is_unclassified());
        assert_eq!(result.algorithm, Some("PBKDF2".to_string()));
    }

    #[test]
    fn test_fallback_to_package_name() {
        let classifier = RulesClassifier::from_bundled().unwrap();
        // No import path, fallback to package name
        let call = make_call(None, Some("crypto/sha256"), "New", "go");

        let result = classify_call(&call, &classifier);

        assert!(!result.is_unclassified());
        assert_eq!(result.algorithm, Some("SHA-256".to_string()));
    }

    #[test]
    fn test_unclassified_unknown_function() {
        let classifier = RulesClassifier::from_bundled().unwrap();
        let call = make_call(Some("mypackage"), Some("mypackage"), "myFunction", "go");

        let result = classify_call(&call, &classifier);

        assert!(result.is_unclassified());
    }

    #[test]
    fn test_classified_call_wrapper() {
        let classifier = RulesClassifier::from_bundled().unwrap();
        let call = make_call(
            Some("golang.org/x/crypto/pbkdf2"),
            Some("pbkdf2"),
            "Key",
            "go",
        );

        let classification = classify_call(&call, &classifier);
        let classified = ClassifiedCall::new(call.clone(), classification);

        assert!(classified.is_classified());
        assert_eq!(classified.call.function_name, "Key");
        assert_eq!(
            classified.classification.algorithm,
            Some("PBKDF2".to_string())
        );
    }

    #[test]
    fn test_classification_contains_cbom_fields() {
        let classifier = RulesClassifier::from_bundled().unwrap();
        let call = make_call(
            Some("golang.org/x/crypto/pbkdf2"),
            Some("pbkdf2"),
            "Key",
            "go",
        );

        let result = classify_call(&call, &classifier);

        assert!(result.algorithm.is_some());
        assert!(result.algorithm_family.is_some());
        assert!(!result.finding_type.is_empty());
        assert!(!result.operation.is_empty());
        assert!(result.primitive.is_some());
        assert!(result.material_source.is_some());
    }

    #[test]
    fn test_ml_kem_post_quantum() {
        let classifier = RulesClassifier::from_bundled().unwrap();
        let call = make_call(Some("crypto/mlkem"), Some("mlkem"), "GenerateKey768", "go");

        let result = classify_call(&call, &classifier);

        assert!(!result.is_unclassified());
        assert_eq!(result.algorithm, Some("ML-KEM-768".to_string()));
        assert_eq!(result.classical_security_level, Some(192));
        assert_eq!(result.nist_quantum_security_level, Some(1));
    }

    #[test]
    fn test_tls_protocol_version() {
        let classifier = RulesClassifier::from_bundled().unwrap();
        let call = make_call(Some("crypto/tls"), Some("tls"), "Version_TLS13", "go");

        let result = classify_call(&call, &classifier);

        assert!(!result.is_unclassified());
        assert_eq!(result.protocol_name, Some("TLS".to_string()));
        assert_eq!(result.protocol_version, Some("1.3".to_string()));
    }
}
