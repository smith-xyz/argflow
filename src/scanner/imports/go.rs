//! Go import extraction
//!
//! Handles Go import declarations:
//! - Simple: `import "crypto/sha256"`
//! - Grouped: `import ("crypto/sha256" "crypto/sha512")`
//! - Aliased: `import pb "golang.org/x/crypto/pbkdf2"`
//! - Dot import: `import . "crypto/sha256"`
//! - Blank import: `import _ "crypto/sha256"`

use super::{unquote_string, ImportMap};
use tree_sitter::{Node, Tree};

pub fn extract(tree: &Tree, source: &[u8]) -> ImportMap {
    let mut imports = ImportMap::new();
    extract_recursive(tree.root_node(), source, &mut imports);
    imports
}

fn extract_recursive(node: Node, source: &[u8], imports: &mut ImportMap) {
    match node.kind() {
        "import_declaration" => {
            process_import_declaration(node, source, imports);
        }
        _ => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                extract_recursive(child, source, imports);
            }
        }
    }
}

fn process_import_declaration(node: Node, source: &[u8], imports: &mut ImportMap) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "import_spec" => {
                process_import_spec(child, source, imports);
            }
            "import_spec_list" => {
                let mut list_cursor = child.walk();
                for spec in child.children(&mut list_cursor) {
                    if spec.kind() == "import_spec" {
                        process_import_spec(spec, source, imports);
                    }
                }
            }
            _ => {}
        }
    }
}

fn process_import_spec(node: Node, source: &[u8], imports: &mut ImportMap) {
    let mut alias: Option<String> = None;
    let mut path: Option<String> = None;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "package_identifier" | "blank_identifier" | "dot" => {
                alias = Some(get_node_text(&child, source));
            }
            "interpreted_string_literal" => {
                let text = get_node_text(&child, source);
                path = Some(unquote_string(&text));
            }
            _ => {}
        }
    }

    if let Some(import_path) = path {
        let short_name = alias.unwrap_or_else(|| extract_package_name(&import_path));
        imports.insert(short_name, import_path);
    }
}

fn extract_package_name(import_path: &str) -> String {
    import_path
        .rsplit('/')
        .next()
        .unwrap_or(import_path)
        .to_string()
}

fn get_node_text<'a>(node: &Node<'a>, source: &'a [u8]) -> String {
    node.utf8_text(source).unwrap_or("").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> Tree {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    #[test]
    fn test_simple_import() {
        let source = r#"
package main
import "crypto/sha256"
"#;
        let tree = parse(source);
        let imports = extract(&tree, source.as_bytes());

        assert_eq!(imports.len(), 1);
        assert_eq!(imports.get("sha256"), Some(&"crypto/sha256".to_string()));
    }

    #[test]
    fn test_extended_import() {
        let source = r#"
package main
import "golang.org/x/crypto/pbkdf2"
"#;
        let tree = parse(source);
        let imports = extract(&tree, source.as_bytes());

        assert_eq!(imports.len(), 1);
        assert_eq!(
            imports.get("pbkdf2"),
            Some(&"golang.org/x/crypto/pbkdf2".to_string())
        );
    }

    #[test]
    fn test_aliased_import() {
        let source = r#"
package main
import pb "golang.org/x/crypto/pbkdf2"
"#;
        let tree = parse(source);
        let imports = extract(&tree, source.as_bytes());

        assert_eq!(imports.len(), 1);
        assert_eq!(
            imports.get("pb"),
            Some(&"golang.org/x/crypto/pbkdf2".to_string())
        );
    }

    #[test]
    fn test_grouped_imports() {
        let source = r#"
package main
import (
    "crypto/sha256"
    "crypto/sha512"
    pb "golang.org/x/crypto/pbkdf2"
)
"#;
        let tree = parse(source);
        let imports = extract(&tree, source.as_bytes());

        assert_eq!(imports.len(), 3);
        assert_eq!(imports.get("sha256"), Some(&"crypto/sha256".to_string()));
        assert_eq!(imports.get("sha512"), Some(&"crypto/sha512".to_string()));
        assert_eq!(
            imports.get("pb"),
            Some(&"golang.org/x/crypto/pbkdf2".to_string())
        );
    }

    #[test]
    fn test_dot_import() {
        let source = r#"
package main
import . "crypto/sha256"
"#;
        let tree = parse(source);
        let imports = extract(&tree, source.as_bytes());

        assert_eq!(imports.len(), 1);
        assert_eq!(imports.get("."), Some(&"crypto/sha256".to_string()));
    }

    #[test]
    fn test_blank_import() {
        let source = r#"
package main
import _ "crypto/sha256"
"#;
        let tree = parse(source);
        let imports = extract(&tree, source.as_bytes());

        assert_eq!(imports.len(), 1);
        assert_eq!(imports.get("_"), Some(&"crypto/sha256".to_string()));
    }
}
