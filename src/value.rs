use std::cmp::Ordering;
use std::fmt::Display;

use crate::function::Function;

#[derive(Debug, Clone)]
pub(crate) enum Value {
    Nil,
    Number(f64),
    Boolean(bool),
    String(String),
    Function((usize, Option<Function>)),
}

#[derive(PartialEq)]
enum Type {
    Nil,
    Number,
    String,
    Boolean,
    Function,
}

impl Value {
    fn get_type(&self) -> Type {
        match self {
            Self::Nil => Type::Nil,
            Self::Number(_) => Type::Number,
            Self::String(_) => Type::String,
            Self::Boolean(_) => Type::Boolean,
            Self::Function(_) => Type::Function,
        }
    }
}

impl From<Value> for f64 {
    fn from(value: Value) -> Self {
        match value {
            Value::Number(value) => value,
            _ => panic!("value is not a number"),
        }
    }
}

impl From<Value> for i128 {
    fn from(value: Value) -> Self {
        let value: f64 = value.into();
        value as i128
    }
}

impl From<Value> for String {
    fn from(val: Value) -> Self {
        match val {
            Value::Nil => "nil".to_string(),
            Value::String(value) => value,
            Value::Boolean(true) => "true".to_string(),
            Value::Boolean(false) => "false".to_string(),
            Value::Number(value) => value.to_string(),
            Value::Function((_, value)) => match value {
                Some(function) => function.to_string(),
                None => String::new(),
            },
        }
    }
}

impl From<Value> for bool {
    fn from(value: Value) -> Self {
        match value {
            Value::Boolean(value) => value,
            _ => panic!("value is not a boolean"),
        }
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self.clone()))?;
        Ok(())
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self.get_type() == other.get_type() {
            false => false,
            true => self.clone().to_string() == other.clone().to_string(),
        }
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
                        (Self::Number(v1), Self::Number(v2)) => v1.partial_cmp(v2),
                        (Self::Boolean(v1), Self::Boolean(v2)) => v1.partial_cmp(v2),
                        _ => None,
                    },
                },
                false => match (self_type, other_type) {
                    (Type::Nil, _) => Some(Ordering::Less),
                    (_, Type::Nil) => Some(Ordering::Greater),
                    (_, Type::Function) => Some(Ordering::Less),
                    (Type::Function, _) => Some(Ordering::Greater),
                    (_, Type::String) => Some(Ordering::Less),
                    (Type::String, _) => Some(Ordering::Greater),
                    (_, Type::Number) => Some(Ordering::Less),
                    (Type::Number, _) => Some(Ordering::Greater),
                    _ => None,
                },
            },
        }
    }
}
