#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    ShiftLeft,
    ShiftRight,
    BitAnd,
    BitOr,
    BitXor,
    Eq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    LogicalAnd,
    LogicalOr,
}

impl BinaryOp {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "+" => Some(Self::Add),
            "-" => Some(Self::Sub),
            "*" => Some(Self::Mul),
            "/" => Some(Self::Div),
            "%" => Some(Self::Mod),
            "<<" => Some(Self::ShiftLeft),
            ">>" => Some(Self::ShiftRight),
            "&" => Some(Self::BitAnd),
            "|" => Some(Self::BitOr),
            "^" => Some(Self::BitXor),
            "==" => Some(Self::Eq),
            "!=" | "<>" => Some(Self::NotEq),
            "<" => Some(Self::Less),
            "<=" => Some(Self::LessEq),
            ">" => Some(Self::Greater),
            ">=" => Some(Self::GreaterEq),
            "&&" | "and" => Some(Self::LogicalAnd),
            "||" | "or" => Some(Self::LogicalOr),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Mod => "%",
            Self::ShiftLeft => "<<",
            Self::ShiftRight => ">>",
            Self::BitAnd => "&",
            Self::BitOr => "|",
            Self::BitXor => "^",
            Self::Eq => "==",
            Self::NotEq => "!=",
            Self::Less => "<",
            Self::LessEq => "<=",
            Self::Greater => ">",
            Self::GreaterEq => ">=",
            Self::LogicalAnd => "&&",
            Self::LogicalOr => "||",
        }
    }

    pub fn is_arithmetic(&self) -> bool {
        matches!(
            self,
            Self::Add | Self::Sub | Self::Mul | Self::Div | Self::Mod
        )
    }

    pub fn is_bitwise(&self) -> bool {
        matches!(
            self,
            Self::ShiftLeft | Self::ShiftRight | Self::BitAnd | Self::BitOr | Self::BitXor
        )
    }

    pub fn is_comparison(&self) -> bool {
        matches!(
            self,
            Self::Eq | Self::NotEq | Self::Less | Self::LessEq | Self::Greater | Self::GreaterEq
        )
    }

    pub fn is_logical(&self) -> bool {
        matches!(self, Self::LogicalAnd | Self::LogicalOr)
    }

    pub fn evaluate(&self, left: i64, right: i64) -> Option<i64> {
        match self {
            Self::Add => Some(left.saturating_add(right)),
            Self::Sub => Some(left.saturating_sub(right)),
            Self::Mul => Some(left.saturating_mul(right)),
            Self::Div if right != 0 => Some(left / right),
            Self::Mod if right != 0 => Some(left % right),
            Self::ShiftLeft if (0..64).contains(&right) => Some(left << right),
            Self::ShiftRight if (0..64).contains(&right) => Some(left >> right),
            Self::BitAnd => Some(left & right),
            Self::BitOr => Some(left | right),
            Self::BitXor => Some(left ^ right),
            Self::Eq => Some(if left == right { 1 } else { 0 }),
            Self::NotEq => Some(if left != right { 1 } else { 0 }),
            Self::Less => Some(if left < right { 1 } else { 0 }),
            Self::LessEq => Some(if left <= right { 1 } else { 0 }),
            Self::Greater => Some(if left > right { 1 } else { 0 }),
            Self::GreaterEq => Some(if left >= right { 1 } else { 0 }),
            Self::LogicalAnd => Some(if left != 0 && right != 0 { 1 } else { 0 }),
            Self::LogicalOr => Some(if left != 0 || right != 0 { 1 } else { 0 }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Pos,
    BitNot,
    LogicalNot,
}

impl UnaryOp {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "-" => Some(Self::Neg),
            "+" => Some(Self::Pos),
            "^" | "~" => Some(Self::BitNot),
            "!" => Some(Self::LogicalNot),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Neg => "-",
            Self::Pos => "+",
            Self::BitNot => "~",
            Self::LogicalNot => "!",
        }
    }

    pub fn evaluate(&self, operand: i64) -> Option<i64> {
        match self {
            Self::Neg => Some(-operand),
            Self::Pos => Some(operand),
            Self::BitNot => Some(!operand),
            Self::LogicalNot => Some(if operand == 0 { 1 } else { 0 }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_op_parse_arithmetic() {
        assert_eq!(BinaryOp::parse("+"), Some(BinaryOp::Add));
        assert_eq!(BinaryOp::parse("-"), Some(BinaryOp::Sub));
        assert_eq!(BinaryOp::parse("*"), Some(BinaryOp::Mul));
        assert_eq!(BinaryOp::parse("/"), Some(BinaryOp::Div));
        assert_eq!(BinaryOp::parse("%"), Some(BinaryOp::Mod));
    }

    #[test]
    fn test_binary_op_parse_bitwise() {
        assert_eq!(BinaryOp::parse("<<"), Some(BinaryOp::ShiftLeft));
        assert_eq!(BinaryOp::parse(">>"), Some(BinaryOp::ShiftRight));
        assert_eq!(BinaryOp::parse("&"), Some(BinaryOp::BitAnd));
        assert_eq!(BinaryOp::parse("|"), Some(BinaryOp::BitOr));
        assert_eq!(BinaryOp::parse("^"), Some(BinaryOp::BitXor));
    }

    #[test]
    fn test_binary_op_parse_comparison() {
        assert_eq!(BinaryOp::parse("=="), Some(BinaryOp::Eq));
        assert_eq!(BinaryOp::parse("!="), Some(BinaryOp::NotEq));
        assert_eq!(BinaryOp::parse("<>"), Some(BinaryOp::NotEq));
        assert_eq!(BinaryOp::parse("<"), Some(BinaryOp::Less));
        assert_eq!(BinaryOp::parse("<="), Some(BinaryOp::LessEq));
        assert_eq!(BinaryOp::parse(">"), Some(BinaryOp::Greater));
        assert_eq!(BinaryOp::parse(">="), Some(BinaryOp::GreaterEq));
    }

    #[test]
    fn test_binary_op_parse_logical() {
        assert_eq!(BinaryOp::parse("&&"), Some(BinaryOp::LogicalAnd));
        assert_eq!(BinaryOp::parse("||"), Some(BinaryOp::LogicalOr));
        assert_eq!(BinaryOp::parse("and"), Some(BinaryOp::LogicalAnd));
        assert_eq!(BinaryOp::parse("or"), Some(BinaryOp::LogicalOr));
        assert_eq!(BinaryOp::parse("unknown"), None);
    }

    #[test]
    fn test_binary_op_as_str() {
        assert_eq!(BinaryOp::Add.as_str(), "+");
        assert_eq!(BinaryOp::ShiftLeft.as_str(), "<<");
        assert_eq!(BinaryOp::Eq.as_str(), "==");
        assert_eq!(BinaryOp::LogicalAnd.as_str(), "&&");
    }

    #[test]
    fn test_binary_op_category() {
        assert!(BinaryOp::Add.is_arithmetic());
        assert!(BinaryOp::Mul.is_arithmetic());
        assert!(!BinaryOp::Add.is_bitwise());

        assert!(BinaryOp::ShiftLeft.is_bitwise());
        assert!(BinaryOp::BitAnd.is_bitwise());
        assert!(!BinaryOp::ShiftLeft.is_arithmetic());

        assert!(BinaryOp::Eq.is_comparison());
        assert!(BinaryOp::Less.is_comparison());
        assert!(!BinaryOp::Eq.is_logical());

        assert!(BinaryOp::LogicalAnd.is_logical());
        assert!(BinaryOp::LogicalOr.is_logical());
        assert!(!BinaryOp::LogicalAnd.is_comparison());
    }

    #[test]
    fn test_binary_op_evaluate_arithmetic() {
        assert_eq!(BinaryOp::Add.evaluate(100, 50), Some(150));
        assert_eq!(BinaryOp::Sub.evaluate(100, 30), Some(70));
        assert_eq!(BinaryOp::Mul.evaluate(10, 5), Some(50));
        assert_eq!(BinaryOp::Div.evaluate(100, 4), Some(25));
        assert_eq!(BinaryOp::Div.evaluate(100, 0), None);
        assert_eq!(BinaryOp::Mod.evaluate(100, 30), Some(10));
        assert_eq!(BinaryOp::Mod.evaluate(100, 0), None);
    }

    #[test]
    fn test_binary_op_evaluate_bitwise() {
        assert_eq!(BinaryOp::ShiftLeft.evaluate(1, 4), Some(16));
        assert_eq!(BinaryOp::ShiftRight.evaluate(16, 2), Some(4));
        assert_eq!(BinaryOp::BitAnd.evaluate(0b1111, 0b1010), Some(0b1010));
        assert_eq!(BinaryOp::BitOr.evaluate(0b1100, 0b0011), Some(0b1111));
        assert_eq!(BinaryOp::BitXor.evaluate(0b1111, 0b1010), Some(0b0101));
    }

    #[test]
    fn test_binary_op_evaluate_comparison() {
        assert_eq!(BinaryOp::Eq.evaluate(10, 10), Some(1));
        assert_eq!(BinaryOp::Eq.evaluate(10, 20), Some(0));
        assert_eq!(BinaryOp::NotEq.evaluate(10, 20), Some(1));
        assert_eq!(BinaryOp::NotEq.evaluate(10, 10), Some(0));
        assert_eq!(BinaryOp::Less.evaluate(5, 10), Some(1));
        assert_eq!(BinaryOp::Less.evaluate(10, 5), Some(0));
        assert_eq!(BinaryOp::LessEq.evaluate(10, 10), Some(1));
        assert_eq!(BinaryOp::Greater.evaluate(10, 5), Some(1));
        assert_eq!(BinaryOp::GreaterEq.evaluate(10, 10), Some(1));
    }

    #[test]
    fn test_binary_op_evaluate_logical() {
        assert_eq!(BinaryOp::LogicalAnd.evaluate(1, 1), Some(1));
        assert_eq!(BinaryOp::LogicalAnd.evaluate(1, 0), Some(0));
        assert_eq!(BinaryOp::LogicalAnd.evaluate(0, 1), Some(0));
        assert_eq!(BinaryOp::LogicalOr.evaluate(1, 0), Some(1));
        assert_eq!(BinaryOp::LogicalOr.evaluate(0, 0), Some(0));
    }

    #[test]
    fn test_unary_op_parse() {
        assert_eq!(UnaryOp::parse("-"), Some(UnaryOp::Neg));
        assert_eq!(UnaryOp::parse("~"), Some(UnaryOp::BitNot));
        assert_eq!(UnaryOp::parse("^"), Some(UnaryOp::BitNot));
    }

    #[test]
    fn test_unary_op_evaluate() {
        assert_eq!(UnaryOp::Neg.evaluate(42), Some(-42));
        assert_eq!(UnaryOp::BitNot.evaluate(0), Some(-1));
        assert_eq!(UnaryOp::LogicalNot.evaluate(0), Some(1));
        assert_eq!(UnaryOp::LogicalNot.evaluate(5), Some(0));
    }
}
