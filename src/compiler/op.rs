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
    JumpIfFalse,
    GetCaptured,
    MakeClosure,
    GreaterEqual,
    Subtract,
    JumpBack,

    Invalid,
}

impl OpCode {
    pub(crate) fn params(&self) -> u8 {
        match self {
            Self::Constant | Self::GetLocal | Self::SetLocal | Self::JumpBack => 1,
            Self::DefGlobal
            | Self::GetGlobal
            | Self::SetGlobal
            | Self::MakeClosure
            | Self::GetCaptured => 2,
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
            8 => Self::Equal,
            9 => Self::Concat,
            10 => Self::Divide,
            11 => Self::Negate,
            12 => Self::Return,
            13 => Self::Greater,
            14 => Self::GetLocal,
            15 => Self::Constant,
            16 => Self::Multiply,
            17 => Self::NotEqual,
            18 => Self::SetLocal,
            19 => Self::DefGlobal,
            20 => Self::GetGlobal,
            21 => Self::LessEqual,
            22 => Self::SetGlobal,
            23 => Self::JumpIfFalse,
            24 => Self::GetCaptured,
            25 => Self::MakeClosure,
            26 => Self::GreaterEqual,
            27 => Self::Subtract,
            28 => Self::JumpBack,
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
            OpCode::Equal => 8,
            OpCode::Concat => 9,
            OpCode::Divide => 10,
            OpCode::Negate => 11,
            OpCode::Return => 12,
            OpCode::Greater => 13,
            OpCode::GetLocal => 14,
            OpCode::Constant => 15,
            OpCode::Multiply => 16,
            OpCode::NotEqual => 17,
            OpCode::SetLocal => 18,
            OpCode::DefGlobal => 19,
            OpCode::GetGlobal => 20,
            OpCode::LessEqual => 21,
            OpCode::SetGlobal => 22,
            OpCode::JumpIfFalse => 23,
            OpCode::GetCaptured => 24,
            OpCode::MakeClosure => 25,
            OpCode::GreaterEqual => 26,
            OpCode::Subtract => 27,
            OpCode::JumpBack => 28,
            OpCode::Invalid => 255,
        }
    }
}
