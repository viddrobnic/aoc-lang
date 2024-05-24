use crate::{
    position::Position,
    token::{Token, TokenType},
};

pub struct Lexer<'a> {
    input: std::iter::Peekable<std::str::CharIndices<'a>>,

    position: Position,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.char_indices().peekable(),
            position: Position::default(),
        }
    }

    /// Skips whitespace, except \n which is a special token.
    fn skip_whitespace(&mut self) {
        loop {
            let Some((_, ch)) = self.input.peek() else {
                return;
            };

            if ch.is_whitespace() && *ch != '\n' {
                self.position.character += ch.len_utf8();
                self.input.next();
            } else {
                return;
            }
        }
    }

    fn peek_or(&mut self, expected: char, token: TokenType, default_token: TokenType) -> TokenType {
        match self.input.peek() {
            Some((_, ch)) if *ch == expected => {
                self.input.next();
                self.position.character += '='.len_utf8();
                token
            }
            _ => default_token,
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        let (_, ch) = self.input.next()?;
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
            '<' => self.peek_or('=', TokenType::Leq, TokenType::Le),
            '>' => self.peek_or('=', TokenType::Geq, TokenType::Ge),
            '=' => self.peek_or('=', TokenType::Eq, TokenType::Assign),
            '!' => self.peek_or('=', TokenType::Neq, TokenType::Bang),
            ch if ch.is_alphabetic() => {
                todo!()
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
        assert_eq!(lexer.input.peek(), Some((5, 'a')).as_ref());
        assert_eq!(lexer.position.line, 0);
        assert_eq!(lexer.position.character, 5);

        let mut lexer = Lexer::new("\t  \n");
        lexer.skip_whitespace();
        assert_eq!(lexer.input.peek(), Some((3, '\n')).as_ref());
        assert_eq!(lexer.position.line, 0);
        assert_eq!(lexer.position.character, 3);
    }

    #[test]
    fn lex_program() {
        let input = r#"
            [ ] (){} < <=
            > >= == !=
            !+-*/%&|=;,
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
                TokenType::Eol
            ]
        );
    }
}
