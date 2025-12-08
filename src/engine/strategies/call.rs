use crate::engine::{Context, Language, NodeCategory, Strategy, UnresolvedSource, Value};
use tree_sitter::Node;

pub struct CallStrategy;

impl Default for CallStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl CallStrategy {
    pub fn new() -> Self {
        Self
    }

    fn get_function_name<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Option<String> {
        let lang = ctx.node_types()?.language();

        match lang {
            Language::Go | Language::Rust | Language::C | Language::Cpp => node
                .child_by_field_name("function")
                .map(|n| ctx.get_node_text(&n)),
            Language::Python => node
                .child_by_field_name("function")
                .map(|n| ctx.get_node_text(&n)),
            Language::JavaScript | Language::TypeScript => node
                .child_by_field_name("function")
                .map(|n| ctx.get_node_text(&n)),
            Language::Java => node
                .child_by_field_name("name")
                .map(|n| ctx.get_node_text(&n)),
        }
    }

    fn find_function_declaration<'a>(
        &self,
        name: &str,
        root: Node<'a>,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        if let Some(func_info) = ctx.find_cross_file_function(name) {
            if let Some(node) = ctx.find_node_at_position(func_info.start_byte, func_info.end_byte)
            {
                return Some(node);
            }
        }

        self.find_function_in_tree(name, root, ctx)
    }

    fn find_function_in_tree<'a>(
        &self,
        name: &str,
        node: Node<'a>,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        if ctx.is_node_category(node.kind(), NodeCategory::FunctionDeclaration) {
            if let Some(func_name) = self.get_function_declaration_name(&node, ctx) {
                if func_name == name {
                    return Some(node);
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = self.find_function_in_tree(name, child, ctx) {
                return Some(found);
            }
        }

        None
    }

    fn get_function_declaration_name<'a>(
        &self,
        node: &Node<'a>,
        ctx: &Context<'a>,
    ) -> Option<String> {
        let lang = ctx.node_types()?.language();

        match lang {
            Language::Go => node
                .child_by_field_name("name")
                .map(|n| ctx.get_node_text(&n)),
            Language::Python => node
                .child_by_field_name("name")
                .map(|n| ctx.get_node_text(&n)),
            Language::Rust => node
                .child_by_field_name("name")
                .map(|n| ctx.get_node_text(&n)),
            Language::JavaScript | Language::TypeScript => node
                .child_by_field_name("name")
                .map(|n| ctx.get_node_text(&n)),
            Language::C | Language::Cpp => node
                .child_by_field_name("declarator")
                .and_then(|d| d.child_by_field_name("declarator"))
                .map(|n| ctx.get_node_text(&n)),
            Language::Java => node
                .child_by_field_name("name")
                .map(|n| ctx.get_node_text(&n)),
        }
    }

    fn get_function_body<'a>(&self, func: &Node<'a>, ctx: &Context<'a>) -> Option<Node<'a>> {
        let lang = ctx.node_types()?.language();

        match lang {
            Language::Go | Language::Rust | Language::C | Language::Cpp | Language::Java => {
                func.child_by_field_name("body")
            }
            Language::Python => func.child_by_field_name("body"),
            Language::JavaScript | Language::TypeScript => func
                .child_by_field_name("body")
                .or_else(|| func.named_child(func.named_child_count().saturating_sub(1))),
        }
    }

    fn collect_return_values<'a>(&self, body: Node<'a>, ctx: &Context<'a>) -> Vec<Value> {
        let mut values = Vec::new();
        self.collect_returns_recursive(body, ctx, &mut values);
        values
    }

    fn collect_returns_recursive<'a>(
        &self,
        node: Node<'a>,
        ctx: &Context<'a>,
        values: &mut Vec<Value>,
    ) {
        if ctx.is_node_category(node.kind(), NodeCategory::ReturnStatement) {
            if let Some(return_value) = self.extract_return_value(&node, ctx) {
                values.push(return_value);
            }
            return;
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if ctx.is_node_category(child.kind(), NodeCategory::FunctionDeclaration) {
                continue;
            }
            self.collect_returns_recursive(child, ctx, values);
        }
    }

    fn extract_return_value<'a>(&self, return_node: &Node<'a>, ctx: &Context<'a>) -> Option<Value> {
        let lang = ctx.node_types()?.language();

        match lang {
            Language::Go => self.extract_go_return(return_node, ctx),
            Language::Python => self.extract_python_return(return_node, ctx),
            Language::Rust => self.extract_rust_return(return_node, ctx),
            Language::JavaScript | Language::TypeScript => self.extract_js_return(return_node, ctx),
            Language::C | Language::Cpp => self.extract_c_return(return_node, ctx),
            Language::Java => self.extract_java_return(return_node, ctx),
        }
    }

    fn extract_go_return<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Option<Value> {
        let mut cursor = node.walk();
        let children: Vec<_> = node
            .children(&mut cursor)
            .filter(|c| c.is_named())
            .collect();

        if children.is_empty() {
            return None;
        }

        if children.len() == 1 {
            let child = children[0];
            if child.kind() == "expression_list" {
                return Some(self.resolve_expression_list(&child, ctx));
            }
            return Some(self.resolve_value_node(child, ctx));
        }

        Some(self.resolve_multiple_values(&children, ctx))
    }

    fn extract_python_return<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Option<Value> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() && child.kind() != "comment" {
                if child.kind() == "tuple" || child.kind() == "expression_list" {
                    return Some(self.resolve_tuple(&child, ctx));
                }
                return Some(self.resolve_value_node(child, ctx));
            }
        }
        None
    }

    fn extract_rust_return<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Option<Value> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                if child.kind() == "tuple_expression" {
                    return Some(self.resolve_tuple(&child, ctx));
                }
                return Some(self.resolve_value_node(child, ctx));
            }
        }
        None
    }

    fn extract_js_return<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Option<Value> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                return Some(self.resolve_value_node(child, ctx));
            }
        }
        None
    }

    fn extract_c_return<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Option<Value> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                return Some(self.resolve_value_node(child, ctx));
            }
        }
        None
    }

    fn extract_java_return<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Option<Value> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                return Some(self.resolve_value_node(child, ctx));
            }
        }
        None
    }

    fn resolve_expression_list<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        let mut cursor = node.walk();
        let children: Vec<_> = node
            .children(&mut cursor)
            .filter(|c| c.is_named())
            .collect();
        self.resolve_multiple_values(&children, ctx)
    }

    fn resolve_tuple<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        let mut cursor = node.walk();
        let children: Vec<_> = node
            .children(&mut cursor)
            .filter(|c| c.is_named())
            .collect();
        self.resolve_multiple_values(&children, ctx)
    }

    fn resolve_multiple_values<'a>(&self, nodes: &[Node<'a>], ctx: &Context<'a>) -> Value {
        let mut all_ints = Vec::new();
        let mut all_strings = Vec::new();
        let mut all_resolved = true;

        for node in nodes {
            let value = self.resolve_value_node(*node, ctx);
            if value.is_resolved {
                all_ints.extend(value.int_values);
                all_strings.extend(value.string_values);
            } else {
                all_resolved = false;
            }
        }

        if all_resolved && (!all_ints.is_empty() || !all_strings.is_empty()) {
            Value {
                is_resolved: true,
                int_values: all_ints,
                string_values: all_strings,
                source: String::new(),
                expression: String::new(),
            }
        } else {
            let texts: Vec<_> = nodes.iter().map(|n| ctx.get_node_text(n)).collect();
            Value::partial_expression(texts.join(", "))
        }
    }

    fn resolve_value_node<'a>(&self, node: Node<'a>, ctx: &Context<'a>) -> Value {
        let kind = node.kind();

        if ctx.is_node_category(kind, NodeCategory::IntegerLiteral) {
            let text = ctx.get_node_text(&node);
            if let Some(value) = ctx.parse_int_literal(&text) {
                return Value::resolved_int(value);
            }
        }

        if ctx.is_node_category(kind, NodeCategory::StringLiteral) {
            let text = ctx.get_node_text(&node);
            let unquoted = ctx.unquote_string(&text);
            return Value::resolved_string(unquoted);
        }

        if ctx.is_node_category(kind, NodeCategory::BooleanLiteral) {
            let text = ctx.get_node_text(&node);
            if text.eq_ignore_ascii_case("true") {
                return Value::resolved_string("true".to_string());
            } else if text.eq_ignore_ascii_case("false") {
                return Value::resolved_string("false".to_string());
            }
        }

        if ctx.is_node_category(kind, NodeCategory::NilLiteral) {
            return Value::resolved_string(kind.to_string());
        }

        Value::partial_expression(ctx.get_node_text(&node))
    }

    fn merge_return_values(&self, values: Vec<Value>) -> Value {
        if values.is_empty() {
            return Value::unextractable(UnresolvedSource::NotImplemented);
        }

        if values.len() == 1 {
            return values.into_iter().next().unwrap();
        }

        let mut all_ints = Vec::new();
        let mut all_strings = Vec::new();
        let mut any_unresolved = false;
        let mut expressions = Vec::new();

        for value in values {
            if value.is_resolved {
                for i in value.int_values {
                    if !all_ints.contains(&i) {
                        all_ints.push(i);
                    }
                }
                for s in value.string_values {
                    if !all_strings.contains(&s) {
                        all_strings.push(s);
                    }
                }
            } else {
                any_unresolved = true;
                if !value.expression.is_empty() {
                    expressions.push(value.expression);
                }
            }
        }

        if !all_ints.is_empty() || !all_strings.is_empty() {
            Value {
                is_resolved: !any_unresolved,
                int_values: all_ints,
                string_values: all_strings,
                source: String::new(),
                expression: if any_unresolved {
                    expressions.join(" | ")
                } else {
                    String::new()
                },
            }
        } else if !expressions.is_empty() {
            Value::partial_expression(expressions.join(" | "))
        } else {
            Value::unextractable(UnresolvedSource::NotImplemented)
        }
    }
}

impl Strategy for CallStrategy {
    fn name(&self) -> &'static str {
        "call"
    }

    fn can_handle<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> bool {
        ctx.is_node_category(node.kind(), NodeCategory::CallExpression)
    }

    fn resolve<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        let func_name = match self.get_function_name(node, ctx) {
            Some(name) => name,
            None => return Value::unextractable(UnresolvedSource::Unknown),
        };

        let simple_name = func_name.split('.').next_back().unwrap_or(&func_name);

        let func_decl =
            match self.find_function_declaration(simple_name, ctx.tree().root_node(), ctx) {
                Some(decl) => decl,
                None => {
                    return Value::unextractable(UnresolvedSource::FunctionNotFound);
                }
            };

        let body = match self.get_function_body(&func_decl, ctx) {
            Some(b) => b,
            None => return Value::unextractable(UnresolvedSource::Unknown),
        };

        let return_values = self.collect_return_values(body, ctx);

        if return_values.is_empty() {
            return Value::unextractable(UnresolvedSource::NotImplemented);
        }

        self.merge_return_values(return_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tree_sitter::Tree;

    fn parse_go(source: &str) -> Tree {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    fn parse_python(source: &str) -> Tree {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    fn parse_rust(source: &str) -> Tree {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    fn parse_javascript(source: &str) -> Tree {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    fn create_go_context<'a>(tree: &'a Tree, source: &'a [u8]) -> Context<'a> {
        Context::new(
            tree,
            source,
            "test.go".to_string(),
            "go".to_string(),
            HashMap::new(),
        )
    }

    fn create_python_context<'a>(tree: &'a Tree, source: &'a [u8]) -> Context<'a> {
        Context::new(
            tree,
            source,
            "test.py".to_string(),
            "python".to_string(),
            HashMap::new(),
        )
    }

    fn create_rust_context<'a>(tree: &'a Tree, source: &'a [u8]) -> Context<'a> {
        Context::new(
            tree,
            source,
            "test.rs".to_string(),
            "rust".to_string(),
            HashMap::new(),
        )
    }

    fn create_js_context<'a>(tree: &'a Tree, source: &'a [u8]) -> Context<'a> {
        Context::new(
            tree,
            source,
            "test.js".to_string(),
            "javascript".to_string(),
            HashMap::new(),
        )
    }

    fn find_first_call<'a>(node: Node<'a>, ctx: &Context<'a>) -> Option<Node<'a>> {
        if ctx.is_node_category(node.kind(), NodeCategory::CallExpression) {
            return Some(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = find_first_call(child, ctx) {
                return Some(found);
            }
        }
        None
    }

    fn find_call_by_name<'a>(node: Node<'a>, name: &str, ctx: &Context<'a>) -> Option<Node<'a>> {
        if ctx.is_node_category(node.kind(), NodeCategory::CallExpression) {
            let text = ctx.get_node_text(&node);
            if text.contains(name) {
                return Some(node);
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = find_call_by_name(child, name, ctx) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn test_strategy_name() {
        let strategy = CallStrategy::new();
        assert_eq!(strategy.name(), "call");
    }

    #[test]
    fn test_can_handle_call() {
        let source = "package main\nfunc main() { foo() }";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = CallStrategy::new();

        let call_node = find_first_call(tree.root_node(), &ctx).unwrap();
        assert!(strategy.can_handle(&call_node, &ctx));
    }

    #[test]
    fn test_cannot_handle_literal() {
        let source = "package main\nconst x = 10000";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = CallStrategy::new();

        let root = tree.root_node();
        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            if child.kind() == "const_declaration" {
                assert!(!strategy.can_handle(&child, &ctx));
            }
        }
    }

    // =========================================================================
    // Go - Simple Return Tests
    // =========================================================================

    #[test]
    fn test_go_simple_int_return() {
        let source = r#"
package main

func getIterations() int {
    return 10000
}

func main() {
    x := getIterations()
}"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = CallStrategy::new();

        let call_node = find_call_by_name(tree.root_node(), "getIterations", &ctx).unwrap();
        let value = strategy.resolve(&call_node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![10000]);
    }

    #[test]
    fn test_go_simple_string_return() {
        let source = r#"
package main

func getAlgorithm() string {
    return "sha256"
}

func main() {
    x := getAlgorithm()
}"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = CallStrategy::new();

        let call_node = find_call_by_name(tree.root_node(), "getAlgorithm", &ctx).unwrap();
        let value = strategy.resolve(&call_node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.string_values, vec!["sha256"]);
    }

    #[test]
    fn test_go_function_not_found() {
        let source = r#"
package main

func main() {
    x := unknownFunction()
}"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = CallStrategy::new();

        let call_node = find_call_by_name(tree.root_node(), "unknownFunction", &ctx).unwrap();
        let value = strategy.resolve(&call_node, &ctx);

        assert!(!value.is_resolved);
        assert_eq!(value.source, "function_not_found");
    }

    // =========================================================================
    // Go - Multiple Return Paths (Control Flow)
    // =========================================================================

    #[test]
    fn test_go_if_else_returns() {
        let source = r#"
package main

func getKeySize(aes256 bool) int {
    if aes256 {
        return 32
    }
    return 16
}

func main() {
    x := getKeySize(true)
}"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = CallStrategy::new();

        let call_node = find_call_by_name(tree.root_node(), "getKeySize", &ctx).unwrap();
        let value = strategy.resolve(&call_node, &ctx);

        assert!(value.is_resolved);
        assert!(value.int_values.contains(&32));
        assert!(value.int_values.contains(&16));
    }

    #[test]
    fn test_go_switch_returns() {
        let source = r#"
package main

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
    x := getBlockSize("AES-256")
}"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = CallStrategy::new();

        let call_node = find_call_by_name(tree.root_node(), "getBlockSize", &ctx).unwrap();
        let value = strategy.resolve(&call_node, &ctx);

        assert!(value.is_resolved);
        assert!(value.int_values.contains(&16));
        assert!(value.int_values.contains(&24));
        assert!(value.int_values.contains(&32));
    }

    // =========================================================================
    // Go - Tuple Returns
    // =========================================================================

    #[test]
    fn test_go_tuple_return() {
        let source = r#"
package main

func getConfig() (int, string) {
    return 10000, "sha256"
}

func main() {
    iter, algo := getConfig()
}"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = CallStrategy::new();

        let call_node = find_call_by_name(tree.root_node(), "getConfig", &ctx).unwrap();
        let value = strategy.resolve(&call_node, &ctx);

        assert!(value.is_resolved);
        assert!(value.int_values.contains(&10000));
        assert!(value.string_values.contains(&"sha256".to_string()));
    }

    // =========================================================================
    // Python Tests
    // =========================================================================

    #[test]
    fn test_python_simple_int_return() {
        let source = r#"
def get_iterations():
    return 10000

x = get_iterations()
"#;
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = CallStrategy::new();

        let call_node = find_call_by_name(tree.root_node(), "get_iterations", &ctx).unwrap();
        let value = strategy.resolve(&call_node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![10000]);
    }

    #[test]
    fn test_python_simple_string_return() {
        let source = r#"
def get_algorithm():
    return "sha256"

x = get_algorithm()
"#;
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = CallStrategy::new();

        let call_node = find_call_by_name(tree.root_node(), "get_algorithm", &ctx).unwrap();
        let value = strategy.resolve(&call_node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.string_values, vec!["sha256"]);
    }

    #[test]
    fn test_python_tuple_return() {
        let source = r#"
def get_config():
    return 10000, "sha256"

config = get_config()
"#;
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = CallStrategy::new();

        let call_node = find_call_by_name(tree.root_node(), "get_config", &ctx).unwrap();
        let value = strategy.resolve(&call_node, &ctx);

        assert!(value.is_resolved);
        assert!(value.int_values.contains(&10000));
        assert!(value.string_values.contains(&"sha256".to_string()));
    }

    #[test]
    fn test_python_if_else_returns() {
        let source = r#"
def get_key_size(aes256):
    if aes256:
        return 32
    return 16

x = get_key_size(True)
"#;
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = CallStrategy::new();

        let call_node = find_call_by_name(tree.root_node(), "get_key_size", &ctx).unwrap();
        let value = strategy.resolve(&call_node, &ctx);

        assert!(value.is_resolved);
        assert!(value.int_values.contains(&32));
        assert!(value.int_values.contains(&16));
    }

    // =========================================================================
    // Rust Tests
    // =========================================================================

    #[test]
    fn test_rust_simple_int_return() {
        let source = r#"
fn get_iterations() -> i32 {
    return 10000;
}

fn main() {
    let x = get_iterations();
}
"#;
        let tree = parse_rust(source);
        let ctx = create_rust_context(&tree, source.as_bytes());
        let strategy = CallStrategy::new();

        let call_node = find_call_by_name(tree.root_node(), "get_iterations", &ctx).unwrap();
        let value = strategy.resolve(&call_node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![10000]);
    }

    #[test]
    fn test_rust_simple_string_return() {
        let source = r#"
fn get_algorithm() -> &'static str {
    return "sha256";
}

fn main() {
    let x = get_algorithm();
}
"#;
        let tree = parse_rust(source);
        let ctx = create_rust_context(&tree, source.as_bytes());
        let strategy = CallStrategy::new();

        let call_node = find_call_by_name(tree.root_node(), "get_algorithm", &ctx).unwrap();
        let value = strategy.resolve(&call_node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.string_values, vec!["sha256"]);
    }

    #[test]
    fn test_rust_if_else_returns() {
        let source = r#"
fn get_key_size(aes256: bool) -> i32 {
    if aes256 {
        return 32;
    }
    return 16;
}

fn main() {
    let x = get_key_size(true);
}
"#;
        let tree = parse_rust(source);
        let ctx = create_rust_context(&tree, source.as_bytes());
        let strategy = CallStrategy::new();

        let call_node = find_call_by_name(tree.root_node(), "get_key_size", &ctx).unwrap();
        let value = strategy.resolve(&call_node, &ctx);

        assert!(value.is_resolved);
        assert!(value.int_values.contains(&32));
        assert!(value.int_values.contains(&16));
    }

    // =========================================================================
    // JavaScript Tests
    // =========================================================================

    #[test]
    fn test_js_simple_int_return() {
        let source = r#"
function getIterations() {
    return 10000;
}

const x = getIterations();
"#;
        let tree = parse_javascript(source);
        let ctx = create_js_context(&tree, source.as_bytes());
        let strategy = CallStrategy::new();

        let call_node = find_call_by_name(tree.root_node(), "getIterations", &ctx).unwrap();
        let value = strategy.resolve(&call_node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![10000]);
    }

    #[test]
    fn test_js_simple_string_return() {
        let source = r#"
function getAlgorithm() {
    return "sha256";
}

const x = getAlgorithm();
"#;
        let tree = parse_javascript(source);
        let ctx = create_js_context(&tree, source.as_bytes());
        let strategy = CallStrategy::new();

        let call_node = find_call_by_name(tree.root_node(), "getAlgorithm", &ctx).unwrap();
        let value = strategy.resolve(&call_node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.string_values, vec!["sha256"]);
    }

    #[test]
    fn test_js_if_else_returns() {
        let source = r#"
function getKeySize(aes256) {
    if (aes256) {
        return 32;
    }
    return 16;
}

const x = getKeySize(true);
"#;
        let tree = parse_javascript(source);
        let ctx = create_js_context(&tree, source.as_bytes());
        let strategy = CallStrategy::new();

        let call_node = find_call_by_name(tree.root_node(), "getKeySize", &ctx).unwrap();
        let value = strategy.resolve(&call_node, &ctx);

        assert!(value.is_resolved);
        assert!(value.int_values.contains(&32));
        assert!(value.int_values.contains(&16));
    }

    // =========================================================================
    // Crypto-Relevant Tests
    // =========================================================================

    #[test]
    fn test_go_pbkdf2_iterations_function() {
        let source = r#"
package main

func getIterations() int {
    return 100000
}

func main() {
    iter := getIterations()
    _ = iter
}"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = CallStrategy::new();

        let call_node = find_call_by_name(tree.root_node(), "getIterations", &ctx).unwrap();
        let value = strategy.resolve(&call_node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![100000]);
    }

    #[test]
    fn test_go_aes_key_sizes() {
        let source = r#"
package main

func getAESKeySize(bits int) int {
    switch bits {
    case 128:
        return 16
    case 192:
        return 24
    case 256:
        return 32
    }
    return 32
}

func main() {
    size := getAESKeySize(256)
    _ = size
}"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = CallStrategy::new();

        let call_node = find_call_by_name(tree.root_node(), "getAESKeySize", &ctx).unwrap();
        let value = strategy.resolve(&call_node, &ctx);

        assert!(value.is_resolved);
        assert!(value.int_values.contains(&16));
        assert!(value.int_values.contains(&24));
        assert!(value.int_values.contains(&32));
    }

    // =========================================================================
    // Partial Resolution Tests
    // =========================================================================

    #[test]
    fn test_go_unresolvable_return() {
        let source = r#"
package main

func getIterations(multiplier int) int {
    return multiplier * 1000
}

func main() {
    x := getIterations(10)
}"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = CallStrategy::new();

        let call_node = find_call_by_name(tree.root_node(), "getIterations", &ctx).unwrap();
        let value = strategy.resolve(&call_node, &ctx);

        assert!(!value.is_resolved);
        assert!(value.expression.contains("multiplier"));
    }
}
