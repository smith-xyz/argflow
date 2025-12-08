// Resolution strategies for extracting cryptographic parameter values.
//
// Strategies are applied in order of complexity. Each strategy handles
// a specific type of AST node and attempts to resolve it to a concrete value.
//
// Implementation order (uncomment as completed):
// 1. Literal    - Direct values: 10000, "sha256", true
// 2. Unary      - Unary operations: -x, !flag, &x
// 3. Binary     - Binary operations: BASE + 10000, keyLen * 8
// 4. Identifier - Variable lookup: iterations -> find declaration
// 5. Call       - Function returns: getIterations() -> trace returns
// 6. Selector   - Field/method access: cfg.Iterations, pkg.Constant
// 7. Index      - Array/map access: arr[2], map["key"]
// 8. Composite  - Literal structures: []int{16, 32}, Config{Iter: 10000}

// ============================================================================
// IMPLEMENTED STRATEGIES
// ============================================================================

pub mod literal;
pub use literal::LiteralStrategy;

// ============================================================================
// TODO: STRATEGIES TO IMPLEMENT
// Uncomment each module and use statement as they are completed.
// ============================================================================

// pub mod unary;
// pub use unary::UnaryStrategy;

// pub mod binary;
// pub use binary::BinaryStrategy;

// pub mod identifier;
// pub use identifier::IdentifierStrategy;

// pub mod call;
// pub use call::CallStrategy;

// pub mod selector;
// pub use selector::SelectorStrategy;

// pub mod index;
// pub use index::IndexStrategy;

// pub mod composite;
// pub use composite::CompositeStrategy;
