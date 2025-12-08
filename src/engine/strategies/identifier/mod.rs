use crate::engine::{Context, Language, NodeCategory, Resolver, Strategy, UnresolvedSource, Value};
use tree_sitter::Node;

mod languages;

pub struct IdentifierStrategy {
    resolver: Option<Resolver>,
}

impl Default for IdentifierStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl IdentifierStrategy {
    pub fn new() -> Self {
        Self { resolver: None }
    }

    pub fn with_resolver(resolver: Resolver) -> Self {
        Self {
            resolver: Some(resolver),
        }
    }

    fn find_enclosing_function<'a>(&self, node: Node<'a>, ctx: &Context<'a>) -> Option<Node<'a>> {
        let mut current = node;
        loop {
            if ctx.is_node_category(current.kind(), NodeCategory::FunctionDeclaration) {
                return Some(current);
            }
            current = current.parent()?;
        }
    }

    fn find_declaration_in_scope<'a>(
        &self,
        name: &str,
        scope_node: Node<'a>,
        use_position: usize,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        let lang = ctx.node_types()?.language();

        // All languages: try to get function body first, then search within it
        // Tree-sitter uses "body" field for most languages
        let search_node = scope_node.child_by_field_name("body").unwrap_or(scope_node);

        match lang {
            Language::Go => {
                languages::go_find_declaration(self, name, search_node, use_position, ctx)
            }
            Language::Python => self.find_python_declaration(name, search_node, use_position, ctx),
            Language::Rust => self.find_rust_declaration(name, search_node, use_position, ctx),
            Language::JavaScript | Language::TypeScript => {
                self.find_js_declaration(name, search_node, use_position, ctx)
            }
            Language::C | Language::Cpp => {
                self.find_c_declaration(name, search_node, use_position, ctx)
            }
            Language::Java => self.find_java_declaration(name, search_node, use_position, ctx),
        }
    }

    fn find_python_declaration<'a>(
        &self,
        name: &str,
        scope_node: Node<'a>,
        use_position: usize,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        let mut cursor = scope_node.walk();
        self.search_python_declarations(&mut cursor, scope_node, name, use_position, ctx)
    }

    fn search_python_declarations<'a>(
        &self,
        cursor: &mut tree_sitter::TreeCursor<'a>,
        node: Node<'a>,
        name: &str,
        use_position: usize,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        let mut result: Option<Node<'a>> = None;

        for child in node.children(cursor) {
            if child.start_byte() >= use_position {
                continue;
            }

            match child.kind() {
                "assignment" | "augmented_assignment" => {
                    if let Some(value) = self.extract_python_assignment(child, name, ctx) {
                        result = Some(value);
                    }
                }
                "expression_statement" => {
                    if let Some(assign) = child.named_child(0) {
                        if assign.kind() == "assignment" || assign.kind() == "augmented_assignment"
                        {
                            if let Some(value) = self.extract_python_assignment(assign, name, ctx) {
                                result = Some(value);
                            }
                        }
                    }
                }
                "block" => {
                    let mut inner_cursor = child.walk();
                    if let Some(found) = self.search_python_declarations(
                        &mut inner_cursor,
                        child,
                        name,
                        use_position,
                        ctx,
                    ) {
                        result = Some(found);
                    }
                }
                _ => {}
            }
        }
        result
    }

    fn extract_python_assignment<'a>(
        &self,
        node: Node<'a>,
        name: &str,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        let left = node.child_by_field_name("left")?;
        let right = node.child_by_field_name("right")?;

        if left.kind() == "identifier" && ctx.get_node_text(&left) == name {
            return Some(right);
        }

        if left.kind() == "pattern_list" || left.kind() == "tuple_pattern" {
            let mut cursor = left.walk();
            let names: Vec<_> = left
                .children(&mut cursor)
                .filter(|c| c.is_named())
                .collect();

            if right.kind() == "tuple" || right.kind() == "list" {
                let mut value_cursor = right.walk();
                let values: Vec<_> = right
                    .children(&mut value_cursor)
                    .filter(|c| c.is_named())
                    .collect();

                for (i, name_node) in names.iter().enumerate() {
                    if ctx.get_node_text(name_node) == name {
                        return values.get(i).copied();
                    }
                }
            }
        }

        None
    }

    fn find_rust_declaration<'a>(
        &self,
        name: &str,
        scope_node: Node<'a>,
        use_position: usize,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        let mut cursor = scope_node.walk();
        self.search_rust_declarations(&mut cursor, scope_node, name, use_position, ctx)
    }

    fn search_rust_declarations<'a>(
        &self,
        cursor: &mut tree_sitter::TreeCursor<'a>,
        node: Node<'a>,
        name: &str,
        use_position: usize,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        for child in node.children(cursor) {
            if child.start_byte() >= use_position {
                continue;
            }

            match child.kind() {
                "let_declaration" => {
                    if let Some(value) = self.extract_rust_let(child, name, ctx) {
                        return Some(value);
                    }
                }
                "const_item" | "static_item" => {
                    if let Some(value) = self.extract_rust_const(child, name, ctx) {
                        return Some(value);
                    }
                }
                "block" => {
                    let mut inner_cursor = child.walk();
                    if let Some(found) = self.search_rust_declarations(
                        &mut inner_cursor,
                        child,
                        name,
                        use_position,
                        ctx,
                    ) {
                        return Some(found);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn extract_rust_let<'a>(
        &self,
        node: Node<'a>,
        name: &str,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        let pattern = node.child_by_field_name("pattern")?;
        let value = node.child_by_field_name("value")?;

        if pattern.kind() == "identifier" && ctx.get_node_text(&pattern) == name {
            return Some(value);
        }

        if pattern.kind() == "tuple_pattern" {
            let mut cursor = pattern.walk();
            let names: Vec<_> = pattern
                .children(&mut cursor)
                .filter(|c| c.kind() == "identifier")
                .collect();

            if value.kind() == "tuple_expression" {
                let mut value_cursor = value.walk();
                let values: Vec<_> = value
                    .children(&mut value_cursor)
                    .filter(|c| c.is_named())
                    .collect();

                for (i, name_node) in names.iter().enumerate() {
                    if ctx.get_node_text(name_node) == name {
                        return values.get(i).copied();
                    }
                }
            }
        }

        None
    }

    fn extract_rust_const<'a>(
        &self,
        node: Node<'a>,
        name: &str,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        let name_node = node.child_by_field_name("name")?;
        if ctx.get_node_text(&name_node) == name {
            return node.child_by_field_name("value");
        }
        None
    }

    fn find_js_declaration<'a>(
        &self,
        name: &str,
        scope_node: Node<'a>,
        use_position: usize,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        let mut cursor = scope_node.walk();
        self.search_js_declarations(&mut cursor, scope_node, name, use_position, ctx)
    }

    fn search_js_declarations<'a>(
        &self,
        cursor: &mut tree_sitter::TreeCursor<'a>,
        node: Node<'a>,
        name: &str,
        use_position: usize,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        for child in node.children(cursor) {
            if child.start_byte() >= use_position {
                continue;
            }

            match child.kind() {
                "variable_declaration" | "lexical_declaration" => {
                    if let Some(value) = self.extract_js_var_decl(child, name, ctx) {
                        return Some(value);
                    }
                }
                "statement_block" => {
                    let mut inner_cursor = child.walk();
                    if let Some(found) = self.search_js_declarations(
                        &mut inner_cursor,
                        child,
                        name,
                        use_position,
                        ctx,
                    ) {
                        return Some(found);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn extract_js_var_decl<'a>(
        &self,
        node: Node<'a>,
        name: &str,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "variable_declarator" {
                let name_node = child.child_by_field_name("name")?;
                if ctx.get_node_text(&name_node) == name {
                    return child.child_by_field_name("value");
                }
            }
        }
        None
    }

    fn find_c_declaration<'a>(
        &self,
        name: &str,
        scope_node: Node<'a>,
        use_position: usize,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        let mut cursor = scope_node.walk();
        self.search_c_declarations(&mut cursor, scope_node, name, use_position, ctx)
    }

    fn search_c_declarations<'a>(
        &self,
        cursor: &mut tree_sitter::TreeCursor<'a>,
        node: Node<'a>,
        name: &str,
        use_position: usize,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        for child in node.children(cursor) {
            if child.start_byte() >= use_position {
                continue;
            }

            if child.kind() == "declaration" {
                if let Some(value) = self.extract_c_declaration(child, name, ctx) {
                    return Some(value);
                }
            }

            if child.kind() == "compound_statement" {
                let mut inner_cursor = child.walk();
                if let Some(found) =
                    self.search_c_declarations(&mut inner_cursor, child, name, use_position, ctx)
                {
                    return Some(found);
                }
            }
        }
        None
    }

    fn extract_c_declaration<'a>(
        &self,
        node: Node<'a>,
        name: &str,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        let declarator = node.child_by_field_name("declarator")?;

        if declarator.kind() == "init_declarator" {
            let decl_name = declarator.child_by_field_name("declarator")?;
            if ctx.get_node_text(&decl_name) == name {
                return declarator.child_by_field_name("value");
            }
        }

        None
    }

    fn find_java_declaration<'a>(
        &self,
        name: &str,
        scope_node: Node<'a>,
        use_position: usize,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        let mut cursor = scope_node.walk();
        self.search_java_declarations(&mut cursor, scope_node, name, use_position, ctx)
    }

    fn search_java_declarations<'a>(
        &self,
        cursor: &mut tree_sitter::TreeCursor<'a>,
        node: Node<'a>,
        name: &str,
        use_position: usize,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        for child in node.children(cursor) {
            if child.start_byte() >= use_position {
                continue;
            }

            if child.kind() == "local_variable_declaration" {
                if let Some(value) = self.extract_java_var_decl(child, name, ctx) {
                    return Some(value);
                }
            }

            if child.kind() == "block" {
                let mut inner_cursor = child.walk();
                if let Some(found) =
                    self.search_java_declarations(&mut inner_cursor, child, name, use_position, ctx)
                {
                    return Some(found);
                }
            }
        }
        None
    }

    fn extract_java_var_decl<'a>(
        &self,
        node: Node<'a>,
        name: &str,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        let declarator = node.child_by_field_name("declarator")?;

        if declarator.kind() == "variable_declarator" {
            let name_node = declarator.child_by_field_name("name")?;
            if ctx.get_node_text(&name_node) == name {
                return declarator.child_by_field_name("value");
            }
        }

        None
    }

    fn find_file_level_constant<'a>(
        &self,
        name: &str,
        root: Node<'a>,
        use_position: usize,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        let lang = ctx.node_types()?.language();

        match lang {
            Language::Go => {
                languages::go_find_file_level_const(self, name, root, use_position, ctx)
            }
            Language::Python => self.find_python_file_level_const(name, root, use_position, ctx),
            Language::Rust => self.find_rust_file_level_const(name, root, use_position, ctx),
            Language::JavaScript | Language::TypeScript => {
                self.find_js_file_level_const(name, root, use_position, ctx)
            }
            _ => None,
        }
    }

    fn find_python_file_level_const<'a>(
        &self,
        name: &str,
        root: Node<'a>,
        use_position: usize,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            if child.start_byte() >= use_position {
                continue;
            }

            if child.kind() == "expression_statement" {
                if let Some(assign) = child.named_child(0) {
                    if assign.kind() == "assignment" {
                        if let Some(value) = self.extract_python_assignment(assign, name, ctx) {
                            return Some(value);
                        }
                    }
                }
            }
        }
        None
    }

    fn find_rust_file_level_const<'a>(
        &self,
        name: &str,
        root: Node<'a>,
        use_position: usize,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            if child.start_byte() >= use_position {
                continue;
            }

            match child.kind() {
                "const_item" | "static_item" => {
                    if let Some(value) = self.extract_rust_const(child, name, ctx) {
                        return Some(value);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn find_js_file_level_const<'a>(
        &self,
        name: &str,
        root: Node<'a>,
        use_position: usize,
        ctx: &Context<'a>,
    ) -> Option<Node<'a>> {
        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            if child.start_byte() >= use_position {
                continue;
            }

            match child.kind() {
                "variable_declaration" | "lexical_declaration" => {
                    if let Some(value) = self.extract_js_var_decl(child, name, ctx) {
                        return Some(value);
                    }
                }
                "expression_statement" => {
                    if let Some(assign) = child.named_child(0) {
                        if assign.kind() == "assignment_expression" {
                            let left = assign.child_by_field_name("left")?;
                            if ctx.get_node_text(&left) == name {
                                return assign.child_by_field_name("right");
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn is_function_parameter<'a>(
        &self,
        name: &str,
        function_node: Node<'a>,
        ctx: &Context<'a>,
    ) -> bool {
        let lang = ctx.node_types().map(|nt| nt.language());

        let params_field = match lang {
            Some(Language::Go) => "parameters",
            Some(Language::Python) => "parameters",
            Some(Language::Rust) => "parameters",
            Some(Language::JavaScript | Language::TypeScript) => "parameters",
            Some(Language::Java) => "parameters",
            _ => return false,
        };

        if let Some(params) = function_node.child_by_field_name(params_field) {
            return self.check_param_list_for_name(params, name, ctx);
        }

        false
    }

    fn check_param_list_for_name<'a>(
        &self,
        params: Node<'a>,
        name: &str,
        ctx: &Context<'a>,
    ) -> bool {
        let mut cursor = params.walk();
        for child in params.children(&mut cursor) {
            if self.extract_param_name(&child, ctx).as_deref() == Some(name) {
                return true;
            }
        }
        false
    }

    fn extract_param_name<'a>(&self, param: &Node<'a>, ctx: &Context<'a>) -> Option<String> {
        match param.kind() {
            "identifier" => Some(ctx.get_node_text(param)),
            "parameter_declaration" => {
                let name_node = param.child_by_field_name("name")?;
                Some(ctx.get_node_text(&name_node))
            }
            "typed_parameter" | "default_parameter" | "typed_default_parameter" => param
                .child_by_field_name("name")
                .map(|n| ctx.get_node_text(&n)),
            _ => {
                if let Some(name_field) = param.child_by_field_name("name") {
                    return Some(ctx.get_node_text(&name_field));
                }
                param.named_child(0).map(|n| ctx.get_node_text(&n))
            }
        }
    }

    fn resolve_value_node<'a>(&self, node: Node<'a>, ctx: &Context<'a>) -> Value {
        // Use resolver if available for full strategy chain
        if let Some(ref resolver) = self.resolver {
            return resolver.resolve(&node, ctx);
        }

        // Create a new resolver for full strategy chain resolution
        // This enables proper chaining through composite/array strategies
        let resolver = Resolver::new();
        resolver.resolve(&node, ctx)
    }
}

impl Strategy for IdentifierStrategy {
    fn name(&self) -> &'static str {
        "identifier"
    }

    fn can_handle<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> bool {
        ctx.is_node_category(node.kind(), NodeCategory::Identifier)
    }

    fn resolve<'a>(&self, node: &Node<'a>, ctx: &Context<'a>) -> Value {
        let name = ctx.get_node_text(node);
        let use_position = node.start_byte();

        if name == "true" || name == "false" || name == "nil" || name == "null" || name == "None" {
            return Value::resolved_string(name);
        }

        if let Some(function_node) = self.find_enclosing_function(*node, ctx) {
            if self.is_function_parameter(&name, function_node, ctx) {
                return Value::unextractable(UnresolvedSource::FunctionParameter);
            }

            if let Some(value_node) =
                self.find_declaration_in_scope(&name, function_node, use_position, ctx)
            {
                return self.resolve_value_node(value_node, ctx);
            }
        }

        let root = ctx.tree().root_node();
        if let Some(value_node) = self.find_file_level_constant(&name, root, use_position, ctx) {
            return self.resolve_value_node(value_node, ctx);
        }

        if let Some(value) = ctx.find_cross_file_constant(&name) {
            return value;
        }

        Value::unextractable(UnresolvedSource::IdentifierNotFound)
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

    fn find_identifier_by_name<'a>(
        node: tree_sitter::Node<'a>,
        name: &str,
        ctx: &Context<'a>,
    ) -> Option<tree_sitter::Node<'a>> {
        if node.kind() == "identifier" && ctx.get_node_text(&node) == name {
            return Some(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = find_identifier_by_name(child, name, ctx) {
                return Some(found);
            }
        }
        None
    }

    fn find_last_identifier_by_name<'a>(
        node: tree_sitter::Node<'a>,
        name: &str,
        ctx: &Context<'a>,
    ) -> Option<tree_sitter::Node<'a>> {
        let mut last: Option<tree_sitter::Node<'a>> = None;
        find_all_identifiers_by_name(node, name, ctx, &mut last);
        last
    }

    fn find_all_identifiers_by_name<'a>(
        node: tree_sitter::Node<'a>,
        name: &str,
        ctx: &Context<'a>,
        last: &mut Option<tree_sitter::Node<'a>>,
    ) {
        if node.kind() == "identifier" && ctx.get_node_text(&node) == name {
            *last = Some(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            find_all_identifiers_by_name(child, name, ctx, last);
        }
    }

    #[test]
    fn test_strategy_name() {
        let strategy = IdentifierStrategy::new();
        assert_eq!(strategy.name(), "identifier");
    }

    #[test]
    fn test_can_handle_identifier() {
        let source = "package main\nvar x = someVar";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node = find_identifier_by_name(tree.root_node(), "someVar", &ctx).unwrap();
        assert!(strategy.can_handle(&node, &ctx));
    }

    #[test]
    fn test_cannot_handle_literal() {
        let source = "package main\nconst x = 10000";
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let mut cursor = tree.root_node().walk();
        for child in tree.root_node().children(&mut cursor) {
            if child.kind() == "const_declaration" {
                let spec = child.named_child(0).unwrap();
                let value = spec.child_by_field_name("value").unwrap();
                let literal = value.named_child(0).unwrap();
                assert!(!strategy.can_handle(&literal, &ctx));
            }
        }
    }

    // =========================================================================
    // Go Tests
    // =========================================================================

    #[test]
    fn test_go_local_variable_short_decl() {
        let source = r#"
package main

func test() {
    iterations := 10000
    use(iterations)
}"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node = find_last_identifier_by_name(tree.root_node(), "iterations", &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![10000]);
    }

    #[test]
    fn test_go_local_variable_var_decl() {
        let source = r#"
package main

func test() {
    var keyLen = 32
    use(keyLen)
}"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node = find_last_identifier_by_name(tree.root_node(), "keyLen", &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![32]);
    }

    #[test]
    fn test_go_file_level_const() {
        let source = r#"
package main

const DefaultIterations = 100000

func test() {
    use(DefaultIterations)
}"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node =
            find_last_identifier_by_name(tree.root_node(), "DefaultIterations", &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![100000]);
    }

    #[test]
    fn test_go_file_level_var() {
        let source = r#"
package main

var globalKey = 256

func test() {
    use(globalKey)
}"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node = find_last_identifier_by_name(tree.root_node(), "globalKey", &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![256]);
    }

    #[test]
    fn test_go_function_parameter() {
        let source = r#"
package main

func derive(password []byte, iterations int) {
    use(iterations)
}"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node = find_last_identifier_by_name(tree.root_node(), "iterations", &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(!value.is_resolved);
        assert_eq!(value.source, "function_parameter");
    }

    #[test]
    fn test_go_string_constant() {
        let source = r#"
package main

const Algorithm = "sha256"

func test() {
    use(Algorithm)
}"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node = find_last_identifier_by_name(tree.root_node(), "Algorithm", &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.string_values, vec!["sha256"]);
    }

    #[test]
    fn test_go_identifier_not_found() {
        let source = r#"
package main

func test() {
    use(undeclaredVar)
}"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node = find_identifier_by_name(tree.root_node(), "undeclaredVar", &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(!value.is_resolved);
        assert_eq!(value.source, "identifier_not_found");
    }

    #[test]
    fn test_go_shadowing() {
        let source = r#"
package main

const iterations = 100000

func test() {
    iterations := 50000
    use(iterations)
}"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node = find_last_identifier_by_name(tree.root_node(), "iterations", &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![50000]);
    }

    #[test]
    fn test_go_assignment_update() {
        let source = r#"
package main

func test() {
    x := 100
    x = 200
    use(x)
}"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node = find_last_identifier_by_name(tree.root_node(), "x", &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![200]);
    }

    // =========================================================================
    // Python Tests
    // =========================================================================

    #[test]
    fn test_python_local_variable() {
        let source = r#"
def test():
    iterations = 10000
    use(iterations)
"#;
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node = find_last_identifier_by_name(tree.root_node(), "iterations", &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![10000]);
    }

    #[test]
    fn test_python_file_level_constant() {
        let source = r#"
DEFAULT_ITERATIONS = 100000

def test():
    use(DEFAULT_ITERATIONS)
"#;
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node =
            find_last_identifier_by_name(tree.root_node(), "DEFAULT_ITERATIONS", &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![100000]);
    }

    #[test]
    fn test_python_function_parameter() {
        let source = r#"
def derive(password, iterations):
    use(iterations)
"#;
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node = find_last_identifier_by_name(tree.root_node(), "iterations", &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(!value.is_resolved);
        assert_eq!(value.source, "function_parameter");
    }

    #[test]
    fn test_python_string_variable() {
        let source = r#"
def test():
    algorithm = "sha256"
    use(algorithm)
"#;
        let tree = parse_python(source);
        let ctx = create_python_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node = find_last_identifier_by_name(tree.root_node(), "algorithm", &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.string_values, vec!["sha256"]);
    }

    // =========================================================================
    // Rust Tests
    // =========================================================================

    #[test]
    fn test_rust_local_let() {
        let source = r#"
fn test() {
    let iterations = 10000;
    use_val(iterations);
}
"#;
        let tree = parse_rust(source);
        let ctx = create_rust_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node = find_last_identifier_by_name(tree.root_node(), "iterations", &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![10000]);
    }

    #[test]
    fn test_rust_file_level_const() {
        let source = r#"
const DEFAULT_ITERATIONS: u32 = 100000;

fn test() {
    use_val(DEFAULT_ITERATIONS);
}
"#;
        let tree = parse_rust(source);
        let ctx = create_rust_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node =
            find_last_identifier_by_name(tree.root_node(), "DEFAULT_ITERATIONS", &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![100000]);
    }

    #[test]
    fn test_rust_function_parameter() {
        let source = r#"
fn derive(password: &[u8], iterations: u32) {
    use_val(iterations);
}
"#;
        let tree = parse_rust(source);
        let ctx = create_rust_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node = find_last_identifier_by_name(tree.root_node(), "iterations", &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(!value.is_resolved);
        assert_eq!(value.source, "function_parameter");
    }

    // =========================================================================
    // JavaScript Tests
    // =========================================================================

    #[test]
    fn test_js_const_declaration() {
        let source = r#"
function test() {
    const iterations = 10000;
    use(iterations);
}
"#;
        let tree = parse_javascript(source);
        let ctx = create_js_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node = find_last_identifier_by_name(tree.root_node(), "iterations", &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![10000]);
    }

    #[test]
    fn test_js_let_declaration() {
        let source = r#"
function test() {
    let keyLen = 32;
    use(keyLen);
}
"#;
        let tree = parse_javascript(source);
        let ctx = create_js_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node = find_last_identifier_by_name(tree.root_node(), "keyLen", &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![32]);
    }

    #[test]
    fn test_js_file_level_const() {
        let source = r#"
const DEFAULT_ITERATIONS = 100000;

function test() {
    use(DEFAULT_ITERATIONS);
}
"#;
        let tree = parse_javascript(source);
        let ctx = create_js_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node =
            find_last_identifier_by_name(tree.root_node(), "DEFAULT_ITERATIONS", &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.int_values, vec![100000]);
    }

    #[test]
    fn test_js_function_parameter() {
        let source = r#"
function derive(password, iterations) {
    use(iterations);
}
"#;
        let tree = parse_javascript(source);
        let ctx = create_js_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node = find_last_identifier_by_name(tree.root_node(), "iterations", &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(!value.is_resolved);
        assert_eq!(value.source, "function_parameter");
    }

    // =========================================================================
    // Special Cases
    // =========================================================================

    #[test]
    fn test_go_boolean_variable() {
        let source = r#"
package main

func test() {
    enabled := true
    use(enabled)
}"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node = find_last_identifier_by_name(tree.root_node(), "enabled", &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.string_values, vec!["true"]);
    }

    #[test]
    fn test_go_nil_variable() {
        let source = r#"
package main

func test() {
    ptr := nil
    use(ptr)
}"#;
        let tree = parse_go(source);
        let ctx = create_go_context(&tree, source.as_bytes());
        let strategy = IdentifierStrategy::new();

        let node = find_last_identifier_by_name(tree.root_node(), "ptr", &ctx).unwrap();
        let value = strategy.resolve(&node, &ctx);

        assert!(value.is_resolved);
        assert_eq!(value.string_values, vec!["nil"]);
    }
}
