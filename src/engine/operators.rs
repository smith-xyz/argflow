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
        }
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
    fn test_binary_op_parse() {
        assert_eq!(BinaryOp::parse("+"), Some(BinaryOp::Add));
        assert_eq!(BinaryOp::parse("-"), Some(BinaryOp::Sub));
        assert_eq!(BinaryOp::parse("<<"), Some(BinaryOp::ShiftLeft));
        assert_eq!(BinaryOp::parse("unknown"), None);
    }

    #[test]
    fn test_binary_op_as_str() {
        assert_eq!(BinaryOp::Add.as_str(), "+");
        assert_eq!(BinaryOp::ShiftLeft.as_str(), "<<");
    }

    #[test]
    fn test_binary_op_evaluate() {
        assert_eq!(BinaryOp::Add.evaluate(100, 50), Some(150));
        assert_eq!(BinaryOp::Sub.evaluate(100, 30), Some(70));
        assert_eq!(BinaryOp::Mul.evaluate(10, 5), Some(50));
        assert_eq!(BinaryOp::Div.evaluate(100, 4), Some(25));
        assert_eq!(BinaryOp::Div.evaluate(100, 0), None);
        assert_eq!(BinaryOp::ShiftLeft.evaluate(1, 4), Some(16));
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
