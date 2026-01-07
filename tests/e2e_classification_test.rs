use argflow::classifier::{classify_call, RulesClassifier};
use argflow::scanner::Scanner;

fn parse_go(source: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_go::LANGUAGE.into())
        .unwrap();
    parser.parse(source, None).unwrap()
}

fn parse_python(source: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .unwrap();
    parser.parse(source, None).unwrap()
}

fn create_scanner_with_mappings() -> (Scanner, RulesClassifier) {
    let classifier = RulesClassifier::from_bundled().unwrap();
    let scanner = Scanner::with_mappings(classifier.get_mappings().clone());
    (scanner, classifier)
}

#[test]
fn test_e2e_go_pbkdf2_classification() {
    let source = r#"
package main

import "golang.org/x/crypto/pbkdf2"

func main() {
    key := pbkdf2.Key(password, salt, 100000, 32, sha256.New)
}
"#;

    let tree = parse_go(source);
    let (scanner, classifier) = create_scanner_with_mappings();

    let result = scanner.scan_tree(&tree, source.as_bytes(), "test.go", "go");

    assert_eq!(result.call_count(), 1, "Should find one crypto call");

    let call = &result.calls[0];
    assert_eq!(call.function_name, "Key");
    assert_eq!(call.package, Some("pbkdf2".to_string()));
    assert_eq!(
        call.import_path,
        Some("golang.org/x/crypto/pbkdf2".to_string())
    );

    let classification = classify_call(call, &classifier);
    assert_eq!(classification.algorithm, Some("PBKDF2".to_string()));
    assert_eq!(classification.finding_type, "kdf");
    assert_eq!(classification.operation, "keyderive");
}

#[test]
fn test_e2e_go_sha256_classification() {
    let source = r#"
package main

import "crypto/sha256"

func main() {
    h := sha256.New()
}
"#;

    let tree = parse_go(source);
    let (scanner, classifier) = create_scanner_with_mappings();

    let result = scanner.scan_tree(&tree, source.as_bytes(), "test.go", "go");

    assert_eq!(result.call_count(), 1);

    let call = &result.calls[0];
    assert_eq!(call.function_name, "New");
    assert_eq!(call.import_path, Some("crypto/sha256".to_string()));

    let classification = classify_call(call, &classifier);
    assert_eq!(classification.algorithm, Some("SHA-256".to_string()));
    assert_eq!(classification.finding_type, "hash");
}

#[test]
fn test_e2e_go_aes_gcm_classification() {
    let source = r#"
package main

import "crypto/cipher"

func main() {
    gcm, _ := cipher.NewGCM(block)
}
"#;

    let tree = parse_go(source);
    let (scanner, classifier) = create_scanner_with_mappings();

    let result = scanner.scan_tree(&tree, source.as_bytes(), "test.go", "go");

    assert_eq!(result.call_count(), 1);

    let call = &result.calls[0];
    assert_eq!(call.function_name, "NewGCM");
    assert_eq!(call.import_path, Some("crypto/cipher".to_string()));

    let classification = classify_call(call, &classifier);
    assert_eq!(classification.algorithm, Some("AES-GCM".to_string()));
    assert_eq!(classification.mode, Some("GCM".to_string()));
}

#[test]
fn test_e2e_go_argument_resolution() {
    let source = r#"
package main

import "golang.org/x/crypto/pbkdf2"

func main() {
    key := pbkdf2.Key(password, salt, 100000, 32, sha256.New)
}
"#;

    let tree = parse_go(source);
    let (scanner, _classifier) = create_scanner_with_mappings();

    let result = scanner.scan_tree(&tree, source.as_bytes(), "test.go", "go");

    assert_eq!(result.call_count(), 1);

    let call = &result.calls[0];
    assert_eq!(call.arguments.len(), 5);

    // iterations = 100000
    let iterations = &call.arguments[2];
    assert!(iterations.is_resolved);
    assert_eq!(iterations.int_values, vec![100000]);

    // key_length = 32
    let key_length = &call.arguments[3];
    assert!(key_length.is_resolved);
    assert_eq!(key_length.int_values, vec![32]);
}

#[test]
fn test_e2e_python_hashlib_classification() {
    let source = r#"
import hashlib

key = hashlib.pbkdf2_hmac('sha256', password, salt, 100000)
"#;

    let tree = parse_python(source);
    let (scanner, classifier) = create_scanner_with_mappings();

    let result = scanner.scan_tree(&tree, source.as_bytes(), "test.py", "python");

    assert_eq!(result.call_count(), 1);

    let call = &result.calls[0];
    assert_eq!(call.function_name, "pbkdf2_hmac");
    assert_eq!(call.package, Some("hashlib".to_string()));

    let classification = classify_call(call, &classifier);
    assert_eq!(classification.algorithm, Some("PBKDF2".to_string()));
    assert_eq!(classification.finding_type, "kdf");
}

#[test]
fn test_e2e_python_argument_resolution() {
    let source = r#"
import hashlib

key = hashlib.pbkdf2_hmac('sha256', password, salt, 100000, dklen=32)
"#;

    let tree = parse_python(source);
    let (scanner, _classifier) = create_scanner_with_mappings();

    let result = scanner.scan_tree(&tree, source.as_bytes(), "test.py", "python");

    assert_eq!(result.call_count(), 1);

    let call = &result.calls[0];

    // First argument should be 'sha256'
    let hash_name = &call.arguments[0];
    assert!(hash_name.is_resolved);
    assert_eq!(hash_name.string_values, vec!["sha256"]);

    // Fourth argument should be 100000
    let iterations = &call.arguments[3];
    assert!(iterations.is_resolved);
    assert_eq!(iterations.int_values, vec![100000]);
}

#[test]
fn test_e2e_unclassified_call() {
    let source = r#"
package main

import "mycompany/pkg"

func main() {
    result := pkg.DoSomething()
}
"#;

    let tree = parse_go(source);
    let (scanner, _classifier) = create_scanner_with_mappings();

    let result = scanner.scan_tree(&tree, source.as_bytes(), "test.go", "go");

    // This call won't be detected as crypto because it doesn't have a mapping
    assert_eq!(result.call_count(), 0);
}

#[test]
fn test_e2e_multiple_calls_same_file() {
    let source = r#"
package main

import (
    "crypto/sha256"
    "golang.org/x/crypto/pbkdf2"
)

func main() {
    h := sha256.New()
    key := pbkdf2.Key(password, salt, 100000, 32, sha256.New)
}
"#;

    let tree = parse_go(source);
    let (scanner, classifier) = create_scanner_with_mappings();

    let result = scanner.scan_tree(&tree, source.as_bytes(), "test.go", "go");

    assert!(
        result.call_count() >= 2,
        "Should find at least 2 crypto calls"
    );

    // Find the sha256.New call
    let sha_call = result
        .calls
        .iter()
        .find(|c| c.function_name == "New" && c.package == Some("sha256".to_string()));
    assert!(sha_call.is_some(), "Should find sha256.New call");

    let sha_classification = classify_call(sha_call.unwrap(), &classifier);
    assert_eq!(sha_classification.algorithm, Some("SHA-256".to_string()));

    // Find the pbkdf2.Key call
    let pbkdf2_call = result
        .calls
        .iter()
        .find(|c| c.function_name == "Key" && c.package == Some("pbkdf2".to_string()));
    assert!(pbkdf2_call.is_some(), "Should find pbkdf2.Key call");

    let pbkdf2_classification = classify_call(pbkdf2_call.unwrap(), &classifier);
    assert_eq!(pbkdf2_classification.algorithm, Some("PBKDF2".to_string()));
}
