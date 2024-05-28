use std::fmt::Display;

use crate::position::Range;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Ident(String),
    Integer(i64),
    Float(f64),
    True,
    False,
    String(String),
    LSquare,   // [
    RSquare,   // ]
    LBracket,  // (
    RBracket,  // )
    LCurly,    // {
    RCurly,    // }
    Le,        // <
    Leq,       // <=
    Ge,        // >
    Geq,       // >=
    Eq,        // ==
    Neq,       // !=
    Plus,      // +
    Minus,     // -
    Mult,      // *
    Div,       // /
    Modulo,    // %
    And,       // &
    Or,        // |
    Bang,      // !
    Assign,    // =
    Colon,     // :
    Semicolon, // ;
    Comma,     // ,
    Dot,       // .
    If,
    Else,
    While,
    For,
    Break,
    Continue,
    Return,
    Fn,
    Use,
    Eol, // \n
    Comment(String),
}

impl TokenKind {
    pub fn from_ident(ident: &str) -> Option<Self> {
        let token = match ident {
            "true" => Self::True,
            "false" => Self::False,
            "if" => Self::If,
            "else" => Self::Else,
            "while" => Self::While,
            "for" => Self::For,
            "break" => Self::Break,
            "continue" => Self::Continue,
            "return" => Self::Return,
            "fn" => Self::Fn,
            "use" => Self::Use,
            _ => return None,
        };

        Some(token)
    }

    pub fn is_infix(&self) -> bool {
        matches!(
            self,
            Self::LSquare
                | Self::LBracket
                | Self::Le
                | Self::Leq
                | Self::Ge
                | Self::Geq
                | Self::Eq
                | Self::Neq
                | Self::Plus
                | Self::Minus
                | Self::Mult
                | Self::Div
                | Self::Modulo
                | Self::And
                | Self::Or
                | Self::Assign
                | Self::Dot
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub range: Range,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Ident(_) => write!(f, "IDENT"),
            TokenKind::Integer(_) => write!(f, "INTEGER"),
            TokenKind::Float(_) => write!(f, "FLOAT"),
            TokenKind::True => write!(f, "TRUE"),
            TokenKind::False => write!(f, "FALSE"),
            TokenKind::String(_) => write!(f, "STRING"),
            TokenKind::LSquare => write!(f, "LSQUARE"),
            TokenKind::RSquare => write!(f, "RSQUARE"),
            TokenKind::LBracket => write!(f, "LBRACKET"),
            TokenKind::RBracket => write!(f, "RBRACKET"),
            TokenKind::LCurly => write!(f, "LCURLY"),
            TokenKind::RCurly => write!(f, "RCURLY"),
            TokenKind::Le => write!(f, "LE"),
            TokenKind::Leq => write!(f, "LEQ"),
            TokenKind::Ge => write!(f, "GE"),
            TokenKind::Geq => write!(f, "GEQ"),
            TokenKind::Eq => write!(f, "EQ"),
            TokenKind::Neq => write!(f, "NEQ"),
            TokenKind::Plus => write!(f, "PLUS"),
            TokenKind::Minus => write!(f, "MINUS"),
            TokenKind::Mult => write!(f, "MULT"),
            TokenKind::Div => write!(f, "DIV"),
            TokenKind::Modulo => write!(f, "MODULO"),
            TokenKind::And => write!(f, "AND"),
            TokenKind::Or => write!(f, "OR"),
            TokenKind::Bang => write!(f, "BANG"),
            TokenKind::Assign => write!(f, "ASSIGN"),
            TokenKind::Colon => write!(f, "COLON"),
            TokenKind::Semicolon => write!(f, "SEMICOLON"),
            TokenKind::Comma => write!(f, "COMMA"),
            TokenKind::Dot => write!(f, "DOT"),
            TokenKind::If => write!(f, "IF"),
            TokenKind::Else => write!(f, "ELSE"),
            TokenKind::While => write!(f, "WHILE"),
            TokenKind::For => write!(f, "FOR"),
            TokenKind::Break => write!(f, "BREAK"),
            TokenKind::Continue => write!(f, "CONTINUE"),
            TokenKind::Return => write!(f, "RETURN"),
            TokenKind::Fn => write!(f, "FN"),
            TokenKind::Use => write!(f, "USE"),
            TokenKind::Eol => write!(f, "EOL"),
            TokenKind::Comment(_) => write!(f, "COMMENT"),
        }
    }
}
