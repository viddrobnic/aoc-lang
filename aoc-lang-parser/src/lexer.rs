use crate::{
    position::Position,
    token::{Token, TokenType},
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
        token: TokenType,
        default_token: TokenType,
    ) -> TokenType {
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
    fn read_number(&mut self, start: usize) -> Result<TokenType, ()> {
        let end = loop {
            let (pos, ch) = self.chars.peek().ok_or(())?;

            if ch.is_ascii_digit() || *ch == '.' {
                self.chars.next();
            } else {
                break *pos;
            }
        };

        self.position.character += end - start;

        let number = &self.input[start..end];

        if number.contains('.') {
            let float: f64 = number.parse().map_err(|_| ())?;
            Ok(TokenType::Float(float))
        } else {
            let int: i64 = number.parse().map_err(|_| ())?;
            Ok(TokenType::Integer(int))
        }
    }

    // Read ident or keywoard, where the first char is at `self.input[start]`
    fn read_ident(&mut self, start: usize) -> Result<TokenType, ()> {
        let end = loop {
            let (pos, ch) = self.chars.peek().ok_or(())?;

            if ch.is_alphabetic() || ch.is_ascii_digit() || *ch == '_' {
                self.chars.next();
            } else {
                break *pos;
            }
        };

        self.position.character += end - start;

        let ident = &self.input[start..end];
        Ok(TokenType::from_ident(ident).unwrap_or_else(|| TokenType::Ident(ident.to_string())))
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        let (start, ch) = self.chars.next()?;
        let current_position = self.position;

        self.position.character += ch.len_utf8();

        let token_type = match ch {
            '[' => TokenType::LSquare,
            ']' => TokenType::RSquare,
            '(' => TokenType::LBracket,
            ')' => TokenType::RBracket,
            '{' => TokenType::LCurly,
            '}' => TokenType::RCurly,
            '+' => TokenType::Plus,
            '-' => TokenType::Minus,
            '*' => TokenType::Mult,
            '/' => TokenType::Div,
            '%' => TokenType::Modulo,
            '&' => TokenType::And,
            '|' => TokenType::Or,
            ';' => TokenType::Semicolon,
            ',' => TokenType::Comma,
            '\n' => {
                self.position.line += 1;
                self.position.character = 0;
                TokenType::Eol
            }
            '<' => self.peek_parse('=', TokenType::Leq, TokenType::Le),
            '>' => self.peek_parse('=', TokenType::Geq, TokenType::Ge),
            '=' => self.peek_parse('=', TokenType::Eq, TokenType::Assign),
            '!' => self.peek_parse('=', TokenType::Neq, TokenType::Bang),
            ch if ch.is_ascii_digit() => {
                self.position.character -= ch.len_utf8();

                match self.read_number(start) {
                    Ok(token) => token,
                    Err(err) => return Some(Err(err)),
                }
            }
            ch if ch.is_alphabetic() => {
                self.position.character -= ch.len_utf8();

                match self.read_ident(start) {
                    Ok(token) => token,
                    Err(err) => return Some(Err(err)),
                }
            }
            _ => return Some(Err(())),
        };

        Some(Ok(Token {
            token_type,
            position: current_position,
        }))
    }
}

#[cfg(test)]
mod test {
    use crate::{position::Position, token::TokenType};

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
    fn lex_program() {
        let input = r#"
            [ ] (){} < <=
            > >= == !=
            !+-*/%&|=;,
            123 1.234
            true false if else while for break continue return fn use
            foo bar1 bar_1 bar_baz
        "#;

        let lexer = Lexer::new(input);
        let (tokens, positions): (Vec<TokenType>, Vec<Position>) = lexer
            .map(|token| {
                let token = token.unwrap();
                (token.token_type, token.position)
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
            ]
        );

        assert_eq!(
            tokens,
            vec![
                TokenType::Eol,
                TokenType::LSquare,
                TokenType::RSquare,
                TokenType::LBracket,
                TokenType::RBracket,
                TokenType::LCurly,
                TokenType::RCurly,
                TokenType::Le,
                TokenType::Leq,
                TokenType::Eol,
                TokenType::Ge,
                TokenType::Geq,
                TokenType::Eq,
                TokenType::Neq,
                TokenType::Eol,
                TokenType::Bang,
                TokenType::Plus,
                TokenType::Minus,
                TokenType::Mult,
                TokenType::Div,
                TokenType::Modulo,
                TokenType::And,
                TokenType::Or,
                TokenType::Assign,
                TokenType::Semicolon,
                TokenType::Comma,
                TokenType::Eol,
                TokenType::Integer(123),
                TokenType::Float(1.234),
                TokenType::Eol,
                TokenType::True,
                TokenType::False,
                TokenType::If,
                TokenType::Else,
                TokenType::While,
                TokenType::For,
                TokenType::Break,
                TokenType::Continue,
                TokenType::Return,
                TokenType::Fn,
                TokenType::Use,
                TokenType::Eol,
                TokenType::Ident("foo".to_string()),
                TokenType::Ident("bar1".to_string()),
                TokenType::Ident("bar_1".to_string()),
                TokenType::Ident("bar_baz".to_string()),
                TokenType::Eol,
            ]
        );
    }
}
