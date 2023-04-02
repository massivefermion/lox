#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Double(f64),
    String(String),
}

impl Into<f64> for Value {
    fn into(self) -> f64 {
        match self {
            Value::Double(value) => value,
            _ => panic!("value is not a double"),
        }
    }
}

impl Into<String> for Value {
    fn into(self) -> String {
        match self {
            Value::String(value) => value,
            _ => panic!("value is not a string"),
        }
    }
}
