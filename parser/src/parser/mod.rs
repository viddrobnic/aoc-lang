use crate::{
    ast::{self, InfixOperatorKind, NodeKind, PrefixOperatorKind},
    error::{Error, ErrorKind, Result},
    lexer::Lexer,
    parser::validate::validate_assignee,
    position::{Position, Range},
    token::{Token, TokenKind},
};

use self::{precedence::Precedence, validate::validate_node_kind};

mod precedence;
mod validate;

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
            TokenKind::If => {
                let (if_node, end) = self.parse_if(range)?;
                (ast::NodeValue::If(if_node), end)
            }
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
            TokenKind::LSquare => self.parse_index(left, start_token.range)?,
            TokenKind::Dot => self.parse_dot_index(left, start_token.range)?,
            TokenKind::LBracket => todo!("parse function call"),
            TokenKind::Assign => self.parse_assign(left, start_token.range)?,

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

        validate::validate_array_literal(&items)?;
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

        validate::validate_hash_literal(&items)?;
        Ok((ast::NodeValue::HashLiteral(items), end))
    }

    fn parse_assign(
        &mut self,
        left: ast::Node,
        start_range: Range,
    ) -> Result<(ast::NodeValue, Position)> {
        let token = self.next_token(start_range)?;

        let right = self.parse_node(token, Precedence::Lowest)?;
        if right.kind() != NodeKind::Expression {
            return Err(Error {
                kind: ErrorKind::InvalidNodeKind {
                    expected: NodeKind::Expression,
                    got: right.kind(),
                },
                range: right.range,
            });
        }

        validate_assignee(&left)?;

        let end = right.range.end;
        Ok((
            ast::NodeValue::Assign {
                ident: Box::new(left),
                value: Box::new(right),
            },
            end,
        ))
    }

    // Parse index `left[index]`
    fn parse_index(
        &mut self,
        left: ast::Node,
        start_range: Range,
    ) -> Result<(ast::NodeValue, Position)> {
        let token = self.next_token(start_range)?;
        let index = self.parse_node(token, Precedence::Lowest)?;

        let end_token = self.next_token(Range {
            start: start_range.start,
            end: index.range.end,
        })?;
        if end_token.kind != TokenKind::RSquare {
            return Err(Error {
                kind: ErrorKind::InvalidTokenKind {
                    expected: TokenKind::RSquare,
                    got: end_token.kind,
                },
                range: end_token.range,
            });
        }

        validate_node_kind(&left, NodeKind::Expression)?;
        validate_node_kind(&index, NodeKind::Expression)?;

        Ok((
            ast::NodeValue::Index {
                left: Box::new(left),
                index: Box::new(index),
            },
            end_token.range.end,
        ))
    }

    // parse index `left.index` where `index` is ident
    fn parse_dot_index(
        &mut self,
        left: ast::Node,
        start_range: Range,
    ) -> Result<(ast::NodeValue, Position)> {
        let index = self.next_token(start_range)?;

        let TokenKind::Ident(index_ident) = index.kind else {
            return Err(Error {
                kind: ErrorKind::InvalidTokenKind {
                    expected: TokenKind::Ident("".to_string()),
                    got: index.kind,
                },
                range: index.range,
            });
        };

        validate_node_kind(&left, NodeKind::Expression)?;

        Ok((
            ast::NodeValue::Index {
                left: Box::new(left),
                index: Box::new(ast::Node {
                    value: ast::NodeValue::StringLiteral(index_ident),
                    range: index.range,
                }),
            },
            index.range.end,
        ))
    }

    fn parse_if(&mut self, start_range: Range) -> Result<(ast::IfNode, Position)> {
        // Read `(`
        let token = self.next_token(start_range)?;
        if token.kind != TokenKind::LBracket {
            return Err(Error {
                kind: ErrorKind::InvalidTokenKind {
                    expected: TokenKind::LBracket,
                    got: token.kind,
                },
                range: token.range,
            });
        }

        // Parse condition
        let cond_token = self.next_token(Range {
            start: start_range.start,
            end: token.range.end,
        })?;
        let condition = self.parse_node(cond_token, Precedence::Lowest)?;
        validate_node_kind(&condition, NodeKind::Expression)?;

        // Read `)`
        let token = self.next_token(Range {
            start: start_range.start,
            end: condition.range.end,
        })?;
        if token.kind != TokenKind::RBracket {
            return Err(Error {
                kind: ErrorKind::InvalidTokenKind {
                    expected: TokenKind::RBracket,
                    got: token.kind,
                },
                range: token.range,
            });
        }

        // Parse consequence
        let cons_token = self.next_token(Range {
            start: start_range.start,
            end: token.range.end,
        })?;
        let (consequence, cons_end) = self.parse_block(cons_token)?;

        // Construct the if node
        let mut if_node = ast::IfNode {
            condition: Box::new(condition),
            consequence,
            alternative: vec![],
        };

        // After consequence we can have eof, eol or else.
        peek_token!(self, else_token, return Ok((if_node, cons_end)));
        if else_token.kind == TokenKind::Eol {
            return Ok((if_node, cons_end));
        }

        if else_token.kind != TokenKind::Else {
            return Err(Error {
                kind: ErrorKind::InvalidTokenKind {
                    expected: TokenKind::Else,
                    got: else_token.kind.clone(),
                },
                range: else_token.range,
            });
        }

        // Read else token and discard it.
        let else_token_range = else_token.range;
        self.lexer.next();

        // Handle else and else if
        let token = self.next_token(else_token_range)?;
        if token.kind == TokenKind::If {
            let (alternative, alternative_end) = self.parse_if(token.range)?;

            if_node.alternative = vec![ast::Node {
                value: ast::NodeValue::If(alternative),
                range: Range {
                    start: token.range.start,
                    end: alternative_end,
                },
            }];
            Ok((if_node, alternative_end))
        } else {
            let (alternative, alternative_end) = self.parse_block(token)?;

            if_node.alternative = alternative;
            Ok((if_node, alternative_end))
        }
    }

    // Helper function that reads block { ... }.
    // It returns vector of nodes and end position, which is the end
    // position of `}`
    //
    // This function checks if the start token is `{`, so the caller doesn't have to do this.
    fn parse_block(&mut self, start_token: Token) -> Result<(Vec<ast::Node>, Position)> {
        // Start token should be `{`
        validate::validate_token_kind(&start_token, TokenKind::LCurly)?;

        let mut nodes = Vec::new();
        let mut end = start_token.range.end;
        loop {
            // Skip \n's
            self.skip_eol()?;

            // Check if next token is `}`. In this case we are done with the block
            let token = self.next_token(Range {
                start: start_token.range.start,
                end,
            })?;

            if token.kind == TokenKind::RCurly {
                return Ok((nodes, token.range.end));
            }

            // Parse next node
            let node = self.parse_node(token, Precedence::Lowest)?;
            end = node.range.end;
            nodes.push(node);

            // Token after ndoe should be one of:
            // - `}` => We are done with the block
            // - `\n` => We repeat the loop
            // - `// ...` => We repeat the loop
            // Otherwise, we return an error
            let token = self.next_token(Range {
                start: start_token.range.start,
                end,
            })?;

            if token.kind == TokenKind::RCurly {
                return Ok((nodes, token.range.end));
            }

            if !matches!(token.kind, TokenKind::Eol | TokenKind::Comment(_)) {
                return Err(Error {
                    kind: ErrorKind::InvalidTokenKind {
                        expected: TokenKind::RCurly,
                        got: token.kind,
                    },
                    range: token.range,
                });
            }
        }
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
