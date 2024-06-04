use crate::enums::TokenKind;

pub struct Scanner {
    start: usize,
    current: usize,
    line: usize,
    source: String,
}

impl Scanner {
    pub fn get_tokens(source: String) -> Vec<Token> {
        let mut scanner = Scanner {
            start: 0,
            current: 0,
            line: 1,
            source,
        };
        let mut res = vec![];
        loop {
            let token = scanner.next_token();
            let kind = token.kind;
            res.push(token);
            match kind {
                TokenKind::Eof => break,
                _ => {}
            }
        }
        res
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;
        if self.is_at_end() {
            return self.make_token(TokenKind::Eof);
        }

        let c = self.advance();
        if c.is_ascii_alphabetic() {
            return self.identifier();
        }
        if c.is_digit(10) {
            return self.number();
        }
        match c {
            '(' => return self.make_token(TokenKind::LeftParen),
            ')' => return self.make_token(TokenKind::RightParen),
            '{' => return self.make_token(TokenKind::LeftBrace),
            '}' => return self.make_token(TokenKind::RightBrace),
            ';' => return self.make_token(TokenKind::Semicolon),
            ',' => return self.make_token(TokenKind::Comma),
            '.' => return self.make_token(TokenKind::Dot),
            '-' => return self.make_token(TokenKind::Minus),
            '+' => return self.make_token(TokenKind::Plus),
            '/' => return self.make_token(TokenKind::Slash),
            '*' => return self.make_token(TokenKind::Star),
            ':' => return self.make_token(TokenKind::Colon),
            '%' => return self.make_token(TokenKind::Percent),
            '!' => {
                let token = if self.check_next('=') {
                    TokenKind::BangEqual
                } else {
                    TokenKind::Bang
                };
                return self.make_token(token);
            }
            '=' => {
                let token = if self.check_next('=') {
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                };
                return self.make_token(token);
            }
            '<' => {
                let token = if self.check_next('=') {
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                };
                return self.make_token(token);
            }
            '>' => {
                let token = if self.check_next('=') {
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                };
                return self.make_token(token);
            }
            '"' => return self.string(),
            _ => {}
        }

        return self.error_token("Unexpected character");
    }

    fn identifier(&mut self) -> Token {
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphabetic() || c.is_digit(10))
        {
            self.advance();
        }
        return self.make_token(self.identifier_kind());
    }

    fn identifier_list(&self) -> Vec<(String, TokenKind)> {
        vec![
            ("true".to_string(), TokenKind::True),
            ("and".to_string(), TokenKind::And),
            ("class".to_string(), TokenKind::Class),
            ("else".to_string(), TokenKind::Else),
            ("if".to_string(), TokenKind::If),
            ("nil".to_string(), TokenKind::Nil),
            ("or".to_string(), TokenKind::Or),
            ("print".to_string(), TokenKind::Print),
            ("return".to_string(), TokenKind::Return),
            ("int".to_string(), TokenKind::Int),
            ("str".to_string(), TokenKind::Str),
            ("bool".to_string(), TokenKind::Bool),
            ("while".to_string(), TokenKind::While),
            ("false".to_string(), TokenKind::False),
            ("for".to_string(), TokenKind::For),
            ("fun".to_string(), TokenKind::Fun),
            ("in".to_string(), TokenKind::In),
        ]
    }

    fn identifier_kind(&self) -> TokenKind {
        for (s, kind) in self.identifier_list() {
            if self.start + s.len() < self.source.len()
                && &self.source[self.start..self.start + s.len()] == &s
            {
                return kind;
            }
        }
        TokenKind::Identifier
    }

    fn number(&mut self) -> Token {
        while self.peek().is_some() && self.peek().unwrap().is_digit(10) {
            self.advance();
        }
        return self.make_token(TokenKind::Number);
    }

    fn string(&mut self) -> Token {
        while self.peek().is_some() && self.peek().unwrap() != '"' && !self.is_at_end() {
            if self.peek().unwrap() == '\n' {
                self.line += 1;
                self.advance();
            }
            self.advance();
        }
        if self.is_at_end() {
            return self.error_token("unterminated string");
        }
        self.advance();
        return self.make_token(TokenKind::String);
    }

    fn skip_whitespace(&mut self) {
        loop {
            let temp = self.peek();
            if let Some(c) = temp {
                match c {
                    ' ' | '\r' | '\t' => _ = self.advance(),
                    '\n' => {
                        self.line += 1;
                        self.advance();
                    }
                    '/' => {
                        if self.peek_next().is_some() && self.peek_next().unwrap() == '/' {
                            while self.peek().is_some()
                                && self.peek().unwrap() != '\n'
                                && !self.is_at_end()
                            {
                                self.advance();
                            }
                        } else {
                            break;
                        }
                    }
                    _ => break,
                }
            } else {
                break;
            }
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.is_at_end() {
            return None;
        }
        return self.source.chars().nth(self.current + 1);
    }

    fn peek(&self) -> Option<char> {
        return self.source.chars().nth(self.current);
    }

    fn check_next(&mut self, c: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != c {
            return false;
        }
        self.current += 1;
        true
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source.chars().nth(self.current - 1).unwrap()
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        match kind {
            TokenKind::String => Token {
                kind,
                line: self.line,
                value: self.source[(self.start + 1)..(self.current - 1)].to_string(),
            },
            _ => Token {
                kind,
                line: self.line,
                value: self.source[self.start..self.current].to_string(),
            },
        }
    }

    fn error_token(&self, message: &str) -> Token {
        Token {
            kind: TokenKind::Error,
            line: self.line,
            value: message.to_string(),
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub value: String,
}

#[cfg(test)]
mod tests {
    use super::Scanner;
    use crate::scanner::TokenKind;

    #[test]
    fn kinds() {
        let source = r#"
            int i = 1;
            str hello = "a string";
            print "world" + "!";
        "#;
        let tokens = Scanner::get_tokens(source.to_string());
        let temp = vec![
            TokenKind::Int,
            TokenKind::Identifier,
            TokenKind::Equal,
            TokenKind::Number,
            TokenKind::Semicolon,
            TokenKind::Str,
            TokenKind::Identifier,
            TokenKind::Equal,
            TokenKind::String,
            TokenKind::Semicolon,
            TokenKind::Print,
            TokenKind::String,
            TokenKind::Plus,
            TokenKind::String,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ];
        let mut i = 0;
        for token in &tokens {
            assert_eq!(token.kind, temp[i]);
            i += 1;
        }
    }
}
