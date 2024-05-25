use crate::{
    ast::{self, PrefixOperatorKind},
    error::{Error, ErrorKind, Result},
    lexer::Lexer,
    position::Position,
    token::{Token, TokenKind},
};

use self::precedence::Precedence;

mod precedence;

#[cfg(test)]
mod test;

pub fn parse(input: &str) -> Result<ast::Program> {
    let mut parser = Parser::new(Lexer::new(input));
    parser.parse_program()
}

struct Parser<'a> {
    lexer: std::iter::Peekable<Lexer<'a>>,
}

impl<'a> Parser<'a> {
    fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer: lexer.peekable(),
        }
    }
}

// Ugly macro solution to repeating code. First idea was to introduce
// a helper method `peek_token_or_err(&mut self) -> Result<Option<&Token>>`.
// The problem was, that the token extracted from return of `self.lexer.peek()`
// lives only for the duration of the method and can not be returned.
macro_rules! peek_token {
    // Params:
    // - self: pass parse &mut self to macro
    // - var: to which ident the extracted token reference is stored.
    // - eof: statement that is executed in case of eof.
    ($self:ident, $var:ident, $eof:stmt) => {
        let Some(peek) = $self.lexer.peek() else {
            // We reached eof.
            $eof
        };

        // Handle lexer errors
        let $var = match peek {
            Err(_) => {
                // We know that peek is Some(Err(_)), so it is safe to unwrap.
                return Err($self.lexer.next().unwrap().unwrap_err());
            }
            Ok(tkn) => tkn,
        };
    };
}

impl Parser<'_> {
    fn parse_program(&mut self) -> Result<ast::Program> {
        let mut statements = Vec::new();

        loop {
            self.skip_eol()?;

            // If there is no more tokens, we reached EOF.
            if self.lexer.peek().is_none() {
                break;
            }

            let stmt = self.parse_node(Precedence::Lowest)?;
            statements.push(stmt);

            peek_token!(self, token, break);
            if !matches!(token.kind, TokenKind::Eol | TokenKind::Comment(_)) {
                return Err(Error {
                    kind: ErrorKind::ExpectedEol,
                    position: token.position,
                });
            }
        }

        Ok(ast::Program { statements })
    }

    // Skips Token::Eol while they exist
    fn skip_eol(&mut self) -> Result<()> {
        loop {
            peek_token!(self, token, break);
            if token.kind == TokenKind::Eol {
                self.lexer.next();
            } else {
                break;
            }
        }

        Ok(())
    }

    fn parse_node(&mut self, precedence: Precedence) -> Result<ast::Node> {
        let mut left = self.parse_prefix()?;

        loop {
            peek_token!(self, token, break);

            // Handle eol
            if token.kind == TokenKind::Eol {
                break;
            }

            // Handle precedence
            if precedence >= token.into() {
                break;
            }

            if !token.kind.is_infix() {
                break;
            }

            left = self.parse_infix(left)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<ast::Node> {
        let Some(token) = self.lexer.next() else {
            return Err(Error {
                kind: ErrorKind::UnexpectedEof,
                position: Position::default(),
            });
        };
        let Token {
            kind: tkn_kind,
            position,
        } = token?;

        let node_value = match tkn_kind {
            TokenKind::Ident(ident) => ast::NodeValue::Identifier(ident),
            TokenKind::Integer(int) => ast::NodeValue::IntegerLiteral(int),
            TokenKind::Float(flt) => ast::NodeValue::FloatLiteral(flt),
            TokenKind::True => ast::NodeValue::BoolLiteral(true),
            TokenKind::False => ast::NodeValue::BoolLiteral(false),
            TokenKind::String(string) => ast::NodeValue::StringLiteral(string),
            TokenKind::Bang => self.parse_prefix_operator(PrefixOperatorKind::Not)?,
            TokenKind::Minus => self.parse_prefix_operator(PrefixOperatorKind::Negative)?,
            TokenKind::LBracket => todo!("parse grouped"),
            TokenKind::LSquare => todo!("parse array literal"),
            TokenKind::LCurly => todo!("parse hash map literal"),
            TokenKind::If => todo!("parse if statement"),
            TokenKind::While => todo!("parse while loop"),
            TokenKind::For => todo!("parse for loop"),
            TokenKind::Break => ast::NodeValue::Break,
            TokenKind::Continue => ast::NodeValue::Continue,
            TokenKind::Return => todo!("parse return statement"),
            TokenKind::Fn => todo!("parse function literal"),
            TokenKind::Use => todo!("parse use"),
            TokenKind::Comment(comment) => ast::NodeValue::Comment(comment),

            token => {
                return Err(Error {
                    kind: ErrorKind::InvalidExpression(token),
                    position,
                })
            }
        };

        Ok(ast::Node {
            value: node_value,
            position,
        })
    }

    fn parse_infix(&mut self, left: ast::Node) -> Result<ast::Node> {
        todo!()
    }

    fn parse_prefix_operator(&mut self, operator: PrefixOperatorKind) -> Result<ast::NodeValue> {
        let right = self.parse_node(Precedence::Prefix)?;
        Ok(ast::NodeValue::PrefixOperator {
            operator,
            right: Box::new(right),
        })
    }
}
