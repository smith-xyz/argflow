//! Go-specific call resolution tests

use super::test_utils::*;

#[test]
fn test_go_simple_int_return() {
    let source = r#"
package main

import "golang.org/x/crypto/pbkdf2"

func getIterations() int {
    return 10000
}

func main() {
    pbkdf2.Key(nil, nil, getIterations(), 32, nil)
}"#;
    let result = scan_go(source);

    assert_eq!(result.calls.len(), 1);
    assert!(is_arg_resolved(&result, 2));
    assert_eq!(get_first_arg_int(&result, 2), Some(10000));
}

#[test]
fn test_go_simple_string_return() {
    let source = r#"
package main

import "crypto/sha256"

func getAlgorithm() string {
    return "sha256"
}

func main() {
    sha256.New(getAlgorithm())
}"#;
    let result = scan_go(source);

    assert_eq!(result.calls.len(), 1);
    assert!(is_arg_resolved(&result, 0));
    assert_eq!(get_first_arg_string(&result, 0), Some("sha256".to_string()));
}

#[test]
fn test_go_if_else_returns() {
    let source = r#"
package main

import "golang.org/x/crypto/pbkdf2"

func getKeySize(aes256 bool) int {
    if aes256 {
        return 32
    }
    return 16
}

func main() {
    pbkdf2.Key(nil, nil, 10000, getKeySize(true), nil)
}"#;
    let result = scan_go(source);

    assert_eq!(result.calls.len(), 1);
    let key_sizes = get_first_arg_ints(&result, 3);
    assert!(key_sizes.contains(&32));
    assert!(key_sizes.contains(&16));
}

#[test]
fn test_go_switch_returns() {
    let source = r#"
package main

import "golang.org/x/crypto/pbkdf2"

func getBlockSize(mode string) int {
    switch mode {
    case "AES-128":
        return 16
    case "AES-192":
        return 24
    case "AES-256":
        return 32
    default:
        return 16
    }
}

func main() {
    pbkdf2.Key(nil, nil, 10000, getBlockSize("AES-256"), nil)
}"#;
    let result = scan_go(source);

    assert_eq!(result.calls.len(), 1);
    let sizes = get_first_arg_ints(&result, 3);
    assert!(sizes.contains(&16));
    assert!(sizes.contains(&24));
    assert!(sizes.contains(&32));
}

#[test]
fn test_go_tuple_return() {
    let source = r#"
package main

import "golang.org/x/crypto/pbkdf2"

func getConfig() (int, int) {
    return 10000, 32
}

func main() {
    iter, keyLen := getConfig()
    pbkdf2.Key(nil, nil, iter, keyLen, nil)
}"#;
    let result = scan_go(source);

    // The tuple return will be resolved, but assignment tracking
    // requires identifier strategy to trace iter and keyLen
    assert_eq!(result.calls.len(), 1);
}

#[test]
fn test_go_function_not_found() {
    let source = r#"
package main

import "golang.org/x/crypto/pbkdf2"

func main() {
    pbkdf2.Key(nil, nil, unknownFunction(), 32, nil)
}"#;
    let result = scan_go(source);

    assert_eq!(result.calls.len(), 1);
    assert!(!is_arg_resolved(&result, 2));
    assert_eq!(
        get_arg_source(&result, 2),
        Some("function_not_found".to_string())
    );
}

#[test]
fn test_go_unresolvable_return() {
    let source = r#"
package main

import "golang.org/x/crypto/pbkdf2"

func getIterations(multiplier int) int {
    return multiplier * 1000
}

func main() {
    pbkdf2.Key(nil, nil, getIterations(10), 32, nil)
}"#;
    let result = scan_go(source);

    assert_eq!(result.calls.len(), 1);
    // Function return depends on parameter, so partially resolved
    assert!(!is_arg_resolved(&result, 2));
}
