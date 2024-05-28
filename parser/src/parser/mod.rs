use crate::{
    ast::{self, InfixOperatorKind, NodeKind, PrefixOperatorKind},
    error::{Error, ErrorKind, Result},
    lexer::Lexer,
    position::{Position, Range},
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

            let Some(token) = self.lexer.next() else {
                // If there is no more tokens, we reached EOF.
                break;
            };

            let stmt = self.parse_node(token?, Precedence::Lowest)?;
            statements.push(stmt);

            peek_token!(self, token, break);
            if !matches!(token.kind, TokenKind::Eol | TokenKind::Comment(_)) {
                return Err(Error {
                    kind: ErrorKind::ExpectedEol,
                    range: token.range,
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

    fn next_token(&mut self, eof_range: Range) -> Result<Token> {
        self.lexer.next().ok_or(Error {
            kind: ErrorKind::UnexpectedEof,
            range: eof_range,
        })?
    }

    // Parse node with recursive descent. Takes first token as argument,
    // which makes the caller responsible for handling Eof. This way we can
    // append nice range information to the Eof errors. Similar approach is taken
    // for helper methods.
    fn parse_node(&mut self, start_token: Token, precedence: Precedence) -> Result<ast::Node> {
        let mut left = self.parse_prefix(start_token)?;

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

            // We can safely unwrap, because we know it's Some(Ok(_)),
            // since peek_token! already handles those cases.
            let token = self.lexer.next().unwrap().unwrap();
            left = self.parse_infix(token, left)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self, start_token: Token) -> Result<ast::Node> {
        let Token {
            kind: tkn_kind,
            range,
        } = start_token;

        let (node_value, end) = match tkn_kind {
            TokenKind::Ident(ident) => (ast::NodeValue::Identifier(ident), range.end),
            TokenKind::Integer(int) => (ast::NodeValue::IntegerLiteral(int), range.end),
            TokenKind::Float(flt) => (ast::NodeValue::FloatLiteral(flt), range.end),
            TokenKind::True => (ast::NodeValue::BoolLiteral(true), range.end),
            TokenKind::False => (ast::NodeValue::BoolLiteral(false), range.end),
            TokenKind::String(string) => (ast::NodeValue::StringLiteral(string), range.end),
            TokenKind::Bang | TokenKind::Minus => self.parse_prefix_operator(Token {
                kind: tkn_kind,
                range,
            })?,
            TokenKind::LBracket => self.parse_grouped(range)?,
            TokenKind::LSquare => self.parse_array_literal(range)?,
            TokenKind::LCurly => self.parse_hash_map_literal(range)?,
            TokenKind::If => todo!("parse if statement"),
            TokenKind::While => todo!("parse while loop"),
            TokenKind::For => todo!("parse for loop"),
            TokenKind::Break => (ast::NodeValue::Break, range.end),
            TokenKind::Continue => (ast::NodeValue::Continue, range.end),
            TokenKind::Return => todo!("parse return statement"),
            TokenKind::Fn => todo!("parse function literal"),
            TokenKind::Use => {
                let token = self.next_token(range)?;
                let TokenKind::String(val) = token.kind else {
                    return Err(Error {
                        kind: ErrorKind::InvalidTokenKind {
                            expected: TokenKind::String(String::new()),
                            got: token.kind,
                        },
                        range: token.range,
                    });
                };

                (ast::NodeValue::Use(val), token.range.end)
            }
            TokenKind::Comment(comment) => (ast::NodeValue::Comment(comment), range.end),

            token => {
                return Err(Error {
                    kind: ErrorKind::InvalidExpression(token),
                    range,
                })
            }
        };

        Ok(ast::Node {
            value: node_value,
            range: Range {
                start: range.start,
                end,
            },
        })
    }

    fn parse_infix(&mut self, start_token: Token, left: ast::Node) -> Result<ast::Node> {
        let start = left.range.start;

        let (node_value, end) = match &start_token.kind {
            TokenKind::Le
            | TokenKind::Leq
            | TokenKind::Ge
            | TokenKind::Geq
            | TokenKind::Eq
            | TokenKind::Neq
            | TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Mult
            | TokenKind::Div
            | TokenKind::Modulo
            | TokenKind::And
            | TokenKind::Or => self.parse_infix_operation(start_token, left)?,
            TokenKind::LSquare | TokenKind::Dot => todo!("parse index"),
            TokenKind::LBracket => todo!("parse function call"),
            TokenKind::Assign => todo!("parse assign"),

            _ => return Ok(left),
        };

        Ok(ast::Node {
            value: node_value,
            range: Range { start, end },
        })
    }

    fn parse_prefix_operator(&mut self, start_token: Token) -> Result<(ast::NodeValue, Position)> {
        let right_token = self.next_token(start_token.range)?;
        let right = self.parse_node(right_token, Precedence::Prefix)?;

        if right.kind() != NodeKind::Expression {
            return Err(Error {
                kind: ErrorKind::InvalidNodeKind {
                    expected: NodeKind::Expression,
                    got: right.kind(),
                },
                range: right.range,
            });
        }

        let end = right.range.end;

        Ok((
            ast::NodeValue::PrefixOperator {
                operator: token_to_prefix_operator(&start_token.kind),
                right: Box::new(right),
            },
            end,
        ))
    }

    fn parse_infix_operation(
        &mut self,
        start_token: Token,
        left: ast::Node,
    ) -> Result<(ast::NodeValue, Position)> {
        let precedence = Precedence::from(&start_token);
        let operator = token_to_infix_operator(&start_token.kind);

        let right_token = self.next_token(Range {
            start: left.range.start,
            end: start_token.range.end,
        })?;

        let right = self.parse_node(right_token, precedence)?;

        if left.kind() != NodeKind::Expression {
            return Err(Error {
                kind: ErrorKind::InvalidNodeKind {
                    expected: NodeKind::Expression,
                    got: left.kind(),
                },
                range: left.range,
            });
        }

        if right.kind() != NodeKind::Expression {
            return Err(Error {
                kind: ErrorKind::InvalidNodeKind {
                    expected: NodeKind::Expression,
                    got: right.kind(),
                },
                range: right.range,
            });
        }

        let end = right.range.end;

        Ok((
            ast::NodeValue::InfixOperator {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            },
            end,
        ))
    }

    fn parse_grouped(&mut self, start_range: Range) -> Result<(ast::NodeValue, Position)> {
        let token = self.next_token(start_range)?;
        let node = self.parse_node(token, Precedence::Lowest)?;
        if node.kind() != NodeKind::Expression {
            return Err(Error {
                kind: ErrorKind::InvalidNodeKind {
                    expected: NodeKind::Expression,
                    got: node.kind(),
                },
                range: node.range,
            });
        }

        let closing_token = self.next_token(Range {
            start: start_range.start,
            end: node.range.end,
        })?;

        if closing_token.kind != TokenKind::RBracket {
            return Err(Error {
                kind: ErrorKind::InvalidTokenKind {
                    expected: TokenKind::RBracket,
                    got: closing_token.kind,
                },
                range: closing_token.range,
            });
        }

        Ok((node.value, closing_token.range.end))
    }

    fn parse_array_literal(&mut self, start_range: Range) -> Result<(ast::NodeValue, Position)> {
        let (items, end) =
            self.parse_multiple(start_range, TokenKind::RSquare, |parser, token| {
                let item = parser.parse_node(token, Precedence::Lowest)?;
                let end = item.range.end;
                Ok((item, end))
            })?;

        for it in &items {
            if it.kind() != NodeKind::Expression {
                return Err(Error {
                    kind: ErrorKind::InvalidNodeKind {
                        expected: NodeKind::Statement,
                        got: it.kind(),
                    },
                    range: it.range,
                });
            }
        }

        Ok((ast::NodeValue::ArrayLiteral(items), end))
    }

    fn parse_hash_map_literal(&mut self, start_range: Range) -> Result<(ast::NodeValue, Position)> {
        let (items, end) =
            self.parse_multiple(start_range, TokenKind::RCurly, |parser, token| {
                let key = parser.parse_node(token, Precedence::Lowest)?;

                let token = parser.next_token(key.range)?;
                if token.kind != TokenKind::Colon {
                    return Err(Error {
                        kind: ErrorKind::InvalidTokenKind {
                            expected: TokenKind::Colon,
                            got: token.kind,
                        },
                        range: token.range,
                    });
                }

                let val_token = parser.next_token(Range {
                    start: key.range.start,
                    end: token.range.end,
                })?;

                let value = parser.parse_node(val_token, Precedence::Lowest)?;
                let end = value.range.end;

                Ok((ast::HashLiteralPair { key, value }, end))
            })?;

        for it in &items {
            if it.key.kind() != NodeKind::Expression {
                return Err(Error {
                    kind: ErrorKind::InvalidNodeKind {
                        expected: NodeKind::Expression,
                        got: it.key.kind(),
                    },
                    range: it.key.range,
                });
            }

            if it.value.kind() != NodeKind::Expression {
                return Err(Error {
                    kind: ErrorKind::InvalidNodeKind {
                        expected: NodeKind::Expression,
                        got: it.value.kind(),
                    },
                    range: it.value.range,
                });
            }
        }

        Ok((ast::NodeValue::HashLiteral(items), end))
    }

    // Helper function used for parsing arrays, hash maps, function arguments, function calls.
    fn parse_multiple<T, F>(
        &mut self,
        start_range: Range,
        end_token: TokenKind,
        parse_item: F,
    ) -> Result<(Vec<T>, Position)>
    where
        F: Fn(&mut Self, Token) -> Result<(T, Position)>,
    {
        let mut res = vec![];
        let mut end = start_range.end;

        let mut can_parse = true;
        loop {
            self.skip_eol()?;

            let token = self.next_token(Range {
                start: start_range.start,
                end,
            })?;

            if token.kind == end_token {
                return Ok((res, token.range.end));
            }

            if !can_parse {
                return Err(Error {
                    kind: ErrorKind::InvalidTokenKind {
                        expected: TokenKind::RSquare,
                        got: token.kind,
                    },
                    range: token.range,
                });
            }

            let (item, item_end) = parse_item(self, token)?;
            end = item_end;
            res.push(item);

            peek_token!(
                self,
                token_peek,
                return Err(Error {
                    kind: ErrorKind::UnexpectedEof,
                    range: Range {
                        start: start_range.start,
                        end,
                    }
                })
            );

            if token_peek.kind == TokenKind::Comma {
                self.lexer.next();
                can_parse = true;
            } else {
                can_parse = false;
            }
        }
    }
}

fn token_to_infix_operator(token: &TokenKind) -> InfixOperatorKind {
    match token {
        TokenKind::Le => InfixOperatorKind::Le,
        TokenKind::Leq => InfixOperatorKind::Leq,
        TokenKind::Ge => InfixOperatorKind::Ge,
        TokenKind::Geq => InfixOperatorKind::Geq,
        TokenKind::Eq => InfixOperatorKind::Eq,
        TokenKind::Neq => InfixOperatorKind::Neq,
        TokenKind::Plus => InfixOperatorKind::Add,
        TokenKind::Minus => InfixOperatorKind::Subtract,
        TokenKind::Mult => InfixOperatorKind::Multiply,
        TokenKind::Div => InfixOperatorKind::Divide,
        TokenKind::Modulo => InfixOperatorKind::Modulo,
        TokenKind::And => InfixOperatorKind::And,
        TokenKind::Or => InfixOperatorKind::Or,

        _ => panic!("token {token:?} is not infix oeprator"),
    }
}

fn token_to_prefix_operator(token: &TokenKind) -> PrefixOperatorKind {
    match token {
        TokenKind::Bang => PrefixOperatorKind::Not,
        TokenKind::Minus => PrefixOperatorKind::Negative,

        _ => panic!("token {token:?} is not prefix operator"),
    }
}
