use std::cmp::Ordering;

use crate::function::Function;

#[derive(Debug, Clone)]
pub(crate) enum Value {
    Nil,
    Double(f64),
    Boolean(bool),
    String(String),
    Function(Function),
}

#[derive(PartialEq)]
enum Type {
    Nil,
    Double,
    String,
    Boolean,
    Function,
}

impl Value {
    fn get_type(&self) -> Type {
        match self {
            Self::Nil => Type::Nil,
            Self::Double(_) => Type::Double,
            Self::String(_) => Type::String,
            Self::Boolean(_) => Type::Boolean,
            Self::Function(_) => Type::Function,
        }
    }

    fn to_string(self) -> String {
        self.into()
    }
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

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self.get_type() == other.get_type() {
            false => false,
            true => self.clone().to_string() == other.clone().to_string(),
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_type = self.get_type();
        let other_type = other.get_type();
        match self == other {
            true => Some(Ordering::Equal),
            false => match self_type == other_type {
                true => match self_type {
                    Type::Nil => Some(Ordering::Equal),
                    _ => match (self, other) {
                        (Self::Function(_), Self::Function(_)) => None,
                        (Self::String(v1), Self::String(v2)) => v1.partial_cmp(v2),
                        (Self::Double(v1), Self::Double(v2)) => v1.partial_cmp(v2),
                        (Self::Boolean(v1), Self::Boolean(v2)) => v1.partial_cmp(v2),
                        _ => None,
                    },
                },
                false => match (self_type, other_type) {
                    (Type::Nil, _) => Some(Ordering::Less),
                    (_, Type::Nil) => Some(Ordering::Greater),
                    (Type::Function, _) => Some(Ordering::Greater),
                    (_, Type::Function) => Some(Ordering::Less),
                    (Type::String, _) => Some(Ordering::Greater),
                    (_, Type::String) => Some(Ordering::Less),
                    (Type::Double, _) => Some(Ordering::Greater),
                    (_, Type::Double) => Some(Ordering::Less),
                    _ => None,
                },
            },
        }
    }
}
