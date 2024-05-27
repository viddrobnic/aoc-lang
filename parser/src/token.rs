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
