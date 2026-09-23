use crate::token::{Kind, Token};
use crate::value::Value;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Clone)]
pub(crate) struct Scanner<'a> {
    storage: String,
    cursor: (usize, usize),
    source: Peekable<Chars<'a>>,
    token_start: Option<(usize, usize)>,
}

impl<'a> Scanner<'a> {
    pub(crate) fn new(source: &'a str) -> Scanner<'a> {
        Scanner {
            cursor: (1, 1),
            token_start: None,
            storage: String::new(),
            source: source.chars().peekable(),
        }
    }

    fn new_token(&mut self, kind: Kind, start: (usize, usize), proceed_by: usize) -> Option<Token> {
        self.cursor = (self.cursor.0, self.cursor.1 + proceed_by);
        Some(Token::new(kind, start, None))
    }
}

impl Scanner<'_> {
    fn scan_punctuation(&mut self, c: char) -> Option<Token> {
        let kind = match c {
            '(' => Kind::LeftParen,
            ')' => Kind::RightParen,
            ';' => Kind::Semicolon,
            ',' => Kind::Comma,
            '.' => Kind::Dot,
            '+' => Kind::Plus,
            '-' => Kind::Minus,
            '*' => Kind::Star,
            '%' => Kind::Percent,
            '{' => Kind::LeftBrace,
            '}' => Kind::RightBrace,
            _ => unreachable!(),
        };
        self.new_token(kind, self.cursor, 1)
    }

    fn scan_slash(&mut self) -> Option<Token> {
        match self.source.peek() {
            Some('/') => {
                while self.source.peek().is_some() {
                    if self.source.next().unwrap() == '\n' {
                        self.cursor = (self.cursor.0 + 1, 1);
                        break;
                    }
                }
                self.next()
            }
            None | Some(_) => self.new_token(Kind::Slash, self.cursor, 1),
        }
    }

    fn unexpected_eof_token(&self) -> Token {
        Token::new(
            Kind::Error,
            self.cursor,
            Some(Value::from("Unexpected end of script")),
        )
    }

    fn scan_two_char_op(
        &mut self,
        second: char,
        matched_kind: Kind,
        default_kind: Kind,
    ) -> Option<Token> {
        match self.source.peek() {
            Some(c) if *c == second => {
                self.source.next();
                self.new_token(matched_kind, self.cursor, 2)
            }
            Some(_) => self.new_token(default_kind, self.cursor, 1),
            None => Some(self.unexpected_eof_token()),
        }
    }

    fn scan_bang(&mut self) -> Option<Token> {
        match self.source.peek() {
            Some('=') => {
                self.source.next();
                self.new_token(Kind::BangEqual, self.cursor, 2)
            }
            None => Some(self.unexpected_eof_token()),
            Some(character) => Some(Token::new(
                Kind::Error,
                self.cursor,
                Some(Value::String(format!("Unexpected character {}", character))),
            )),
        }
    }

    fn scan_less(&mut self) -> Option<Token> {
        match self.source.peek() {
            Some('=') => {
                self.source.next();
                self.new_token(Kind::LessEqual, self.cursor, 2)
            }
            Some('>') => {
                self.source.next();
                self.new_token(Kind::Concat, self.cursor, 2)
            }
            Some(_) => self.new_token(Kind::Less, self.cursor, 1),
            None => Some(self.unexpected_eof_token()),
        }
    }

    fn begin_multi_char_token(&mut self, first: Option<char>) {
        self.token_start = Some(self.cursor);
        self.cursor = (self.cursor.0, self.cursor.1 + 1);
        if let Some(c) = first {
            self.storage.push(c);
        }
    }

    fn finish_token(&mut self, token: Token) -> Option<Token> {
        self.storage = String::new();
        self.token_start = None;
        Some(token)
    }

    fn scan_string(&mut self) -> Option<Token> {
        self.begin_multi_char_token(None);
        loop {
            let peeked = self.source.peek();

            if peeked.is_none() {
                return Some(self.unexpected_eof_token());
            }

            if *peeked.unwrap() == '"' {
                self.source.next();
                self.cursor = (self.cursor.0, self.cursor.1 + 1);
                break;
            }

            if *peeked.unwrap() == '\n' {
                self.cursor = (self.cursor.0 + 1, 1);
            } else {
                self.cursor = (self.cursor.0, self.cursor.1 + 1);
            }

            self.storage.push(*peeked.unwrap());
            self.source.next();
        }

        let token = Token::new(
            Kind::String,
            self.token_start.unwrap(),
            Some(Value::String(self.storage.clone())),
        );
        self.finish_token(token)
    }

    fn scan_number(&mut self, first: char) -> Option<Token> {
        self.begin_multi_char_token(Some(first));
        loop {
            let peeked = self.source.peek();

            if peeked.is_none()
                || (!(*peeked.unwrap()).is_numeric()
                    && (self.storage.contains('.') || *peeked.unwrap() != '.'))
            {
                break;
            }

            self.cursor = (self.cursor.0, self.cursor.1 + 1);
            self.storage.push(*peeked.unwrap());
            self.source.next();
        }

        let token = Token::new(
            Kind::Number,
            self.token_start.unwrap(),
            Some(Value::Number(self.storage.parse().unwrap())),
        );
        self.finish_token(token)
    }

    fn is_ident_continuation(c: char) -> bool {
        c.is_ascii_alphanumeric() || c as u32 == 0x5f
    }

    fn scan_identifier(&mut self, first: char) -> Option<Token> {
        self.begin_multi_char_token(Some(first));
        loop {
            let peeked = self.source.peek();

            if peeked.is_none() || !Self::is_ident_continuation(*peeked.unwrap()) {
                break;
            }

            self.cursor = (self.cursor.0, self.cursor.1 + 1);
            self.storage.push(*peeked.unwrap());
            self.source.next();
        }

        if let Some(character) = self.source.peek() {
            if ['!', '?'].contains(character) {
                self.storage.push(*character);
                self.source.next();
            }
        }

        let token = if let Some(keyword_kind) = Kind::keyword_equivalent(&self.storage) {
            Token::new(
                keyword_kind,
                self.token_start.unwrap(),
                Some(Value::String(self.storage.clone())),
            )
        } else {
            Token::new(
                Kind::Identifier,
                self.token_start.unwrap(),
                Some(Value::String(self.storage.clone())),
            )
        };

        self.finish_token(token)
    }
}

impl Iterator for Scanner<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        match self.source.next() {
            Some('\n') => {
                self.cursor = (self.cursor.0 + 1, 1);
                self.next()
            }

            Some(' ' | '\r' | '\t') => {
                self.cursor = (self.cursor.0, self.cursor.1 + 1);
                self.next()
            }

            Some(c @ ('(' | ')' | ';' | ',' | '.' | '+' | '-' | '*' | '%' | '{' | '}')) => {
                self.scan_punctuation(c)
            }

            Some('/') => self.scan_slash(),
            Some('!') => self.scan_bang(),
            Some('=') => self.scan_two_char_op('=', Kind::EqualEqual, Kind::Equal),
            Some('<') => self.scan_less(),
            Some('>') => self.scan_two_char_op('=', Kind::GreaterEqual, Kind::Greater),
            Some('"') => self.scan_string(),

            Some(c) if c.is_numeric() => self.scan_number(c),
            Some(c) if c.is_alphabetic() || c == '_' => self.scan_identifier(c),

            Some(c) => Some(Token::new(
                Kind::Error,
                self.cursor,
                Some(Value::String(format!("Unexpected character {}", c))),
            )),

            None => Some(Token::new(Kind::Eof, self.cursor, None)),
        }
    }
}
