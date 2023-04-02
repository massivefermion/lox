use std::error::Error;
use std::fmt;
use std::process::exit;

#[derive(Debug)]
pub enum ErrorContext {
    Runtime,
    Compile,
}

#[derive(Debug)]
pub struct LoxError {
    line: Option<usize>,
    message: String,
    context: ErrorContext,
}

impl LoxError {
    pub fn new(msg: &str, context: ErrorContext, line: Option<usize>) -> LoxError {
        LoxError {
            line,
            context,
            message: msg.to_string(),
        }
    }
}

impl fmt::Display for LoxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.line {
            None => write!(f, "{:?} error: {}", self.context, self.message),
            Some(line) => {
                write!(
                    f,
                    "{:?} error: {} at line {}",
                    self.context, self.message, line
                )
            }
        }
    }
}

impl Error for LoxError {
    fn description(&self) -> &str {
        &self.message
    }
}

pub fn error_out<E: Error>(error: E) {
    eprintln!("{}", error);
    exit(1);
}
