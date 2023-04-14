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
    pub(crate) fn new(source: &'a String) -> Scanner<'a> {
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

impl Iterator for Scanner<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        match self.source.next() {
            Some('\n') => {
                self.cursor = (self.cursor.0 + 1, 1);
                self.next()
            }

            Some(' ') | Some('\r') | Some('\t') => {
                self.cursor = (self.cursor.0, self.cursor.1 + 1);
                self.next()
            }

            Some('/') => match self.source.peek() {
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
            },

            Some('(') => self.new_token(Kind::LeftParen, self.cursor, 1),
            Some(')') => self.new_token(Kind::RightParen, self.cursor, 1),
            Some(';') => self.new_token(Kind::Semicolon, self.cursor, 1),
            Some(',') => self.new_token(Kind::Comma, self.cursor, 1),
            Some('.') => self.new_token(Kind::Dot, self.cursor, 1),
            Some('+') => self.new_token(Kind::Plus, self.cursor, 1),
            Some('-') => self.new_token(Kind::Minus, self.cursor, 1),
            Some('*') => self.new_token(Kind::Star, self.cursor, 1),
            Some('{') => self.new_token(Kind::LeftBrace, self.cursor, 1),
            Some('}') => self.new_token(Kind::RightBrace, self.cursor, 1),

            Some('!') => match self.source.peek() {
                Some('=') => {
                    self.source.next();
                    self.new_token(Kind::BangEqual, self.cursor, 2)
                }
                Some(_) => self.new_token(Kind::Bang, self.cursor, 1),
                None => Some(Token::new(
                    Kind::Error,
                    self.cursor,
                    Some(Value::from("Unexpected end of script")),
                )),
            },

            Some('=') => match self.source.peek() {
                Some('=') => {
                    self.source.next();
                    self.new_token(Kind::EqualEqual, self.cursor, 2)
                }
                Some(_) => self.new_token(Kind::Equal, self.cursor, 1),
                None => Some(Token::new(
                    Kind::Error,
                    self.cursor,
                    Some(Value::from("Unexpected end of script")),
                )),
            },

            Some('<') => match self.source.peek() {
                Some('=') => {
                    self.source.next();
                    self.new_token(Kind::LessEqual, self.cursor, 2)
                }
                Some('>') => {
                    self.source.next();
                    self.new_token(Kind::Concat, self.cursor, 2)
                }
                Some(_) => self.new_token(Kind::Less, self.cursor, 1),
                None => Some(Token::new(
                    Kind::Error,
                    self.cursor,
                    Some(Value::from("Unexpected end of script")),
                )),
            },

            Some('>') => match self.source.peek() {
                Some('=') => {
                    self.source.next();
                    self.new_token(Kind::GreateEqual, self.cursor, 2)
                }
                Some(_) => self.new_token(Kind::Greater, self.cursor, 1),
                None => Some(Token::new(
                    Kind::Error,
                    self.cursor,
                    Some(Value::from("Unexpected end of script")),
                )),
            },

            Some('"') => {
                self.token_start = Some(self.cursor);
                self.cursor = (self.cursor.0, self.cursor.1 + 1);
                loop {
                    let peeked = self.source.peek();

                    if peeked.is_none() {
                        return Some(Token::new(
                            Kind::Error,
                            self.cursor,
                            Some(Value::from("Unexpected end of script")),
                        ));
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
                self.storage = String::new();
                self.token_start = None;
                return Some(token);
            }

            Some(character) if character.is_numeric() => {
                self.token_start = Some(self.cursor);
                self.cursor = (self.cursor.0, self.cursor.1 + 1);
                self.storage.push(character);
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
                    Some(Value::Double(self.storage.parse().unwrap())),
                );
                self.storage = String::new();
                self.token_start = None;
                return Some(token);
            }

            Some(character) if character.is_alphabetic() || character == '_' => {
                self.token_start = Some(self.cursor);
                self.cursor = (self.cursor.0, self.cursor.1 + 1);
                self.storage.push(character);
                loop {
                    let peeked = self.source.peek();

                    if peeked.is_none() || {
                        !(*peeked.unwrap()).is_numeric()
                            && !(*peeked.unwrap()).is_alphabetic()
                            && *peeked.unwrap() != '_'
                    } {
                        break;
                    }

                    self.cursor = (self.cursor.0, self.cursor.1 + 1);
                    self.storage.push(*peeked.unwrap());
                    self.source.next();
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

                self.storage = String::new();
                self.token_start = None;
                return Some(token);
            }

            Some(character) => Some(Token::new(
                Kind::Error,
                self.cursor,
                Some(Value::String(format!("Unexpected character {}", character))),
            )),

            None => Some(Token::new(Kind::Eof, self.cursor, None)),
        }
    }
}
