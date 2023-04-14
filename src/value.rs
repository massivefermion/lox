use crate::function::Function;

#[derive(Debug, Clone)]
pub(crate) enum Value {
    Nil,
    Double(f64),
    Boolean(bool),
    String(String),
    Function(Function),
}

impl Into<f64> for Value {
    fn into(self) -> f64 {
        match self {
            Self::Double(value) => value,
            _ => panic!("value is not a number"),
        }
    }
}

impl Into<String> for Value {
    fn into(self) -> String {
        match self {
            Self::Nil => "nil".to_string(),
            Self::String(value) => value,
            Self::Boolean(true) => "true".to_string(),
            Self::Boolean(false) => "false".to_string(),
            Self::Double(value) => value.to_string(),
            Self::Function(value) => value.to_string(),
        }
    }
}

impl Into<bool> for Value {
    fn into(self) -> bool {
        match self {
            Self::Boolean(value) => value,
            _ => panic!("value is not a boolean"),
        }
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}
