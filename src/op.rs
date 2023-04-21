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
    Negate,
    Return,
    Greater,
    GetLocal,
    Constant,
    Multiply,
    NotEqual,
    SetLocal,
    DefGlobal,
    GetGlobal,
    LessEqual,
    SetGlobal,
    ClearScope,
    JumpIfFalse,
    GetCaptured,
    MakeClosure,
    SetCaptured,
    GreaterEqual,

    Invalid,
}

impl OpCode {
    pub(crate) fn params(&self) -> u8 {
        match self {
            Self::Constant | Self::GetLocal | Self::SetLocal => 1,
            Self::Loop
            | Self::DefGlobal
            | Self::GetGlobal
            | Self::SetGlobal
            | Self::ClearScope
            | Self::GetCaptured
            | Self::MakeClosure
            | Self::SetCaptured => 2,
            Self::Call => 6,
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
            12 => Self::Negate,
            13 => Self::Return,
            14 => Self::Greater,
            15 => Self::GetLocal,
            16 => Self::Constant,
            17 => Self::Multiply,
            18 => Self::NotEqual,
            19 => Self::SetLocal,
            20 => Self::DefGlobal,
            21 => Self::GetGlobal,
            22 => Self::LessEqual,
            23 => Self::SetGlobal,
            24 => Self::ClearScope,
            25 => Self::JumpIfFalse,
            26 => Self::GetCaptured,
            27 => Self::MakeClosure,
            28 => Self::SetCaptured,
            29 => Self::GreaterEqual,
            _ => Self::Invalid,
        }
    }
}

impl From<OpCode> for u8 {
    fn from(val: OpCode) -> Self {
        match val {
            OpCode::Add => 0,
            OpCode::Nil => 1,
            OpCode::Not => 2,
            OpCode::Pop => 3,
            OpCode::Rem => 4,
            OpCode::Call => 5,
            OpCode::Jump => 6,
            OpCode::Less => 7,
            OpCode::Loop => 8,
            OpCode::Equal => 9,
            OpCode::Concat => 10,
            OpCode::Divide => 11,
            OpCode::Negate => 12,
            OpCode::Return => 13,
            OpCode::Greater => 14,
            OpCode::GetLocal => 15,
            OpCode::Constant => 16,
            OpCode::Multiply => 17,
            OpCode::NotEqual => 18,
            OpCode::SetLocal => 19,
            OpCode::DefGlobal => 20,
            OpCode::GetGlobal => 21,
            OpCode::LessEqual => 22,
            OpCode::SetGlobal => 23,
            OpCode::ClearScope => 24,
            OpCode::JumpIfFalse => 25,
            OpCode::GetCaptured => 26,
            OpCode::MakeClosure => 27,
            OpCode::SetCaptured => 28,
            OpCode::GreaterEqual => 29,
            OpCode::Invalid => 255,
        }
    }
}
