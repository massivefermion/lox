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
            OpCode::GetVar => 12,
            OpCode::Negate => 13,
            OpCode::Return => 14,
            OpCode::Greater => 15,
            OpCode::Constant => 16,
            OpCode::Multiply => 17,
            OpCode::NotEqual => 18,
            OpCode::SetLocal => 19,
            OpCode::DefGlobal => 20,
            OpCode::LessEqual => 21,
            OpCode::SetGlobal => 22,
            OpCode::ClearScope => 23,
            OpCode::JumpIfFalse => 24,
            OpCode::GreaterEqual => 25,
            OpCode::Invalid => 255,
        }
    }
}
