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
    Concat,
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
    Expands,
    False,
    Fun,
    For,
    If,
    Let,
    Nil,
    Not,
    Or,
    // Print,
    // PrintLn,
    Return,
    Super,
    This,
    True,
    While,

    Error,
    Eof,
}

impl Kind {
    pub(crate) fn keyword_equivalent(candidate: &String) -> Option<Kind> {
        match candidate.as_str() {
            "and" => Some(Self::And),
            "class" => Some(Self::Class),
            "else" => Some(Self::Else),
            "expands" => Some(Self::Expands),
            "false" => Some(Self::False),
            "fun" => Some(Self::Fun),
            "for" => Some(Self::For),
            "if" => Some(Self::If),
            "nil" => Some(Self::Nil),
            "not" => Some(Self::Not),
            "or" => Some(Self::Or),
            // "print" => Some(Self::Print),
            // "println" => Some(Self::PrintLn),
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
