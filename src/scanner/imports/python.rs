//! Python import extraction
//!
//! Handles Python import statements:
//! - Simple: `import hashlib`
//! - Dotted: `import cryptography.hazmat.primitives`
//! - Aliased: `import hashlib as hl`
//! - From: `from hashlib import sha256`
//! - From multiple: `from hashlib import sha256, md5`
//! - From aliased: `from hashlib import sha256 as s256`

use super::ImportMap;
use tree_sitter::{Node, Tree};

pub fn extract(tree: &Tree, source: &[u8]) -> ImportMap {
    let mut imports = ImportMap::new();
    extract_recursive(tree.root_node(), source, &mut imports);
    imports
}

fn extract_recursive(node: Node, source: &[u8], imports: &mut ImportMap) {
    match node.kind() {
        "import_statement" => {
            process_import_statement(node, source, imports);
        }
        "import_from_statement" => {
            process_from_import(node, source, imports);
        }
        _ => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                extract_recursive(child, source, imports);
            }
        }
    }
}

fn process_import_statement(node: Node, source: &[u8], imports: &mut ImportMap) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "dotted_name" => {
                let module = get_node_text(&child, source);
                let short_name = module.rsplit('.').next().unwrap_or(&module).to_string();
                imports.insert(short_name, module);
            }
            "aliased_import" => {
                process_aliased_import(child, source, imports);
            }
            _ => {}
        }
    }
}

fn process_aliased_import(node: Node, source: &[u8], imports: &mut ImportMap) {
    let mut module: Option<String> = None;
    let mut alias: Option<String> = None;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "dotted_name" => {
                module = Some(get_node_text(&child, source));
            }
            "identifier" => {
                alias = Some(get_node_text(&child, source));
            }
            _ => {}
        }
    }

    if let Some(mod_name) = module {
        let short_name =
            alias.unwrap_or_else(|| mod_name.rsplit('.').next().unwrap_or(&mod_name).to_string());
        imports.insert(short_name, mod_name);
    }
}

fn process_from_import(node: Node, source: &[u8], imports: &mut ImportMap) {
    // AST structure: from <module_path> import <imported_names>
    // We need to find the module path first, then everything after "import" keyword
    let mut module_path: Option<String> = None;
    let mut seen_import_keyword = false;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();

        // Track when we've passed the "import" keyword
        if kind == "import" {
            seen_import_keyword = true;
            continue;
        }

        if !seen_import_keyword {
            // Before "import" keyword: look for module path
            if kind == "dotted_name" || kind == "relative_import" {
                module_path = Some(get_node_text(&child, source));
            }
        } else {
            // After "import" keyword: these are the imported names
            let base = match &module_path {
                Some(m) => m.as_str(),
                None => return,
            };

            match kind {
                "wildcard_import" => {
                    imports.insert("*".to_string(), base.to_string());
                }
                "dotted_name" => {
                    // Imported name wrapped in dotted_name
                    let name = get_node_text(&child, source);
                    let full_path = format!("{base}.{name}");
                    imports.insert(name, full_path);
                }
                "identifier" => {
                    let name = get_node_text(&child, source);
                    let full_path = format!("{base}.{name}");
                    imports.insert(name, full_path);
                }
                "aliased_import" => {
                    process_from_aliased(child, source, base, imports);
                }
                _ => {
                    // Recurse into container nodes (e.g., import lists)
                    if kind != "comment" && kind != "from" {
                        collect_after_import(child, source, base, imports);
                    }
                }
            }
        }
    }
}

fn collect_after_import(node: Node, source: &[u8], base_module: &str, imports: &mut ImportMap) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "wildcard_import" => {
                imports.insert("*".to_string(), base_module.to_string());
            }
            "dotted_name" => {
                let name = get_node_text(&child, source);
                let full_path = format!("{base_module}.{name}");
                imports.insert(name, full_path);
            }
            "identifier" => {
                let name = get_node_text(&child, source);
                let full_path = format!("{base_module}.{name}");
                imports.insert(name, full_path);
            }
            "aliased_import" => {
                process_from_aliased(child, source, base_module, imports);
            }
            _ => {
                if child.kind() != "comment" {
                    collect_after_import(child, source, base_module, imports);
                }
            }
        }
    }
}

fn process_from_aliased(node: Node, source: &[u8], base_module: &str, imports: &mut ImportMap) {
    // For "sha256 as s256": first identifier is the name, second is the alias
    let mut identifiers: Vec<String> = Vec::new();

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "identifier" => {
                let text = get_node_text(&child, source);
                if text != "as" {
                    identifiers.push(text);
                }
            }
            "dotted_name" => {
                identifiers.push(get_node_text(&child, source));
            }
            _ => {}
        }
    }

    if !identifiers.is_empty() {
        let import_name = &identifiers[0];
        let alias = identifiers.get(1).unwrap_or(import_name);
        let full_path = format!("{base_module}.{import_name}");
        imports.insert(alias.clone(), full_path);
    }
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
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    #[test]
    fn test_simple_import() {
        let source = "import hashlib";
        let tree = parse(source);
        let imports = extract(&tree, source.as_bytes());

        assert_eq!(imports.len(), 1);
        assert_eq!(imports.get("hashlib"), Some(&"hashlib".to_string()));
    }

    #[test]
    fn test_dotted_import() {
        let source = "import cryptography.hazmat.primitives";
        let tree = parse(source);
        let imports = extract(&tree, source.as_bytes());

        assert_eq!(imports.len(), 1);
        assert_eq!(
            imports.get("primitives"),
            Some(&"cryptography.hazmat.primitives".to_string())
        );
    }

    #[test]
    fn test_aliased_import() {
        let source = "import hashlib as hl";
        let tree = parse(source);
        let imports = extract(&tree, source.as_bytes());

        assert_eq!(imports.len(), 1);
        assert_eq!(imports.get("hl"), Some(&"hashlib".to_string()));
    }

    #[test]
    fn test_from_import() {
        let source = "from hashlib import sha256";
        let tree = parse(source);
        let imports = extract(&tree, source.as_bytes());

        assert_eq!(imports.len(), 1);
        assert_eq!(imports.get("sha256"), Some(&"hashlib.sha256".to_string()));
    }

    #[test]
    fn test_from_import_multiple() {
        let source = "from hashlib import sha256, md5";
        let tree = parse(source);
        let imports = extract(&tree, source.as_bytes());

        assert_eq!(imports.len(), 2);
        assert_eq!(imports.get("sha256"), Some(&"hashlib.sha256".to_string()));
        assert_eq!(imports.get("md5"), Some(&"hashlib.md5".to_string()));
    }

    #[test]
    fn test_from_import_aliased() {
        let source = "from hashlib import sha256 as s256";
        let tree = parse(source);
        let imports = extract(&tree, source.as_bytes());

        assert_eq!(imports.len(), 1);
        assert_eq!(imports.get("s256"), Some(&"hashlib.sha256".to_string()));
    }

    #[test]
    fn test_from_deep_import() {
        let source = "from cryptography.hazmat.primitives import hashes";
        let tree = parse(source);
        let imports = extract(&tree, source.as_bytes());

        assert_eq!(imports.len(), 1);
        assert_eq!(
            imports.get("hashes"),
            Some(&"cryptography.hazmat.primitives.hashes".to_string())
        );
    }

    #[test]
    fn test_from_wildcard() {
        let source = "from hashlib import *";
        let tree = parse(source);
        let imports = extract(&tree, source.as_bytes());

        assert_eq!(imports.len(), 1);
        assert_eq!(imports.get("*"), Some(&"hashlib".to_string()));
    }
}
