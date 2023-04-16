#[derive(Debug, PartialEq)]
pub(crate) enum OpCode {
    Constant,
    Return,
    Negate,
    Add,
    Multiply,
    Divide,
    Pop,
    Nil,
    DefGlobal,
    GetGlobal,
    SetGlobal,
    GetLocal,
    SetLocal,
    JumpIfFalse,
    Jump,
    Not,
    Concat,
    Call,
    ClearScope,
    EqualEqual,
    BangEqual,
    GreaterEqual,
    Greater,
    LessEqual,
    Less,
    Invalid,
    // Print,
    // NewLine,
}

impl OpCode {
    pub(crate) fn params(&self) -> u8 {
        match self {
            Self::Constant
            | Self::SetLocal
            | Self::GetLocal
            | Self::DefGlobal
            | Self::SetGlobal
            | Self::GetGlobal
            | Self::ClearScope => 1,
            Self::Call => 3,
            _ => 0,
        }
    }
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Constant,
            1 => Self::Return,
            2 => Self::Negate,
            3 => Self::Add,
            4 => Self::Multiply,
            5 => Self::Divide,
            6 => Self::Pop,
            7 => Self::Nil,
            8 => Self::DefGlobal,
            9 => Self::GetGlobal,
            10 => Self::SetGlobal,
            11 => Self::GetLocal,
            12 => Self::SetLocal,
            13 => Self::JumpIfFalse,
            14 => Self::Jump,
            15 => Self::Not,
            16 => Self::Concat,
            17 => Self::Call,
            18 => Self::ClearScope,
            19 => Self::EqualEqual,
            20 => Self::BangEqual,
            21 => Self::GreaterEqual,
            22 => Self::Greater,
            23 => Self::LessEqual,
            24 => Self::Less,
            _ => Self::Invalid,
        }
    }
}

impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        match self {
            Self::Constant => 0,
            Self::Return => 1,
            Self::Negate => 2,
            Self::Add => 3,
            Self::Multiply => 4,
            Self::Divide => 5,
            Self::Pop => 6,
            Self::Nil => 7,
            Self::DefGlobal => 8,
            Self::GetGlobal => 9,
            Self::SetGlobal => 10,
            Self::GetLocal => 11,
            Self::SetLocal => 12,
            Self::JumpIfFalse => 13,
            Self::Jump => 14,
            Self::Not => 15,
            Self::Concat => 16,
            Self::Call => 17,
            Self::ClearScope => 18,
            Self::EqualEqual => 19,
            Self::BangEqual => 20,
            Self::GreaterEqual => 21,
            Self::Greater => 22,
            Self::LessEqual => 23,
            Self::Less => 24,
            Self::Invalid => 255,
        }
    }
}
