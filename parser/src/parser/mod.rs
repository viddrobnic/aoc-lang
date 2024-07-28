use crate::{
    ast::{self, FunctionParamter, InfixOperatorKind, NodeKind, PrefixOperatorKind},
    error::{Error, ErrorKind, Result},
    lexer::Lexer,
    parser::{precedence::Precedence, validate::*},
    position::{Position, Range},
    token::{Token, TokenKind},
};

mod precedence;
mod validate;

#[cfg(test)]
mod test;

pub fn parse(input: &str) -> Result<ast::Program> {
    let parser = Parser::new(Lexer::new(input));
    parser.parse_program()
}

struct Parser<'a> {
    lexer: std::iter::Peekable<Lexer<'a>>,
    comments: Vec<ast::Comment>,
    end: Position,
}

impl<'a> Parser<'a> {
    fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer: lexer.peekable(),
            comments: vec![],
            end: Position::default(),
        }
    }
}

impl Parser<'_> {
    fn parse_program(mut self) -> Result<ast::Program> {
        let mut statements = Vec::new();

        loop {
            self.skip_eol()?;

            let Ok(token) = self.next_token() else {
                // If we reach eof, we don't raise an error.
                break;
            };

            let stmt = self.parse_node(token, Precedence::Lowest)?;
            statements.push(stmt);

            let is_eol = self.peek_token_is(|t| t.kind == TokenKind::Eol)?;
            match is_eol {
                None => break,
                Some(true) => (),
                Some(false) => {
                    let token = self.next_token()?;
                    return Err(Error {
                        kind: ErrorKind::ExpectedEol,
                        range: token.range,
                    });
                }
            }
        }

        Ok(ast::Program {
            statements,
            comments: self.comments,
        })
    }

    // Returns next token.
    //
    // - If an error occurs while lexing the token, the error is returned.
    // - If eof is reached, UnexpectedEof error is returned.
    // - If token is comment, it is added to self.comments and the token
    //   after that is emitted.
    fn next_token(&mut self) -> Result<Token> {
        match self.lexer.next() {
            None => Err(Error {
                kind: ErrorKind::UnexpectedEof,
                range: Range {
                    start: self.end,
                    end: Position {
                        line: self.end.line + 1,
                        character: 0,
                    },
                },
            }),
            Some(Err(err)) => Err(err),
            Some(Ok(token)) => {
                self.end = token.range.end;

                if let TokenKind::Comment(comment) = token.kind {
                    self.comments.push(ast::Comment {
                        comment,
                        range: token.range,
                    });
                    self.next_token()
                } else {
                    Ok(token)
                }
            }
        }
    }

    // Checks if peek token satisfies check.
    //
    // - If an error occurs while lexing next token, the error is returnd.
    // - If eof is reached, Ok(None) is returned
    // - If comment is reached, it is skipped
    // - Otherwise Ok(SOme(check(token))) is returned
    fn peek_token_is<F>(&mut self, check: F) -> Result<Option<bool>>
    where
        F: Fn(&Token) -> bool,
    {
        match self.lexer.peek() {
            None => Ok(None),
            Some(Err(_)) => Err(self.next_token().unwrap_err()),
            Some(Ok(token)) => {
                if matches!(token.kind, TokenKind::Comment(_)) {
                    // Safe to unwrap, because we know the token is Some(Ok(_))
                    let token = self.lexer.next().unwrap().unwrap();
                    let TokenKind::Comment(comment) = token.kind else {
                        unreachable!()
                    };
                    self.comments.push(ast::Comment {
                        comment,
                        range: token.range,
                    });

                    self.peek_token_is(check)
                } else {
                    Ok(Some(check(token)))
                }
            }
        }
    }

    // Skips Token::Eol while they exist
    fn skip_eol(&mut self) -> Result<()> {
        loop {
            match self.peek_token_is(|token| token.kind == TokenKind::Eol)? {
                None | Some(false) => return Ok(()),
                Some(true) => self.next_token()?,
            };
        }
    }

    // Parse node with recursive descent. Takes first token as argument,
    // which makes the caller responsible for handling Eof. This way we can
    // append nice range information to the Eof errors. Similar approach is taken
    // for helper methods.
    fn parse_node(&mut self, start_token: Token, precedence: Precedence) -> Result<ast::Node> {
        let mut left = self.parse_prefix(start_token)?;

        loop {
            let should_break = self.peek_token_is(|t| {
                t.kind == TokenKind::Eol || precedence >= t.into() || !t.kind.is_infix()
            })?;
            match should_break {
                None | Some(true) => break,
                Some(false) => (),
            }

            // since peek_token! already handles those cases.
            let token = self.next_token()?;
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
            TokenKind::Null => (ast::NodeValue::Null, range.end),
            TokenKind::Ident(ident) => (ast::NodeValue::Identifier(ident), range.end),
            TokenKind::Integer(int) => (ast::NodeValue::IntegerLiteral(int), range.end),
            TokenKind::Float(flt) => (ast::NodeValue::FloatLiteral(flt), range.end),
            TokenKind::Char(ch) => (ast::NodeValue::CharLiteral(ch), range.end),
            TokenKind::True => (ast::NodeValue::BoolLiteral(true), range.end),
            TokenKind::False => (ast::NodeValue::BoolLiteral(false), range.end),
            TokenKind::String(string) => (ast::NodeValue::StringLiteral(string), range.end),
            TokenKind::Bang | TokenKind::Minus => self.parse_prefix_operator(Token {
                kind: tkn_kind,
                range,
            })?,
            TokenKind::LBracket => self.parse_grouped()?,
            TokenKind::LSquare => self.parse_array_literal()?,
            TokenKind::LCurly => self.parse_hash_map_literal()?,
            TokenKind::If => {
                let (if_node, end) = self.parse_if()?;
                (ast::NodeValue::If(if_node), end)
            }
            TokenKind::While => self.parse_while()?,
            TokenKind::For => self.parse_for()?,
            TokenKind::Break => (ast::NodeValue::Break, range.end),
            TokenKind::Continue => (ast::NodeValue::Continue, range.end),
            TokenKind::Return => {
                let token = self.next_token()?;
                let node = self.parse_node(token, Precedence::Lowest)?;
                validate_node_kind(&node, NodeKind::Expression)?;

                let end = node.range.end;
                (ast::NodeValue::Return(Box::new(node)), end)
            }
            TokenKind::Fn => self.parse_fn_literal()?,
            TokenKind::Use => {
                let token = self.next_token()?;
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
            TokenKind::LSquare => self.parse_index(left)?,
            TokenKind::Dot => self.parse_dot_index(left)?,
            TokenKind::LBracket => self.parse_fn_call(left)?,
            TokenKind::Assign => self.parse_assign(left)?,

            _ => return Ok(left),
        };

        Ok(ast::Node {
            value: node_value,
            range: Range { start, end },
        })
    }

    fn parse_prefix_operator(&mut self, start_token: Token) -> Result<(ast::NodeValue, Position)> {
        let right_token = self.next_token()?;
        let right = self.parse_node(right_token, Precedence::Prefix)?;

        validate_node_kind(&right, NodeKind::Expression)?;

        let end = right.range.end;

        Ok((
            ast::NodeValue::PrefixOperator(ast::PrefixOperator {
                operator: token_to_prefix_operator(&start_token.kind),
                right: Box::new(right),
            }),
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

        let right_token = self.next_token()?;
        let right = self.parse_node(right_token, precedence)?;

        validate_node_kind(&left, NodeKind::Expression)?;
        validate_node_kind(&right, NodeKind::Expression)?;

        let end = right.range.end;

        Ok((
            ast::NodeValue::InfixOperator(ast::InfixOperator {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            }),
            end,
        ))
    }

    fn parse_grouped(&mut self) -> Result<(ast::NodeValue, Position)> {
        let token = self.next_token()?;
        let node = self.parse_node(token, Precedence::Lowest)?;
        validate_node_kind(&node, NodeKind::Expression)?;

        let closing_token = self.next_token()?;
        validate_token_kind(&closing_token, TokenKind::RBracket)?;

        Ok((node.value, closing_token.range.end))
    }

    fn parse_array_literal(&mut self) -> Result<(ast::NodeValue, Position)> {
        let (items, end) =
            self.parse_multiple(TokenKind::RSquare, TokenKind::Comma, |parser, token| {
                let item = parser.parse_node(token, Precedence::Lowest)?;
                Ok(item)
            })?;

        validate_array_literal(&items)?;
        Ok((ast::NodeValue::ArrayLiteral(items), end))
    }

    fn parse_hash_map_literal(&mut self) -> Result<(ast::NodeValue, Position)> {
        let (items, end) =
            self.parse_multiple(TokenKind::RCurly, TokenKind::Comma, |parser, token| {
                let key = parser.parse_node(token, Precedence::Lowest)?;

                let token = parser.next_token()?;
                validate_token_kind(&token, TokenKind::Colon)?;

                let val_token = parser.next_token()?;
                let value = parser.parse_node(val_token, Precedence::Lowest)?;

                Ok(ast::HashLiteralPair { key, value })
            })?;

        validate_hash_literal(&items)?;
        Ok((ast::NodeValue::HashLiteral(items), end))
    }

    fn parse_assign(&mut self, left: ast::Node) -> Result<(ast::NodeValue, Position)> {
        let token = self.next_token()?;

        let mut right = self.parse_node(token, Precedence::Lowest)?;
        validate_node_kind(&right, NodeKind::Expression)?;
        validate_assignee(&left)?;

        if let (
            ast::NodeValue::Identifier(ident),
            ast::NodeValue::FunctionLiteral(ast::FunctionLiteral { name, .. }),
        ) = (&left.value, &mut right.value)
        {
            *name = Some(ident.clone());
        }

        let end = right.range.end;
        Ok((
            ast::NodeValue::Assign(ast::Assign {
                ident: Box::new(left),
                value: Box::new(right),
            }),
            end,
        ))
    }

    // Parse index `left[index]`
    fn parse_index(&mut self, left: ast::Node) -> Result<(ast::NodeValue, Position)> {
        let token = self.next_token()?;
        let index = self.parse_node(token, Precedence::Lowest)?;

        let end_token = self.next_token()?;
        validate_token_kind(&end_token, TokenKind::RSquare)?;

        validate_node_kind(&left, NodeKind::Expression)?;
        validate_node_kind(&index, NodeKind::Expression)?;

        Ok((
            ast::NodeValue::Index(ast::Index {
                left: Box::new(left),
                index: Box::new(index),
            }),
            end_token.range.end,
        ))
    }

    // parse index `left.index` where `index` is ident
    fn parse_dot_index(&mut self, left: ast::Node) -> Result<(ast::NodeValue, Position)> {
        let index = self.next_token()?;

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
            ast::NodeValue::Index(ast::Index {
                left: Box::new(left),
                index: Box::new(ast::Node {
                    value: ast::NodeValue::StringLiteral(index_ident),
                    range: index.range,
                }),
            }),
            index.range.end,
        ))
    }

    fn parse_if(&mut self) -> Result<(ast::IfNode, Position)> {
        // Read `(`
        let token = self.next_token()?;
        validate_token_kind(&token, TokenKind::LBracket)?;

        // Parse condition
        let cond_token = self.next_token()?;
        let condition = self.parse_node(cond_token, Precedence::Lowest)?;
        validate_node_kind(&condition, NodeKind::Expression)?;

        // Read `)`
        let token = self.next_token()?;
        validate_token_kind(&token, TokenKind::RBracket)?;

        // Parse consequence
        let cons_token = self.next_token()?;
        let (consequence, cons_end) = self.parse_block(cons_token)?;

        // Construct the if node
        let mut if_node = ast::IfNode {
            condition: Box::new(condition),
            consequence,
            alternative: None,
        };

        // After consequence we can have eof, eol or else.
        match self.peek_token_is(|t| t.kind == TokenKind::Eol)? {
            None | Some(true) => return Ok((if_node, cons_end)),
            Some(false) => (),
        }

        // Read else token and discard it.
        let else_token = self.next_token()?;
        validate_token_kind(&else_token, TokenKind::Else)?;

        // Handle else and else if
        let token = self.next_token()?;
        if token.kind == TokenKind::If {
            let (alternative, alternative_end) = self.parse_if()?;

            let range = Range {
                start: token.range.start,
                end: alternative_end,
            };
            if_node.alternative = Some(ast::Block {
                nodes: vec![ast::Node {
                    value: ast::NodeValue::If(alternative),
                    range,
                }],
                range,
            });
            Ok((if_node, alternative_end))
        } else {
            let (alternative, alternative_end) = self.parse_block(token)?;

            if_node.alternative = Some(alternative);
            Ok((if_node, alternative_end))
        }
    }

    fn parse_while(&mut self) -> Result<(ast::NodeValue, Position)> {
        // Read `(`
        let token = self.next_token()?;
        validate_token_kind(&token, TokenKind::LBracket)?;

        // Parse condition
        let cond_token = self.next_token()?;
        let condition = self.parse_node(cond_token, Precedence::Lowest)?;
        validate_node_kind(&condition, NodeKind::Expression)?;

        // Read `)`
        let token = self.next_token()?;
        validate_token_kind(&token, TokenKind::RBracket)?;

        let block_token = self.next_token()?;
        let (block, end) = self.parse_block(block_token)?;

        Ok((
            ast::NodeValue::While(ast::While {
                condition: Box::new(condition),
                body: block,
            }),
            end,
        ))
    }

    fn parse_for(&mut self) -> Result<(ast::NodeValue, Position)> {
        // Read `(`
        let token = self.next_token()?;
        validate_token_kind(&token, TokenKind::LBracket)?;

        // Read inside params.
        let (params, end) = self.parse_multiple(
            TokenKind::RBracket,
            TokenKind::Semicolon,
            |parser, token| parser.parse_node(token, Precedence::Lowest),
        )?;

        if params.len() != 3 {
            return Err(Error {
                kind: ErrorKind::InvalidRange,
                range: Range {
                    start: token.range.start,
                    end,
                },
            });
        }

        let mut params = params.into_iter();
        let initial = params.next().unwrap();
        let condition = params.next().unwrap();
        let after = params.next().unwrap();

        validate_node_kind(&condition, NodeKind::Expression)?;

        let body_token = self.next_token()?;
        let (body, end) = self.parse_block(body_token)?;

        Ok((
            ast::NodeValue::For(ast::For {
                initial: Box::new(initial),
                condition: Box::new(condition),
                after: Box::new(after),
                body,
            }),
            end,
        ))
    }

    fn parse_fn_literal(&mut self) -> Result<(ast::NodeValue, Position)> {
        // Read `(`
        let token = self.next_token()?;
        validate_token_kind(&token, TokenKind::LBracket)?;

        // Read arguments
        let (args, _) = self.parse_multiple(
            TokenKind::RBracket,
            TokenKind::Comma,
            |_, token| match token.kind {
                TokenKind::Ident(ident) => Ok(FunctionParamter {
                    name: ident,
                    range: token.range,
                }),
                _ => Err(Error {
                    kind: ErrorKind::InvalidFunctionParameter,
                    range: token.range,
                }),
            },
        )?;

        let body_token = self.next_token()?;
        let (body, end) = self.parse_block(body_token)?;

        Ok((
            ast::NodeValue::FunctionLiteral(ast::FunctionLiteral {
                name: None,
                parameters: args,
                body,
            }),
            end,
        ))
    }

    fn parse_fn_call(&mut self, left: ast::Node) -> Result<(ast::NodeValue, Position)> {
        // Read arguments
        let (args, end) =
            self.parse_multiple(TokenKind::RBracket, TokenKind::Comma, |parser, token| {
                parser.parse_node(token, Precedence::Lowest)
            })?;

        // Check all nodes are expression
        for arg in &args {
            validate_node_kind(arg, NodeKind::Expression)?;
        }

        Ok((
            ast::NodeValue::FunctionCall(ast::FunctionCall {
                function: Box::new(left),
                arguments: args,
            }),
            end,
        ))
    }

    // Helper function that reads block { ... }.
    // It returns vector of nodes and end position, which is the end
    // position of `}`
    //
    // This function checks if the start token is `{`, so the caller doesn't have to do this.
    fn parse_block(&mut self, start_token: Token) -> Result<(ast::Block, Position)> {
        // Start token should be `{`
        validate_token_kind(&start_token, TokenKind::LCurly)?;

        let (nodes, end) =
            self.parse_multiple(TokenKind::RCurly, TokenKind::Eol, |parser, token| {
                parser.parse_node(token, Precedence::Lowest)
            })?;

        Ok((
            ast::Block {
                nodes,
                range: Range {
                    start: start_token.range.start,
                    end,
                },
            },
            end,
        ))
    }

    // Helper function used for parsing arrays, hash maps, function arguments, function calls.
    fn parse_multiple<T, F>(
        &mut self,
        end_token: TokenKind,
        separator: TokenKind,
        parse_item: F,
    ) -> Result<(Vec<T>, Position)>
    where
        F: Fn(&mut Self, Token) -> Result<T>,
    {
        let mut res = vec![];

        loop {
            self.skip_eol()?;

            let token = self.next_token()?;
            if token.kind == end_token {
                return Ok((res, token.range.end));
            }

            let item = parse_item(self, token)?;
            res.push(item);

            let token = self.next_token()?;

            if token.kind == end_token {
                return Ok((res, token.range.end));
            }

            if token.kind != separator {
                return Err(Error {
                    kind: ErrorKind::InvalidTokenKind {
                        expected: end_token,
                        got: token.kind,
                    },
                    range: token.range,
                });
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
