use crate::value::Value;

#[derive(Debug, Clone)]
pub(crate) struct Token {
    kind: Kind,
    // start: (usize, usize),
    value: Option<Value>,
}

impl Token {
    pub(crate) fn new(kind: Kind, _start: (usize, usize), value: Option<Value>) -> Token {
        Token { kind, value }
    }

    pub(crate) fn kind(&self) -> Kind {
        self.kind.clone()
    }

    pub(crate) fn value(&self) -> Option<Value> {
        self.value.clone()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum Kind {
    // Single-character tokens.
    Dot,
    Plus,
    Star,
    Minus,
    Comma,
    Slash,
    Percent,
    Semicolon,
    LeftBrace,
    LeftParen,
    RightParen,
    RightBrace,

    // One or two character tokens.
    Less,
    Equal,
    Concat,
    Greater,
    BangEqual,
    LessEqual,
    EqualEqual,
    GreaterEqual,

    // Literals.
    Number,
    String,
    Identifier,

    // Keywords.
    If,
    Or,
    And,
    For,
    Fun,
    Let,
    Nil,
    Not,
    Else,
    Enum,
    This,
    True,
    Class,
    False,
    Super,
    While,
    Return,
    Expands,

    Error,
    Eof,
}

impl Kind {
    pub(crate) fn keyword_equivalent(candidate: &str) -> Option<Kind> {
        match candidate {
            "if" => Some(Self::If),
            "or" => Some(Self::Or),
            "and" => Some(Self::And),
            "for" => Some(Self::For),
            "fun" => Some(Self::Fun),
            "let" => Some(Self::Let),
            "nil" => Some(Self::Nil),
            "not" => Some(Self::Not),
            "else" => Some(Self::Else),
            "enum" => Some(Self::Enum),
            "this" => Some(Self::This),
            "true" => Some(Self::True),
            "class" => Some(Self::Class),
            "false" => Some(Self::False),
            "super" => Some(Self::Super),
            "while" => Some(Self::While),
            "return" => Some(Self::Return),
            "expands" => Some(Self::Expands),
            _ => None,
        }
    }
}
