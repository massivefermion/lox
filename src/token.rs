use crate::value;

pub struct Token {
    kind: Kind,
    start: (usize, usize),
    value: Option<value::Value>,
}

impl Token {
    pub fn new(kind: Kind, start: (usize, usize), value: Option<value::Value>) -> Token {
        Token { kind, start, value }
    }

    pub fn kind(&self) -> Kind {
        self.kind.clone()
    }

    pub fn value(&self) -> Option<value::Value> {
        self.value.clone()
    }
}

#[derive(Clone, PartialEq)]
pub enum Kind {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreateEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Let,
    Nil,
    Not,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    While,

    Error,
    Eof,
}

impl Kind {
    pub fn keyword_equivalent(candidate: &String) -> Option<Kind> {
        match candidate.as_str() {
            "and" => Some(Self::And),
            "class" => Some(Self::Class),
            "else" => Some(Self::Else),
            "false" => Some(Self::False),
            "fun" => Some(Self::Fun),
            "for" => Some(Self::For),
            "if" => Some(Self::If),
            "nil" => Some(Self::Nil),
            "not" => Some(Self::Not),
            "or" => Some(Self::Or),
            "print" => Some(Self::Print),
            "return" => Some(Self::Return),
            "super" => Some(Self::Super),
            "this" => Some(Self::This),
            "true" => Some(Self::True),
            "let" => Some(Self::Let),
            "while" => Some(Self::While),
            _ => None,
        }
    }
}
