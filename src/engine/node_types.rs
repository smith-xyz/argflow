use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Go,
    Python,
    Rust,
    JavaScript,
    TypeScript,
    C,
    Cpp,
    Java,
}

impl Language {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "go" => Some(Self::Go),
            "python" | "py" => Some(Self::Python),
            "rust" | "rs" => Some(Self::Rust),
            "javascript" | "js" => Some(Self::JavaScript),
            "typescript" | "ts" => Some(Self::TypeScript),
            "c" => Some(Self::C),
            "cpp" | "c++" => Some(Self::Cpp),
            "java" => Some(Self::Java),
            _ => None,
        }
    }

    pub fn tree_sitter_name(&self) -> &'static str {
        match self {
            Self::Go => "go",
            Self::Python => "python",
            Self::Rust => "rust",
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript",
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::Java => "java",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeCategory {
    IntegerLiteral,
    FloatLiteral,
    StringLiteral,
    BooleanLiteral,
    NilLiteral,
    Identifier,
    BinaryExpression,
    UnaryExpression,
    CallExpression,
    SelectorExpression,
    IndexExpression,
    ArrayLiteral,
    StructLiteral,
    FunctionDeclaration,
    VariableDeclaration,
    ConstantDeclaration,
    Assignment,
    Block,
    IfStatement,
    SwitchStatement,
    ReturnStatement,
}

pub struct NodeTypes {
    language: Language,
}

impl NodeTypes {
    pub fn new(language: Language) -> Self {
        Self { language }
    }

    pub fn from_language_str(lang: &str) -> Option<Self> {
        Language::parse(lang).map(Self::new)
    }

    pub fn language(&self) -> Language {
        self.language
    }

    pub fn is_category(&self, kind: &str, category: NodeCategory) -> bool {
        self.get_node_types(category).contains(kind)
    }

    pub fn get_node_types(&self, category: NodeCategory) -> HashSet<&'static str> {
        match category {
            NodeCategory::IntegerLiteral => self.integer_literal_types(),
            NodeCategory::FloatLiteral => self.float_literal_types(),
            NodeCategory::StringLiteral => self.string_literal_types(),
            NodeCategory::BooleanLiteral => self.boolean_literal_types(),
            NodeCategory::NilLiteral => self.nil_literal_types(),
            NodeCategory::Identifier => self.identifier_types(),
            NodeCategory::BinaryExpression => self.binary_expression_types(),
            NodeCategory::UnaryExpression => self.unary_expression_types(),
            NodeCategory::CallExpression => self.call_expression_types(),
            NodeCategory::SelectorExpression => self.selector_expression_types(),
            NodeCategory::IndexExpression => self.index_expression_types(),
            NodeCategory::ArrayLiteral => self.array_literal_types(),
            NodeCategory::StructLiteral => self.struct_literal_types(),
            NodeCategory::FunctionDeclaration => self.function_declaration_types(),
            NodeCategory::VariableDeclaration => self.variable_declaration_types(),
            NodeCategory::ConstantDeclaration => self.constant_declaration_types(),
            NodeCategory::Assignment => self.assignment_types(),
            NodeCategory::Block => self.block_types(),
            NodeCategory::IfStatement => self.if_statement_types(),
            NodeCategory::SwitchStatement => self.switch_statement_types(),
            NodeCategory::ReturnStatement => self.return_statement_types(),
        }
    }

    fn integer_literal_types(&self) -> HashSet<&'static str> {
        match self.language {
            Language::Go => ["int_literal"].into_iter().collect(),
            Language::Python => ["integer"].into_iter().collect(),
            Language::Rust => ["integer_literal"].into_iter().collect(),
            Language::JavaScript | Language::TypeScript => ["number"].into_iter().collect(),
            Language::C | Language::Cpp => ["number_literal"].into_iter().collect(),
            Language::Java => [
                "decimal_integer_literal",
                "hex_integer_literal",
                "octal_integer_literal",
                "binary_integer_literal",
            ]
            .into_iter()
            .collect(),
        }
    }

    fn float_literal_types(&self) -> HashSet<&'static str> {
        match self.language {
            Language::Go => ["float_literal"].into_iter().collect(),
            Language::Python => ["float"].into_iter().collect(),
            Language::Rust => ["float_literal"].into_iter().collect(),
            Language::JavaScript | Language::TypeScript => ["number"].into_iter().collect(),
            Language::C | Language::Cpp => ["number_literal"].into_iter().collect(),
            Language::Java => [
                "decimal_floating_point_literal",
                "hex_floating_point_literal",
            ]
            .into_iter()
            .collect(),
        }
    }

    fn string_literal_types(&self) -> HashSet<&'static str> {
        match self.language {
            Language::Go => ["interpreted_string_literal", "raw_string_literal"]
                .into_iter()
                .collect(),
            Language::Python => ["string"].into_iter().collect(),
            Language::Rust => ["string_literal", "raw_string_literal", "char_literal"]
                .into_iter()
                .collect(),
            Language::JavaScript | Language::TypeScript => {
                ["string", "template_string"].into_iter().collect()
            }
            Language::C | Language::Cpp => ["string_literal", "char_literal"].into_iter().collect(),
            Language::Java => ["string_literal", "character_literal"]
                .into_iter()
                .collect(),
        }
    }

    fn boolean_literal_types(&self) -> HashSet<&'static str> {
        match self.language {
            Language::Go => ["true", "false"].into_iter().collect(),
            Language::Python => ["true", "false", "True", "False"].into_iter().collect(),
            Language::Rust => ["boolean_literal"].into_iter().collect(),
            Language::JavaScript | Language::TypeScript => ["true", "false"].into_iter().collect(),
            Language::C | Language::Cpp => ["true", "false"].into_iter().collect(),
            Language::Java => ["true", "false"].into_iter().collect(),
        }
    }

    fn nil_literal_types(&self) -> HashSet<&'static str> {
        match self.language {
            Language::Go => ["nil"].into_iter().collect(),
            Language::Python => ["none", "None"].into_iter().collect(),
            Language::Rust => HashSet::new(), // Rust uses Option::None, not a literal
            Language::JavaScript | Language::TypeScript => {
                ["null", "undefined"].into_iter().collect()
            }
            Language::C | Language::Cpp => ["null", "nullptr"].into_iter().collect(),
            Language::Java => ["null_literal"].into_iter().collect(),
        }
    }

    fn identifier_types(&self) -> HashSet<&'static str> {
        match self.language {
            Language::Go => ["identifier"].into_iter().collect(),
            Language::Python => ["identifier"].into_iter().collect(),
            Language::Rust => ["identifier"].into_iter().collect(),
            Language::JavaScript | Language::TypeScript => ["identifier"].into_iter().collect(),
            Language::C | Language::Cpp => ["identifier"].into_iter().collect(),
            Language::Java => ["identifier"].into_iter().collect(),
        }
    }

    fn binary_expression_types(&self) -> HashSet<&'static str> {
        match self.language {
            Language::Go => ["binary_expression"].into_iter().collect(),
            Language::Python => ["binary_operator", "comparison_operator", "boolean_operator"]
                .into_iter()
                .collect(),
            Language::Rust => ["binary_expression"].into_iter().collect(),
            Language::JavaScript | Language::TypeScript => {
                ["binary_expression"].into_iter().collect()
            }
            Language::C | Language::Cpp => ["binary_expression"].into_iter().collect(),
            Language::Java => ["binary_expression"].into_iter().collect(),
        }
    }

    fn unary_expression_types(&self) -> HashSet<&'static str> {
        match self.language {
            Language::Go => ["unary_expression"].into_iter().collect(),
            Language::Python => ["unary_operator", "not_operator"].into_iter().collect(),
            Language::Rust => [
                "unary_expression",
                "reference_expression",
                "dereference_expression",
            ]
            .into_iter()
            .collect(),
            Language::JavaScript | Language::TypeScript => {
                ["unary_expression"].into_iter().collect()
            }
            Language::C | Language::Cpp => ["unary_expression"].into_iter().collect(),
            Language::Java => ["unary_expression"].into_iter().collect(),
        }
    }

    fn call_expression_types(&self) -> HashSet<&'static str> {
        match self.language {
            Language::Go => ["call_expression"].into_iter().collect(),
            Language::Python => ["call"].into_iter().collect(),
            Language::Rust => ["call_expression"].into_iter().collect(),
            Language::JavaScript | Language::TypeScript => {
                ["call_expression"].into_iter().collect()
            }
            Language::C | Language::Cpp => ["call_expression"].into_iter().collect(),
            Language::Java => ["method_invocation"].into_iter().collect(),
        }
    }

    fn selector_expression_types(&self) -> HashSet<&'static str> {
        match self.language {
            Language::Go => ["selector_expression"].into_iter().collect(),
            Language::Python => ["attribute"].into_iter().collect(),
            Language::Rust => ["field_expression"].into_iter().collect(),
            Language::JavaScript | Language::TypeScript => {
                ["member_expression"].into_iter().collect()
            }
            Language::C | Language::Cpp => ["field_expression"].into_iter().collect(),
            Language::Java => ["field_access"].into_iter().collect(),
        }
    }

    fn index_expression_types(&self) -> HashSet<&'static str> {
        match self.language {
            Language::Go => ["index_expression"].into_iter().collect(),
            Language::Python => ["subscript"].into_iter().collect(),
            Language::Rust => ["index_expression"].into_iter().collect(),
            Language::JavaScript | Language::TypeScript => {
                ["subscript_expression"].into_iter().collect()
            }
            Language::C | Language::Cpp => ["subscript_expression"].into_iter().collect(),
            Language::Java => ["array_access"].into_iter().collect(),
        }
    }

    fn array_literal_types(&self) -> HashSet<&'static str> {
        match self.language {
            Language::Go => ["composite_literal"].into_iter().collect(),
            Language::Python => ["list", "tuple"].into_iter().collect(),
            Language::Rust => ["array_expression", "tuple_expression"]
                .into_iter()
                .collect(),
            Language::JavaScript | Language::TypeScript => ["array"].into_iter().collect(),
            Language::C | Language::Cpp => ["initializer_list"].into_iter().collect(),
            Language::Java => ["array_initializer"].into_iter().collect(),
        }
    }

    fn struct_literal_types(&self) -> HashSet<&'static str> {
        match self.language {
            Language::Go => ["composite_literal"].into_iter().collect(),
            Language::Python => ["dictionary"].into_iter().collect(),
            Language::Rust => ["struct_expression"].into_iter().collect(),
            Language::JavaScript | Language::TypeScript => ["object"].into_iter().collect(),
            Language::C | Language::Cpp => ["initializer_list"].into_iter().collect(),
            Language::Java => ["object_creation_expression"].into_iter().collect(),
        }
    }

    fn function_declaration_types(&self) -> HashSet<&'static str> {
        match self.language {
            Language::Go => ["function_declaration", "method_declaration"]
                .into_iter()
                .collect(),
            Language::Python => ["function_definition"].into_iter().collect(),
            Language::Rust => ["function_item"].into_iter().collect(),
            Language::JavaScript | Language::TypeScript => [
                "function_declaration",
                "arrow_function",
                "method_definition",
            ]
            .into_iter()
            .collect(),
            Language::C | Language::Cpp => ["function_definition"].into_iter().collect(),
            Language::Java => ["method_declaration", "constructor_declaration"]
                .into_iter()
                .collect(),
        }
    }

    fn variable_declaration_types(&self) -> HashSet<&'static str> {
        match self.language {
            Language::Go => ["var_declaration", "short_var_declaration"]
                .into_iter()
                .collect(),
            Language::Python => ["assignment", "augmented_assignment"].into_iter().collect(),
            Language::Rust => ["let_declaration"].into_iter().collect(),
            Language::JavaScript | Language::TypeScript => {
                ["variable_declaration", "lexical_declaration"]
                    .into_iter()
                    .collect()
            }
            Language::C | Language::Cpp => ["declaration"].into_iter().collect(),
            Language::Java => ["local_variable_declaration", "field_declaration"]
                .into_iter()
                .collect(),
        }
    }

    fn constant_declaration_types(&self) -> HashSet<&'static str> {
        match self.language {
            Language::Go => ["const_declaration"].into_iter().collect(),
            Language::Python => HashSet::new(), // Python has no const keyword
            Language::Rust => ["const_item", "static_item"].into_iter().collect(),
            Language::JavaScript | Language::TypeScript => {
                ["lexical_declaration"].into_iter().collect()
            }
            Language::C | Language::Cpp => ["declaration"].into_iter().collect(),
            Language::Java => ["field_declaration"].into_iter().collect(),
        }
    }

    fn assignment_types(&self) -> HashSet<&'static str> {
        match self.language {
            Language::Go => ["assignment_statement"].into_iter().collect(),
            Language::Python => ["assignment", "augmented_assignment"].into_iter().collect(),
            Language::Rust => ["assignment_expression"].into_iter().collect(),
            Language::JavaScript | Language::TypeScript => {
                ["assignment_expression"].into_iter().collect()
            }
            Language::C | Language::Cpp => ["assignment_expression"].into_iter().collect(),
            Language::Java => ["assignment_expression"].into_iter().collect(),
        }
    }

    fn block_types(&self) -> HashSet<&'static str> {
        match self.language {
            Language::Go => ["block"].into_iter().collect(),
            Language::Python => ["block"].into_iter().collect(),
            Language::Rust => ["block"].into_iter().collect(),
            Language::JavaScript | Language::TypeScript => {
                ["statement_block"].into_iter().collect()
            }
            Language::C | Language::Cpp => ["compound_statement"].into_iter().collect(),
            Language::Java => ["block"].into_iter().collect(),
        }
    }

    fn if_statement_types(&self) -> HashSet<&'static str> {
        match self.language {
            Language::Go => ["if_statement"].into_iter().collect(),
            Language::Python => ["if_statement"].into_iter().collect(),
            Language::Rust => ["if_expression"].into_iter().collect(),
            Language::JavaScript | Language::TypeScript => ["if_statement"].into_iter().collect(),
            Language::C | Language::Cpp => ["if_statement"].into_iter().collect(),
            Language::Java => ["if_statement"].into_iter().collect(),
        }
    }

    fn switch_statement_types(&self) -> HashSet<&'static str> {
        match self.language {
            Language::Go => ["expression_switch_statement", "type_switch_statement"]
                .into_iter()
                .collect(),
            Language::Python => ["match_statement"].into_iter().collect(),
            Language::Rust => ["match_expression"].into_iter().collect(),
            Language::JavaScript | Language::TypeScript => {
                ["switch_statement"].into_iter().collect()
            }
            Language::C | Language::Cpp => ["switch_statement"].into_iter().collect(),
            Language::Java => ["switch_expression", "switch_statement"]
                .into_iter()
                .collect(),
        }
    }

    fn return_statement_types(&self) -> HashSet<&'static str> {
        match self.language {
            Language::Go => ["return_statement"].into_iter().collect(),
            Language::Python => ["return_statement"].into_iter().collect(),
            Language::Rust => ["return_expression"].into_iter().collect(),
            Language::JavaScript | Language::TypeScript => {
                ["return_statement"].into_iter().collect()
            }
            Language::C | Language::Cpp => ["return_statement"].into_iter().collect(),
            Language::Java => ["return_statement"].into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_parse() {
        assert_eq!(Language::parse("go"), Some(Language::Go));
        assert_eq!(Language::parse("Go"), Some(Language::Go));
        assert_eq!(Language::parse("python"), Some(Language::Python));
        assert_eq!(Language::parse("py"), Some(Language::Python));
        assert_eq!(Language::parse("rust"), Some(Language::Rust));
        assert_eq!(Language::parse("rs"), Some(Language::Rust));
        assert_eq!(Language::parse("javascript"), Some(Language::JavaScript));
        assert_eq!(Language::parse("js"), Some(Language::JavaScript));
        assert_eq!(Language::parse("unknown"), None);
    }

    #[test]
    fn test_is_integer_literal() {
        let go = NodeTypes::new(Language::Go);
        assert!(go.is_category("int_literal", NodeCategory::IntegerLiteral));
        assert!(!go.is_category("float_literal", NodeCategory::IntegerLiteral));

        let python = NodeTypes::new(Language::Python);
        assert!(python.is_category("integer", NodeCategory::IntegerLiteral));
        assert!(!python.is_category("int_literal", NodeCategory::IntegerLiteral));

        let rust = NodeTypes::new(Language::Rust);
        assert!(rust.is_category("integer_literal", NodeCategory::IntegerLiteral));

        let js = NodeTypes::new(Language::JavaScript);
        assert!(js.is_category("number", NodeCategory::IntegerLiteral));
    }

    #[test]
    fn test_is_string_literal() {
        let go = NodeTypes::new(Language::Go);
        assert!(go.is_category("interpreted_string_literal", NodeCategory::StringLiteral));
        assert!(go.is_category("raw_string_literal", NodeCategory::StringLiteral));

        let python = NodeTypes::new(Language::Python);
        assert!(python.is_category("string", NodeCategory::StringLiteral));

        let js = NodeTypes::new(Language::JavaScript);
        assert!(js.is_category("string", NodeCategory::StringLiteral));
        assert!(js.is_category("template_string", NodeCategory::StringLiteral));
    }

    #[test]
    fn test_is_boolean_literal() {
        let go = NodeTypes::new(Language::Go);
        assert!(go.is_category("true", NodeCategory::BooleanLiteral));
        assert!(go.is_category("false", NodeCategory::BooleanLiteral));
    }

    #[test]
    fn test_is_nil_literal() {
        let go = NodeTypes::new(Language::Go);
        assert!(go.is_category("nil", NodeCategory::NilLiteral));

        let js = NodeTypes::new(Language::JavaScript);
        assert!(js.is_category("null", NodeCategory::NilLiteral));
        assert!(js.is_category("undefined", NodeCategory::NilLiteral));

        let python = NodeTypes::new(Language::Python);
        assert!(python.is_category("None", NodeCategory::NilLiteral));
    }

    #[test]
    fn test_from_language_str() {
        let node_types = NodeTypes::from_language_str("go").unwrap();
        assert_eq!(node_types.language(), Language::Go);

        let node_types = NodeTypes::from_language_str("python").unwrap();
        assert_eq!(node_types.language(), Language::Python);

        assert!(NodeTypes::from_language_str("unknown").is_none());
    }

    #[test]
    fn test_call_expression_types() {
        let go = NodeTypes::new(Language::Go);
        assert!(go.is_category("call_expression", NodeCategory::CallExpression));

        let python = NodeTypes::new(Language::Python);
        assert!(python.is_category("call", NodeCategory::CallExpression));

        let java = NodeTypes::new(Language::Java);
        assert!(java.is_category("method_invocation", NodeCategory::CallExpression));
    }
}
