#[derive(Debug, PartialEq)]
pub(crate) enum OpCode {
    Add,
    Nil,
    Not,
    Pop,
    Rem,
    Call,
    Jump,
    Less,
    Loop,
    Equal,
    Concat,
    Divide,
    GetVar,
    Negate,
    Return,
    Greater,
    Constant,
    Multiply,
    NotEqual,
    SetLocal,
    DefGlobal,
    LessEqual,
    SetGlobal,
    ClearScope,
    JumpIfFalse,
    GreaterEqual,

    Invalid,
}

impl OpCode {
    pub(crate) fn params(&self) -> u8 {
        match self {
            Self::Loop
            | Self::Constant
            | Self::SetLocal
            | Self::DefGlobal
            | Self::SetGlobal
            | Self::ClearScope => 1,
            Self::GetVar => 2,
            Self::Call => 3,
            _ => 0,
        }
    }
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Add,
            1 => Self::Nil,
            2 => Self::Not,
            3 => Self::Pop,
            4 => Self::Rem,
            5 => Self::Call,
            6 => Self::Jump,
            7 => Self::Less,
            8 => Self::Loop,
            9 => Self::Equal,
            10 => Self::Concat,
            11 => Self::Divide,
            12 => Self::GetVar,
            13 => Self::Negate,
            14 => Self::Return,
            15 => Self::Greater,
            16 => Self::Constant,
            17 => Self::Multiply,
            18 => Self::NotEqual,
            19 => Self::SetLocal,
            20 => Self::DefGlobal,
            21 => Self::LessEqual,
            22 => Self::SetGlobal,
            23 => Self::ClearScope,
            24 => Self::JumpIfFalse,
            25 => Self::GreaterEqual,
            _ => Self::Invalid,
        }
    }
}

impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        match self {
            Self::Add => 0,
            Self::Nil => 1,
            Self::Not => 2,
            Self::Pop => 3,
            Self::Rem => 4,
            Self::Call => 5,
            Self::Jump => 6,
            Self::Less => 7,
            Self::Loop => 8,
            Self::Equal => 9,
            Self::Concat => 10,
            Self::Divide => 11,
            Self::GetVar => 12,
            Self::Negate => 13,
            Self::Return => 14,
            Self::Greater => 15,
            Self::Constant => 16,
            Self::Multiply => 17,
            Self::NotEqual => 18,
            Self::SetLocal => 19,
            Self::DefGlobal => 20,
            Self::LessEqual => 21,
            Self::SetGlobal => 22,
            Self::ClearScope => 23,
            Self::JumpIfFalse => 24,
            Self::GreaterEqual => 25,
            Self::Invalid => 255,
        }
    }
}
