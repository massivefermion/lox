use std::error::Error;
use std::fmt;
use std::process::{ExitCode, Termination};

#[derive(Debug)]
pub(crate) enum ErrorContext {
    Compile,
}

#[derive(Debug)]
pub(crate) struct LoxError {
    message: String,
    line: Option<usize>,
    context: ErrorContext,
}

impl LoxError {
    pub(crate) fn new(msg: &str, context: ErrorContext, line: Option<usize>) -> LoxError {
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

// pub(crate) fn error_out<E: Error>(error: E) {
//     eprintln!("{}", error);
//     exit(1);
// }

#[derive(Debug)]
pub(crate) enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
    CliError,
}

impl Termination for InterpretResult {
    fn report(self) -> std::process::ExitCode {
        match self {
            Self::Ok => ExitCode::SUCCESS,
            _ => ExitCode::FAILURE,
        }
    }
}
