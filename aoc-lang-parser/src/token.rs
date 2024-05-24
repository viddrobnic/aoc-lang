use crate::position::Position;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
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
    Semicolon, // ;
    Comma,     // ,
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
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub position: Position,
}
