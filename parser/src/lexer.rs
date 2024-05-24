use crate::{
    error::{Error, ErrorKind, Result},
    position::Position,
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
    fn read_number(&mut self, start: usize, mut end: usize) -> Result<TokenKind> {
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
            let float: f64 = number.parse().map_err(|_| {
                let mut pos = self.position;
                pos.character -= len;
                Error {
                    kind: ErrorKind::InvalidNumber(number.to_string()),
                    position: pos,
                }
            })?;

            Ok(TokenKind::Float(float))
        } else {
            let int: i64 = number.parse().map_err(|_| {
                let mut pos = self.position;
                pos.character -= len;
                Error {
                    kind: ErrorKind::InvalidNumber(number.to_string()),
                    position: pos,
                }
            })?;

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
    fn read_string(&mut self) -> Result<TokenKind> {
        let mut string = String::new();

        loop {
            let (_, ch) = self.chars.next().ok_or(Error {
                kind: ErrorKind::UnexpectedEof,
                position: self.position,
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
                position: self.position,
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
                        position,
                    });
                }
            };
            string.push(escaped);
        }

        Ok(TokenKind::String(string))
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        let (start, ch) = self.chars.next()?;
        let current_position = self.position;

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
            '/' => TokenKind::Div,
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
            '"' => match self.read_string() {
                Ok(token) => token,
                Err(err) => return Some(Err(err)),
            },
            ch if ch.is_ascii_digit() => {
                self.position.character -= ch.len_utf8();

                match self.read_number(start, start + ch.len_utf8()) {
                    Ok(token) => token,
                    Err(err) => return Some(Err(err)),
                }
            }
            ch if ch.is_alphabetic() => {
                self.position.character -= ch.len_utf8();
                self.read_ident(start, start + ch.len_utf8())
            }
            ch => {
                return Some(Err(Error {
                    kind: ErrorKind::InvalidChar(ch),
                    position: current_position,
                }))
            }
        };

        Some(Ok(Token {
            kind: token_type,
            position: current_position,
        }))
    }
}

#[cfg(test)]
mod test {
    use crate::{
        error::{Error, ErrorKind},
        position::Position,
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
                position: Position::new(0, 0)
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
                    position: Position::new(0, 0),
                },
            ),
            (
                "\"asdf",
                Error {
                    kind: ErrorKind::UnexpectedEof,
                    position: Position::new(0, 5),
                },
            ),
            (
                "\"asdf\\",
                Error {
                    kind: ErrorKind::UnexpectedEof,
                    position: Position::new(0, 6),
                },
            ),
            (
                "\"asdf\\a",
                Error {
                    kind: ErrorKind::InvalidEscapeChar('a'),
                    position: Position::new(0, 6),
                },
            ),
            (
                "123 $",
                Error {
                    kind: ErrorKind::InvalidChar('$'),
                    position: Position::new(0, 4),
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
        "#;

        let lexer = Lexer::new(input);
        let (tokens, positions): (Vec<TokenKind>, Vec<Position>) = lexer
            .map(|token| {
                let token = token.unwrap();
                (token.kind, token.position)
            })
            .unzip();

        assert_eq!(
            positions,
            vec![
                Position::new(0, 0),
                Position::new(1, 12),
                Position::new(1, 14),
                Position::new(1, 16),
                Position::new(1, 17),
                Position::new(1, 18),
                Position::new(1, 19),
                Position::new(1, 21),
                Position::new(1, 23),
                Position::new(1, 25),
                Position::new(2, 12),
                Position::new(2, 14),
                Position::new(2, 17),
                Position::new(2, 20),
                Position::new(2, 22),
                Position::new(3, 12),
                Position::new(3, 13),
                Position::new(3, 14),
                Position::new(3, 15),
                Position::new(3, 16),
                Position::new(3, 17),
                Position::new(3, 18),
                Position::new(3, 19),
                Position::new(3, 20),
                Position::new(3, 21),
                Position::new(3, 22),
                Position::new(3, 23),
                Position::new(3, 24),
                Position::new(4, 12),
                Position::new(4, 16),
                Position::new(4, 21),
                Position::new(5, 12),
                Position::new(5, 17),
                Position::new(5, 23),
                Position::new(5, 26),
                Position::new(5, 31),
                Position::new(5, 37),
                Position::new(5, 41),
                Position::new(5, 47),
                Position::new(5, 56),
                Position::new(5, 63),
                Position::new(5, 66),
                Position::new(5, 69),
                Position::new(6, 12),
                Position::new(6, 16),
                Position::new(6, 21),
                Position::new(6, 27),
                Position::new(6, 34),
                Position::new(7, 12),
                Position::new(7, 28),
                Position::new(7, 38),
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
            ]
        );
    }
}
