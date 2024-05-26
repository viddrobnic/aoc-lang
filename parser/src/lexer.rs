use crate::{
    error::{Error, ErrorKind, Result},
    position::{Position, Range},
    token::{Token, TokenKind},
};

pub struct Lexer<'a> {
    input: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,

    position: Position,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().peekable(),
            position: Position::default(),
        }
    }

    /// Skips whitespace, except \n which is a special token.
    fn skip_whitespace(&mut self) {
        loop {
            let Some((_, ch)) = self.chars.peek() else {
                return;
            };

            if ch.is_whitespace() && *ch != '\n' {
                self.position.character += ch.len_utf8();
                self.chars.next();
            } else {
                return;
            }
        }
    }

    fn peek_parse(
        &mut self,
        expected: char,
        token: TokenKind,
        default_token: TokenKind,
    ) -> TokenKind {
        match self.chars.peek() {
            Some((_, ch)) if *ch == expected => {
                self.chars.next();
                self.position.character += '='.len_utf8();
                token
            }
            _ => default_token,
        }
    }

    // Read number where the first digit is at `self.input[start]`
    // and `end` is start + utf8 len of first digit.
    fn read_number(
        &mut self,
        start: usize,
        mut end: usize,
    ) -> std::result::Result<TokenKind, ErrorKind> {
        loop {
            let Some((_, ch)) = self.chars.peek() else {
                break;
            };

            if ch.is_ascii_digit() || *ch == '.' {
                // We know it is Some(_), so it's safe to unwrap.
                let (_, ch) = self.chars.next().unwrap();
                end += ch.len_utf8();
            } else {
                break;
            }
        }

        let len = end - start;
        self.position.character += len;

        let number = &self.input[start..end];

        if number.contains('.') {
            let float: f64 = number
                .parse()
                .map_err(|_| ErrorKind::InvalidNumber(number.to_string()))?;

            Ok(TokenKind::Float(float))
        } else {
            let int: i64 = number
                .parse()
                .map_err(|_| ErrorKind::InvalidNumber(number.to_string()))?;

            Ok(TokenKind::Integer(int))
        }
    }

    // Read ident or keywoard, where the first char is at `self.input[start]`
    // and `end` is start + utf8 len of first char
    fn read_ident(&mut self, start: usize, mut end: usize) -> TokenKind {
        loop {
            let Some((_, ch)) = self.chars.peek() else {
                break;
            };

            if ch.is_alphabetic() || ch.is_ascii_digit() || *ch == '_' {
                // We know it is Some(_), so it's safe to unwrap.
                let (_, ch) = self.chars.next().unwrap();
                end += ch.len_utf8();
            } else {
                break;
            }
        }

        self.position.character += end - start;

        let ident = &self.input[start..end];
        TokenKind::from_ident(ident).unwrap_or_else(|| TokenKind::Ident(ident.to_string()))
    }

    // Read string, where `"` is already read.
    fn read_string(&mut self, start_position: Position) -> Result<TokenKind> {
        let mut string = String::new();

        loop {
            let (_, ch) = self.chars.next().ok_or(Error {
                kind: ErrorKind::UnexpectedEof,
                range: Range {
                    start: start_position,
                    end: self.position,
                },
            })?;
            self.position.character += ch.len_utf8();

            if ch == '"' {
                break;
            }

            if ch != '\\' {
                string.push(ch);
                continue;
            }

            let (_, ch) = self.chars.next().ok_or(Error {
                kind: ErrorKind::UnexpectedEof,
                range: Range {
                    start: start_position,
                    end: self.position,
                },
            })?;
            self.position.character += ch.len_utf8();

            let escaped = match ch {
                'n' => '\n',
                't' => '\t',
                '"' => '"',
                '\\' => '\\',
                ch => {
                    let mut position = self.position;
                    position.character -= ch.len_utf8();

                    return Err(Error {
                        kind: ErrorKind::InvalidEscapeChar(ch),
                        range: Range {
                            start: position,
                            end: self.position,
                        },
                    });
                }
            };
            string.push(escaped);
        }

        Ok(TokenKind::String(string))
    }

    /// Read comment where first / is already read.
    fn read_comment(&mut self) -> std::result::Result<TokenKind, ErrorKind> {
        // Read the second /
        let Some((pos, ch)) = self.chars.next() else {
            return Err(ErrorKind::UnexpectedEof);
        };

        if ch != '/' {
            return Err(ErrorKind::InvalidChar(ch));
        }

        // Increase position after possible error returning,
        // to ensure correct position is in the error.
        self.position.character += ch.len_utf8();

        // Read comment string
        let start = pos + ch.len_utf8();
        let mut end = start;

        loop {
            let Some((_, ch)) = self.chars.peek() else {
                break;
            };

            if *ch == '\n' {
                break;
            }

            let len = ch.len_utf8();
            self.chars.next();

            self.position.character += len;
            end += len;
        }

        let comment = &self.input[start..end];
        Ok(TokenKind::Comment(comment.trim().to_string()))
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        let start_position = self.position;

        let (start, ch) = self.chars.next()?;
        self.position.character += ch.len_utf8();

        let token_type = match ch {
            '[' => TokenKind::LSquare,
            ']' => TokenKind::RSquare,
            '(' => TokenKind::LBracket,
            ')' => TokenKind::RBracket,
            '{' => TokenKind::LCurly,
            '}' => TokenKind::RCurly,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Mult,
            '%' => TokenKind::Modulo,
            '&' => TokenKind::And,
            '|' => TokenKind::Or,
            ';' => TokenKind::Semicolon,
            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,
            '\n' => {
                self.position.line += 1;
                self.position.character = 0;
                TokenKind::Eol
            }
            '<' => self.peek_parse('=', TokenKind::Leq, TokenKind::Le),
            '>' => self.peek_parse('=', TokenKind::Geq, TokenKind::Ge),
            '=' => self.peek_parse('=', TokenKind::Eq, TokenKind::Assign),
            '!' => self.peek_parse('=', TokenKind::Neq, TokenKind::Bang),
            '"' => match self.read_string(start_position) {
                Ok(token) => token,
                Err(err) => return Some(Err(err)),
            },
            '/' => match self.chars.peek() {
                None => TokenKind::Div,
                Some((_, ch)) if *ch != '/' => TokenKind::Div,
                _ => match self.read_comment() {
                    Ok(token) => token,
                    Err(kind) => {
                        return Some(Err(Error {
                            kind,
                            range: Range {
                                start: start_position,
                                end: self.position,
                            },
                        }))
                    }
                },
            },
            ch if ch.is_ascii_digit() => {
                self.position.character -= ch.len_utf8();

                match self.read_number(start, start + ch.len_utf8()) {
                    Ok(token) => token,
                    Err(kind) => {
                        return Some(Err(Error {
                            kind,
                            range: Range {
                                start: start_position,
                                end: self.position,
                            },
                        }))
                    }
                }
            }
            ch if ch.is_alphabetic() => {
                self.position.character -= ch.len_utf8();
                self.read_ident(start, start + ch.len_utf8())
            }
            ch => {
                return Some(Err(Error {
                    kind: ErrorKind::InvalidChar(ch),
                    range: Range {
                        start: start_position,
                        end: self.position,
                    },
                }))
            }
        };

        Some(Ok(Token {
            kind: token_type,
            range: Range {
                start: start_position,
                end: self.position,
            },
        }))
    }
}

#[cfg(test)]
mod test {
    use crate::{
        error::{Error, ErrorKind},
        position::{Position, Range},
        token::{Token, TokenKind},
    };

    use super::Lexer;

    #[test]
    fn skip_whitespace() {
        let mut lexer = Lexer::new("  \t  asdf");
        lexer.skip_whitespace();
        assert_eq!(lexer.chars.peek(), Some((5, 'a')).as_ref());
        assert_eq!(lexer.position.line, 0);
        assert_eq!(lexer.position.character, 5);

        let mut lexer = Lexer::new("\t  \n");
        lexer.skip_whitespace();
        assert_eq!(lexer.chars.peek(), Some((3, '\n')).as_ref());
        assert_eq!(lexer.position.line, 0);
        assert_eq!(lexer.position.character, 3);
    }

    #[test]
    fn parse_number() {
        let lexer = Lexer::new("123");
        let tokens = lexer.collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(
            tokens,
            vec![Token {
                kind: TokenKind::Integer(123),
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(0, 3)
                }
            }]
        );
    }

    #[test]
    fn errors() {
        let tests = [
            (
                "1.2.3",
                Error {
                    kind: ErrorKind::InvalidNumber("1.2.3".to_string()),
                    range: Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 5),
                    },
                },
            ),
            (
                "\"asdf",
                Error {
                    kind: ErrorKind::UnexpectedEof,
                    range: Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 5),
                    },
                },
            ),
            (
                "\"asdf\\",
                Error {
                    kind: ErrorKind::UnexpectedEof,
                    range: Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 6),
                    },
                },
            ),
            (
                "\"asdf\\a",
                Error {
                    kind: ErrorKind::InvalidEscapeChar('a'),
                    range: Range {
                        start: Position::new(0, 6),
                        end: Position::new(0, 7),
                    },
                },
            ),
            (
                "123 $",
                Error {
                    kind: ErrorKind::InvalidChar('$'),
                    range: Range {
                        start: Position::new(0, 4),
                        end: Position::new(0, 5),
                    },
                },
            ),
        ];

        for (input, expected) in tests {
            let lexer = Lexer::new(input);
            let result: Result<Vec<_>, _> = lexer.collect();
            assert_eq!(result, Err(expected));
        }
    }

    #[test]
    fn lex_program() {
        let input = r#"
            [ ] (){} < <=
            > >= == !=
            !+-*/%&|=;,.
            123 1.234
            true false if else while for break continue return fn use
            foo bar1 bar_1 bar_baz
            "normal string" "\n\t\\\""
            // line comment
            false //inline comment
        "#;

        let lexer = Lexer::new(input);
        let (tokens, ranges): (Vec<TokenKind>, Vec<Range>) = lexer
            .map(|token| {
                let token = token.unwrap();
                (token.kind, token.range)
            })
            .unzip();

        assert_eq!(
            ranges,
            vec![
                Range {
                    start: Position::new(0, 0),
                    end: Position::new(1, 0),
                },
                Range {
                    start: Position::new(1, 12),
                    end: Position::new(1, 13),
                },
                Range {
                    start: Position::new(1, 14),
                    end: Position::new(1, 15),
                },
                Range {
                    start: Position::new(1, 16),
                    end: Position::new(1, 17),
                },
                Range {
                    start: Position::new(1, 17),
                    end: Position::new(1, 18),
                },
                Range {
                    start: Position::new(1, 18),
                    end: Position::new(1, 19),
                },
                Range {
                    start: Position::new(1, 19),
                    end: Position::new(1, 20),
                },
                Range {
                    start: Position::new(1, 21),
                    end: Position::new(1, 22),
                },
                Range {
                    start: Position::new(1, 23),
                    end: Position::new(1, 25),
                },
                Range {
                    start: Position::new(1, 25),
                    end: Position::new(2, 0),
                },
                Range {
                    start: Position::new(2, 12),
                    end: Position::new(2, 13),
                },
                Range {
                    start: Position::new(2, 14),
                    end: Position::new(2, 16),
                },
                Range {
                    start: Position::new(2, 17),
                    end: Position::new(2, 19),
                },
                Range {
                    start: Position::new(2, 20),
                    end: Position::new(2, 22),
                },
                Range {
                    start: Position::new(2, 22),
                    end: Position::new(3, 0),
                },
                Range {
                    start: Position::new(3, 12),
                    end: Position::new(3, 13),
                },
                Range {
                    start: Position::new(3, 13),
                    end: Position::new(3, 14),
                },
                Range {
                    start: Position::new(3, 14),
                    end: Position::new(3, 15),
                },
                Range {
                    start: Position::new(3, 15),
                    end: Position::new(3, 16),
                },
                Range {
                    start: Position::new(3, 16),
                    end: Position::new(3, 17),
                },
                Range {
                    start: Position::new(3, 17),
                    end: Position::new(3, 18),
                },
                Range {
                    start: Position::new(3, 18),
                    end: Position::new(3, 19),
                },
                Range {
                    start: Position::new(3, 19),
                    end: Position::new(3, 20),
                },
                Range {
                    start: Position::new(3, 20),
                    end: Position::new(3, 21),
                },
                Range {
                    start: Position::new(3, 21),
                    end: Position::new(3, 22),
                },
                Range {
                    start: Position::new(3, 22),
                    end: Position::new(3, 23),
                },
                Range {
                    start: Position::new(3, 23),
                    end: Position::new(3, 24),
                },
                Range {
                    start: Position::new(3, 24),
                    end: Position::new(4, 0),
                },
                Range {
                    start: Position::new(4, 12),
                    end: Position::new(4, 15),
                },
                Range {
                    start: Position::new(4, 16),
                    end: Position::new(4, 21),
                },
                Range {
                    start: Position::new(4, 21),
                    end: Position::new(5, 0),
                },
                Range {
                    start: Position::new(5, 12),
                    end: Position::new(5, 16),
                },
                Range {
                    start: Position::new(5, 17),
                    end: Position::new(5, 22),
                },
                Range {
                    start: Position::new(5, 23),
                    end: Position::new(5, 25),
                },
                Range {
                    start: Position::new(5, 26),
                    end: Position::new(5, 30),
                },
                Range {
                    start: Position::new(5, 31),
                    end: Position::new(5, 36),
                },
                Range {
                    start: Position::new(5, 37),
                    end: Position::new(5, 40),
                },
                Range {
                    start: Position::new(5, 41),
                    end: Position::new(5, 46),
                },
                Range {
                    start: Position::new(5, 47),
                    end: Position::new(5, 55),
                },
                Range {
                    start: Position::new(5, 56),
                    end: Position::new(5, 62),
                },
                Range {
                    start: Position::new(5, 63),
                    end: Position::new(5, 65),
                },
                Range {
                    start: Position::new(5, 66),
                    end: Position::new(5, 69),
                },
                Range {
                    start: Position::new(5, 69),
                    end: Position::new(6, 0),
                },
                Range {
                    start: Position::new(6, 12),
                    end: Position::new(6, 15),
                },
                Range {
                    start: Position::new(6, 16),
                    end: Position::new(6, 20),
                },
                Range {
                    start: Position::new(6, 21),
                    end: Position::new(6, 26),
                },
                Range {
                    start: Position::new(6, 27),
                    end: Position::new(6, 34),
                },
                Range {
                    start: Position::new(6, 34),
                    end: Position::new(7, 0),
                },
                Range {
                    start: Position::new(7, 12),
                    end: Position::new(7, 27),
                },
                Range {
                    start: Position::new(7, 28),
                    end: Position::new(7, 38),
                },
                Range {
                    start: Position::new(7, 38),
                    end: Position::new(8, 0),
                },
                Range {
                    start: Position::new(8, 12),
                    end: Position::new(8, 27),
                },
                Range {
                    start: Position::new(8, 27),
                    end: Position::new(9, 0),
                },
                Range {
                    start: Position::new(9, 12),
                    end: Position::new(9, 17),
                },
                Range {
                    start: Position::new(9, 18),
                    end: Position::new(9, 34),
                },
                Range {
                    start: Position::new(9, 34),
                    end: Position::new(10, 0),
                },
            ]
        );

        assert_eq!(
            tokens,
            vec![
                TokenKind::Eol,
                TokenKind::LSquare,
                TokenKind::RSquare,
                TokenKind::LBracket,
                TokenKind::RBracket,
                TokenKind::LCurly,
                TokenKind::RCurly,
                TokenKind::Le,
                TokenKind::Leq,
                TokenKind::Eol,
                TokenKind::Ge,
                TokenKind::Geq,
                TokenKind::Eq,
                TokenKind::Neq,
                TokenKind::Eol,
                TokenKind::Bang,
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Mult,
                TokenKind::Div,
                TokenKind::Modulo,
                TokenKind::And,
                TokenKind::Or,
                TokenKind::Assign,
                TokenKind::Semicolon,
                TokenKind::Comma,
                TokenKind::Dot,
                TokenKind::Eol,
                TokenKind::Integer(123),
                TokenKind::Float(1.234),
                TokenKind::Eol,
                TokenKind::True,
                TokenKind::False,
                TokenKind::If,
                TokenKind::Else,
                TokenKind::While,
                TokenKind::For,
                TokenKind::Break,
                TokenKind::Continue,
                TokenKind::Return,
                TokenKind::Fn,
                TokenKind::Use,
                TokenKind::Eol,
                TokenKind::Ident("foo".to_string()),
                TokenKind::Ident("bar1".to_string()),
                TokenKind::Ident("bar_1".to_string()),
                TokenKind::Ident("bar_baz".to_string()),
                TokenKind::Eol,
                TokenKind::String("normal string".to_string()),
                TokenKind::String("\n\t\\\"".to_string()),
                TokenKind::Eol,
                TokenKind::Comment("line comment".to_string()),
                TokenKind::Eol,
                TokenKind::False,
                TokenKind::Comment("inline comment".to_string()),
                TokenKind::Eol,
            ]
        );
    }
}
