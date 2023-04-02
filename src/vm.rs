use std::env::var_os;
use std::fmt::Debug;

use crate::chunk;
use crate::compiler::Compiler;
use crate::error::{error_out, ErrorContext, LoxError};
use crate::op;
use crate::value;

pub struct VM {
    codes: chunk::Chunk<usize>,
    stack: Vec<value::Value>,
    constants: chunk::Chunk<value::Value>,
    lines: chunk::Chunk<usize>,
}

#[derive(Debug)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
    CliError,
}

impl VM {
    pub fn new() -> VM {
        VM {
            codes: chunk::Chunk::new(),
            stack: vec![],
            constants: chunk::Chunk::new(),
            lines: chunk::Chunk::new(),
        }
    }

    pub fn interpret(&mut self, source: String) -> InterpretResult {
        let mut compiler = Compiler::new(self, &source);
        compiler.compile();
        self.run();
        InterpretResult::Ok
    }

    pub fn run(&mut self) -> InterpretResult {
        let debug = if var_os("DEBUG").is_some() {
            true
        } else {
            false
        };

        println!("{:?}", self);

        let cloned_codes = self.codes.clone();
        let mut iterator = cloned_codes.into_iter().peekable().enumerate();
        while let Some((_offset, current)) = iterator.next() {
            let op_code = op::OpCode::from(*current as u8);
            if debug {
                println!("\n{:?}", op_code);
                println!("{:?}\n", self.stack);
            }
            match op_code {
                op::OpCode::Return => return InterpretResult::Ok,

                op::OpCode::Constant => {
                    let Some((_, address)) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(constant) = self.get_constant(*address) else {
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_push(constant.clone());
                }

                op::OpCode::Negate => {
                    let Some(value::Value::Double(value)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_push(value::Value::Double(-value));
                }

                op::OpCode::Add => {
                    let Some(value::Value::Double(right)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(value::Value::Double(left)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };

                    self.stack_push(value::Value::Double(left + right))
                }

                op::OpCode::Multiply => {
                    let Some(value::Value::Double(right)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(value::Value::Double(left)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_push(value::Value::Double(left * right))
                }

                op::OpCode::Divide => {
                    let Some(value::Value::Double(right)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(value::Value::Double(left)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_push(value::Value::Double(left / right))
                }

                op::OpCode::Print => {
                    let Some(value::Value::Double(value)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    println!("{:?}", value);
                }

                op::OpCode::Pop => {
                    self.stack_pop();
                }

                op::OpCode::Nil => {
                    self.stack_push(value::Value::Nil);
                }

                op::OpCode::DefGlobal => {}

                op::OpCode::Invalid => return InterpretResult::CompileError,
            }
        }

        InterpretResult::Ok
    }

    pub fn add_op(&mut self, op: op::OpCode, line: usize) {
        self.lines.add(line);
        self.codes.add(op as usize);
    }

    pub fn add_variable(&mut self, name: value::Value) {
        let address = self.constants.add(name);
        self.add_op(op::OpCode::DefGlobal, 0);
        self.codes.add(address);
    }

    pub fn add_constant(&mut self, constant: value::Value) {
        let address = self.constants.add(constant);
        self.add_op(op::OpCode::Constant, 0);
        self.codes.add(address);
    }

    pub fn stack_push(&mut self, value: value::Value) {
        self.stack.push(value);
    }

    pub fn stack_pop(&mut self) -> Option<value::Value> {
        self.stack.pop()
    }

    fn get_constant(&self, address: usize) -> Option<&value::Value> {
        self.constants.get(address)
    }

    fn get_line(&self, address: usize) -> Option<usize> {
        self.lines.get(address).map(|line| *line)
    }
}

impl Debug for VM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iterator = self.codes.into_iter().peekable().enumerate();
        while let Some((offset, current)) = iterator.next() {
            let op_code = op::OpCode::from(*current as u8);
            let line = self.get_line(offset);
            let string_offset = format!("{:0>4}", offset);
            if op_code.has_parameter() {
                let Some((_, address)) = iterator.next() else {
                    error_out(
                        LoxError::new(
                            format!("No parameter for op {:?}",op_code).as_str(),
                        ErrorContext::Runtime,
                        line
                        )
                    );  
                    break;
                };
                let Some(constant) = self.get_constant(*address) else {
                    error_out(
                        LoxError::new(
                                format!("Empty parameter for op {:?}",op_code).as_str(),
                            ErrorContext::Runtime,
                            line
                            )
                    );  
                    break;
                };
                writeln!(f, "{}   {:?}    {:?}", string_offset, op_code, constant)?;
                continue;
            }
            writeln!(f, "{}   {:?}", string_offset, op_code)?;
        }
        Ok(())
    }
}
