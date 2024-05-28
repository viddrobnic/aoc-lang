use crate::token::{Token, TokenKind};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Precedence {
    Lowest,
    Assign,
    Or,
    And,
    Equals,
    LessGreater,
    Sum,
    Product,
    Prefix,
    CallIndex,
}

impl From<&TokenKind> for Precedence {
    fn from(value: &TokenKind) -> Self {
        match value {
            TokenKind::Assign => Self::Assign,
            TokenKind::Or => Self::Or,
            TokenKind::And => Self::And,
            TokenKind::Eq | TokenKind::Neq => Self::Equals,
            TokenKind::Le | TokenKind::Leq | TokenKind::Ge | TokenKind::Geq => Self::LessGreater,
            TokenKind::Plus | TokenKind::Minus => Self::Sum,
            TokenKind::Mult | TokenKind::Div | TokenKind::Modulo => Self::Product,
            TokenKind::LBracket | TokenKind::Dot | TokenKind::LSquare => Self::CallIndex,
            _ => Self::Lowest,
        }
    }
}

impl From<&Token> for Precedence {
    fn from(value: &Token) -> Self {
        (&value.kind).into()
    }
}
