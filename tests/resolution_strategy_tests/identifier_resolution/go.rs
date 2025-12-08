//! Go identifier resolution tests

use super::test_utils::{
    get_arg_source, get_first_arg_int, get_first_arg_string, is_arg_resolved, scan_go,
};

// =============================================================================
// Local Variable Resolution
// =============================================================================

#[test]
fn test_local_variable_short_decl() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() {
    iterations := 100000
    pbkdf2.Key(p, s, iterations, 32, h)
}
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(100000),
        "Local variable via short declaration"
    );
}

#[test]
fn test_local_variable_var_decl() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() {
    var keyLen = 32
    pbkdf2.Key(p, s, 10000, keyLen, h)
}
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(32),
        "Local variable via var declaration"
    );
}

#[test]
fn test_multiple_variables() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() {
    iterations := 100000
    keyLen := 32
    pbkdf2.Key(p, s, iterations, keyLen, h)
}
"#,
    );
    assert_eq!(get_first_arg_int(&result, 2), Some(100000), "iterations");
    assert_eq!(get_first_arg_int(&result, 3), Some(32), "keyLen");
}

// =============================================================================
// File-Level Constants
// =============================================================================

#[test]
fn test_file_level_const() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
const DefaultIterations = 100000
func main() {
    pbkdf2.Key(p, s, DefaultIterations, 32, h)
}
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(100000),
        "File-level constant"
    );
}

#[test]
fn test_file_level_var() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
var GlobalKeyLen = 256
func main() {
    pbkdf2.Key(p, s, 10000, GlobalKeyLen, h)
}
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 3),
        Some(256),
        "File-level variable"
    );
}

// =============================================================================
// Function Parameters (Unresolved)
// =============================================================================

#[test]
fn test_function_parameter_unresolved() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func DeriveKey(iterations int) {
    pbkdf2.Key(p, s, iterations, 32, h)
}
"#,
    );
    assert!(
        !is_arg_resolved(&result, 2),
        "Function parameter should not be resolved"
    );
    assert_eq!(
        get_arg_source(&result, 2),
        Some("function_parameter".to_string()),
        "Should be marked as function_parameter"
    );
}

// =============================================================================
// Variable Shadowing
// =============================================================================

#[test]
fn test_local_shadows_global() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
const iterations = 10000
func main() {
    iterations := 100000
    pbkdf2.Key(p, s, iterations, 32, h)
}
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(100000),
        "Local should shadow global"
    );
}

// =============================================================================
// Variable Reassignment
// =============================================================================

#[test]
fn test_variable_reassignment() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() {
    iterations := 10000
    iterations = 100000
    pbkdf2.Key(p, s, iterations, 32, h)
}
"#,
    );
    assert_eq!(
        get_first_arg_int(&result, 2),
        Some(100000),
        "Should use latest assignment"
    );
}

// =============================================================================
// String Variables
// =============================================================================

#[test]
fn test_string_variable() {
    let result = scan_go(
        r#"
package main
import "crypto/sha256"
func main() {
    algorithm := "sha256"
    sha256.New(algorithm)
}
"#,
    );
    assert_eq!(
        get_first_arg_string(&result, 0),
        Some("sha256".to_string()),
        "String variable resolution"
    );
}

#[test]
fn test_string_constant() {
    let result = scan_go(
        r#"
package main
import "crypto/sha256"
const Algorithm = "sha256"
func main() {
    sha256.New(Algorithm)
}
"#,
    );
    assert_eq!(
        get_first_arg_string(&result, 0),
        Some("sha256".to_string()),
        "String constant resolution"
    );
}

// =============================================================================
// Identifier Not Found
// =============================================================================

#[test]
fn test_identifier_not_found() {
    let result = scan_go(
        r#"
package main
import "golang.org/x/crypto/pbkdf2"
func main() {
    pbkdf2.Key(p, s, undeclaredVar, 32, h)
}
"#,
    );
    assert!(
        !is_arg_resolved(&result, 2),
        "Undeclared variable should not be resolved"
    );
    assert_eq!(
        get_arg_source(&result, 2),
        Some("identifier_not_found".to_string()),
        "Should be marked as identifier_not_found"
    );
}
