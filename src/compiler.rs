use crate::error::{ErrorContext, LoxError};
use crate::op::OpCode;
use crate::scanner::Scanner;
use crate::token::Kind;
use crate::vm::VM;
use std::iter::Peekable;
pub struct Compiler<'a> {
    scanner: Peekable<Scanner<'a>>,
    vm: &'a mut VM,
    errors: Vec<LoxError>,
}

impl<'a> Compiler<'a> {
    pub fn new(vm: &'a mut VM, source: &'a String) -> Compiler<'a> {
        Compiler {
            scanner: Scanner::new(source).peekable(),
            vm,
            errors: vec![],
        }
    }

    pub fn compile(&mut self) {
        loop {
            let peeked = self.scanner.peek();
            if peeked.is_none() || peeked.unwrap().kind() == Kind::Eof {
                break;
            }
            self.compile_declaration();
        }

        self.vm.add_op(OpCode::Return, 0);
        match self.errors.len() {
            0 => (),
            _ => self.errors.iter().for_each(|error| {
                eprintln!("{}", error);
            }),
        }
    }

    fn compile_declaration(&mut self) {
        match self.scanner.peek() {
            Some(token) => match token.kind() {
                Kind::Let => {
                    self.scanner.next();
                    self.compile_let();
                }

                _ => self.compile_statement(),
            },

            None => {
                self.errors.push(LoxError::new(
                    "Unexpected end of script",
                    ErrorContext::Compile,
                    None,
                ));
            }
        }
    }

    fn compile_let(&mut self) {
        match self.scanner.peek() {
            Some(token) => match token.kind() {
                Kind::Identifier => {
                    let name: String = token.value().unwrap().into();
                    self.scanner.next();

                    match self.scanner.peek() {
                        Some(token) => match token.kind() {
                            Kind::Equal => {}

                            _ => {}
                        },

                        None => {
                            self.errors.push(LoxError::new(
                                "Unexpected end of script",
                                ErrorContext::Compile,
                                None,
                            ));
                        }
                    }
                }

                _ => {
                    self.errors.push(LoxError::new(
                        "Unexpected token",
                        ErrorContext::Compile,
                        None,
                    ));
                }
            },

            None => {
                self.errors.push(LoxError::new(
                    "Unexpected end of script",
                    ErrorContext::Compile,
                    None,
                ));
            }
        }
    }

    fn compile_statement(&mut self) {
        match self.scanner.peek() {
            Some(token) => match token.kind() {
                Kind::Print => {
                    self.scanner.next();
                    self.compile_print();
                }
                _ => {
                    self.compile_expression();
                    self.expect_semicolon();
                    self.vm.add_op(OpCode::Pop, 0);
                }
            },
            None => {
                self.errors.push(LoxError::new(
                    "Unexpected end of script",
                    ErrorContext::Compile,
                    None,
                ));
            }
        }
    }

    fn compile_print(&mut self) {
        self.compile_expression();
        self.expect_semicolon();
        self.vm.add_op(OpCode::Print, 0);
    }

    fn compile_expression(&mut self) {
        self.compile_term();
        loop {
            let is_negated = match self.scanner.peek() {
                Some(token) => match token.kind() {
                    Kind::Minus => {
                        self.scanner.next();
                        true
                    }

                    Kind::Plus => {
                        self.scanner.next();
                        false
                    }

                    _ => return,
                },

                None => {
                    self.errors.push(LoxError::new(
                        "Unexpected end of script",
                        ErrorContext::Compile,
                        None,
                    ));
                    false
                }
            };

            self.compile_term();
            if is_negated {
                self.vm.add_op(OpCode::Negate, 0);
            }
            self.vm.add_op(OpCode::Add, 0);
        }
    }

    fn compile_term(&mut self) {
        self.compile_factor();
        loop {
            match self.scanner.peek() {
                Some(token) => match token.kind() {
                    Kind::Star => {
                        self.scanner.next();
                        self.compile_factor();
                        self.vm.add_op(OpCode::Multiply, 0);
                    }

                    Kind::Slash => {
                        self.scanner.next();
                        self.compile_factor();
                        self.vm.add_op(OpCode::Divide, 0);
                    }

                    _ => return,
                },

                None => {
                    self.errors.push(LoxError::new(
                        "Unexpected end of script",
                        ErrorContext::Compile,
                        None,
                    ));
                }
            };
        }
    }

    fn compile_factor(&mut self) {
        match self.scanner.peek() {
            Some(token) => match token.kind() {
                Kind::LeftParen => {
                    self.scanner.next();
                    self.compile_expression();
                    match self.scanner.peek() {
                        Some(token) => match token.kind() {
                            Kind::RightParen => {
                                self.scanner.next();
                            }

                            _ => {
                                self.errors.push(LoxError::new(
                                    "Unexpected token",
                                    ErrorContext::Compile,
                                    None,
                                ));
                            }
                        },

                        None => {
                            self.errors.push(LoxError::new(
                                "Unexpected end of script",
                                ErrorContext::Compile,
                                None,
                            ));
                        }
                    }
                }

                Kind::Nil => {
                    self.vm.add_op(OpCode::Nil, 0);
                }

                Kind::Number | Kind::String => {
                    self.vm.add_op(OpCode::Constant, 0);
                    self.vm.add_constant(token.value().unwrap());
                    self.scanner.next();
                }

                _ => {
                    self.errors.push(LoxError::new(
                        "Unexpected token",
                        ErrorContext::Compile,
                        None,
                    ));
                }
            },

            None => {
                self.errors.push(LoxError::new(
                    "Unexpected end of script",
                    ErrorContext::Compile,
                    None,
                ));
            }
        }
    }

    fn expect_semicolon(&mut self) {
        match self.scanner.peek() {
            Some(token) => match token.kind() {
                Kind::Semicolon => {
                    self.scanner.next();
                }

                _ => {
                    self.errors.push(LoxError::new(
                        "Unexpected token",
                        ErrorContext::Compile,
                        None,
                    ));
                }
            },

            None => {
                self.errors.push(LoxError::new(
                    "Unexpected end of script",
                    ErrorContext::Compile,
                    None,
                ));
            }
        }
    }
}
