#[derive(Debug)]
pub enum OpCode {
    Constant,
    Return,
    Negate,
    Add,
    Multiply,
    Divide,
    Print,
    Pop,
    Nil,
    DefGlobal,
    Invalid,
}

impl OpCode {
    pub fn has_parameter(&self) -> bool {
        match self {
            OpCode::Constant => true,
            _ => false,
        }
    }
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            0 => OpCode::Constant,
            1 => OpCode::Return,
            2 => OpCode::Negate,
            3 => OpCode::Add,
            4 => OpCode::Multiply,
            5 => OpCode::Divide,
            6 => OpCode::Print,
            7 => OpCode::Pop,
            8 => OpCode::Nil,
            9 => OpCode::DefGlobal,
            _ => OpCode::Invalid,
        }
    }
}

impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        match self {
            OpCode::Constant => 0,
            OpCode::Return => 1,
            OpCode::Negate => 2,
            OpCode::Add => 3,
            OpCode::Multiply => 4,
            OpCode::Divide => 5,
            OpCode::Print => 6,
            OpCode::Pop => 7,
            OpCode::Nil => 8,
            OpCode::DefGlobal => 9,
            OpCode::Invalid => 255,
        }
    }
}
