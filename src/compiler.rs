use std::iter::Peekable;

use crate::error::{ErrorContext, InterpretResult, LoxError};
use crate::function::Function;
use crate::nif::resolve_nif;
use crate::op::OpCode;
use crate::scanner::Scanner;
use crate::token::Kind;
use crate::value::Value;
use crate::vm::VM;

pub(crate) struct Compiler<'a> {
    vm: &'a mut VM,
    scope_depth: u128,
    errors: Vec<LoxError>,
    functions: Vec<Function>,
    locals: Vec<(String, u128)>,
    scanner: Peekable<Scanner<'a>>,
}

impl<'a> Compiler<'a> {
    pub(crate) fn new(vm: &'a mut VM, function: Function, source: &'a String) -> Compiler<'a> {
        Compiler {
            vm,
            functions: vec![function],
            errors: vec![],
            locals: vec![],
            scope_depth: 0,
            scanner: Scanner::new(source).peekable(),
        }
    }

    pub(crate) fn compile(&mut self) -> Result<Function, InterpretResult> {
        loop {
            self.compile_declaration();
            match self.scanner.peek().unwrap().kind() {
                Kind::Eof => break,
                _ => continue,
            }
        }

        match self.errors.len() {
            0 => return Ok(self.function().clone()),
            _ => {
                self.errors.iter().for_each(|e| eprintln!("{}", e));
                Err(InterpretResult::CompileError)
            }
        }
    }

    fn compile_declaration(&mut self) {
        match self.scanner.peek() {
            Some(token) => match token.kind() {
                Kind::Let => {
                    self.scanner.next();
                    self.compile_let();
                }

                Kind::Fun => {
                    self.scanner.next();
                    self.compile_fun();
                }

                Kind::Return => {
                    self.scanner.next();
                    self.compile_expression();
                    self.expect(Kind::Semicolon);
                    self.function().add_op(OpCode::Return);
                    self.function().has_return();
                }

                _ => self.compile_statement(true),
            },

            None => self.errors.push(LoxError::new(
                "Unexpected end of script",
                ErrorContext::Compile,
                None,
            )),
        }
    }

    fn compile_let(&mut self) {
        match self.scanner.peek() {
            Some(token) => match token.kind() {
                Kind::Identifier => {
                    let variable_name = token.value().unwrap();
                    self.scanner.next();

                    match self.scanner.peek() {
                        Some(token) => match token.kind() {
                            Kind::Equal => {
                                self.scanner.next();
                                self.compile_expression();
                            }

                            _ => self.function().add_op(OpCode::Nil),
                        },

                        None => self.errors.push(LoxError::new(
                            "Unexpected end of script",
                            ErrorContext::Compile,
                            None,
                        )),
                    }
                    self.expect(Kind::Semicolon);

                    match self.scope_depth {
                        0 => {
                            self.function().add_op(OpCode::DefGlobal);
                            self.add_constant(variable_name);
                        }

                        _ => {
                            let variable_name: String = variable_name.into();

                            if variable_name != "_".to_string() {
                                match self.locals.iter().find(|(name, scope)| {
                                    *name == variable_name && *scope == self.scope_depth
                                }) {
                                    Some(_) => self.errors.push(LoxError::new(
                                        format!("Variable {:?} is already defined", variable_name)
                                            .as_str(),
                                        ErrorContext::Compile,
                                        None,
                                    )),
                                    None => {
                                        self.locals.push((variable_name.into(), self.scope_depth));
                                    }
                                }
                            }
                        }
                    }
                }

                _ => self.errors.push(LoxError::new(
                    format!("unexpected {:?} #1", token).as_str(),
                    ErrorContext::Compile,
                    None,
                )),
            },

            None => self.errors.push(LoxError::new(
                "Unexpected end of script",
                ErrorContext::Compile,
                None,
            )),
        }
    }

    fn compile_fun(&mut self) {
        match self.scanner.next() {
            Some(token) if token.kind() == Kind::Identifier => {
                let function_name: String = token.value().unwrap().into();

                if self.vm.function_exists(self.scope_depth, &function_name)
                    || resolve_nif(&function_name).is_some()
                {
                    self.errors.push(LoxError::new(
                        format!("Function {} already exists", function_name).as_str(),
                        ErrorContext::Compile,
                        None,
                    ));
                    return;
                }

                self.expect(Kind::LeftParen);
                self.scope_depth += 1;
                let mut arity = 0;

                loop {
                    match self.scanner.next() {
                        Some(token) if token.kind() == Kind::Identifier => {
                            arity += 1;
                            let variable_name: String = token.value().unwrap().into();
                            self.locals.push((variable_name.into(), self.scope_depth));

                            match self.scanner.peek() {
                                Some(token) if token.kind() == Kind::Comma => {
                                    self.scanner.next();
                                    continue;
                                }

                                Some(token) if token.kind() == Kind::RightParen => {
                                    self.scanner.next();
                                    break;
                                }

                                None => self.errors.push(LoxError::new(
                                    "Unexpected end of script",
                                    ErrorContext::Compile,
                                    None,
                                )),

                                _ => self.errors.push(LoxError::new(
                                    format!("unexpected {:?} #1", token).as_str(),
                                    ErrorContext::Compile,
                                    None,
                                )),
                            }
                        }

                        Some(token) if token.kind() == Kind::RightParen => {
                            break;
                        }

                        None => self.errors.push(LoxError::new(
                            "Unexpected end of script",
                            ErrorContext::Compile,
                            None,
                        )),

                        _ => self.errors.push(LoxError::new(
                            format!("unexpected {:?} #1", token).as_str(),
                            ErrorContext::Compile,
                            None,
                        )),
                    }
                }

                self.new_function(function_name.into(), arity);
                self.compile_statement(false);
                match self.function().has_return() {
                    Some(false) => {
                        self.function().add_op(OpCode::Nil);
                        self.function().add_op(OpCode::Return);
                    }
                    _ => (),
                }
                self.scope_depth -= 1;
                let function = self.functions.pop().unwrap();
                self.vm.add_function(self.scope_depth, function);
            }

            None => self.errors.push(LoxError::new(
                "Unexpected end of script",
                ErrorContext::Compile,
                None,
            )),

            Some(token) => self.errors.push(LoxError::new(
                format!("unexpected {:?} #1", token).as_str(),
                ErrorContext::Compile,
                None,
            )),
        }
    }

    fn compile_statement(&mut self, manage_scope: bool) {
        match self.scanner.peek() {
            // Some(token) if [Kind::Print, Kind::PrintLn].contains(&token.kind()) => {
            //     let kind = token.kind();
            //     self.scanner.next();
            //     self.compile_print(kind == Kind::PrintLn);
            // }
            Some(token) if token.kind() == Kind::If => {
                self.scanner.next();
                self.compile_if();
            }

            Some(token) if token.kind() == Kind::LeftBrace => {
                self.scanner.next();
                if manage_scope {
                    self.scope_depth += 1;
                }
                loop {
                    match self.scanner.peek() {
                        Some(token) => match token.kind() {
                            Kind::RightBrace | Kind::Eof => break,
                            _ => (),
                        },

                        None => self.errors.push(LoxError::new(
                            "Unexpected end of script",
                            ErrorContext::Compile,
                            None,
                        )),
                    }
                    self.compile_declaration();
                }
                self.expect(Kind::RightBrace);

                self.locals = self
                    .locals
                    .iter()
                    .filter(|(_, scope)| *scope != self.scope_depth)
                    .map(|item| item.clone())
                    .collect();

                let scope = self.scope_depth;
                self.function().add_op(OpCode::ClearScope);
                self.function().add_address(scope as usize);

                if manage_scope {
                    self.scope_depth -= 1;
                }
            }

            None => {
                self.errors.push(LoxError::new(
                    "Unexpected end of script",
                    ErrorContext::Compile,
                    None,
                ));
            }

            _ => {
                self.compile_expression();
                self.expect(Kind::Semicolon);
                // self.function().add_op(OpCode::Pop);
                // self.function().add_op(OpCode::Nil);
            }
        }
    }

    // fn compile_print(&mut self, with_new_line: bool) {
    //     self.compile_expression();
    //     self.expect(Kind::Semicolon);
    //     self.function().add_op(OpCode::Print);
    //     if with_new_line {
    //         self.function().add_op(OpCode::NewLine);
    //     }
    // }

    fn compile_if(&mut self) {
        self.compile_expression();
        let jump_address = self.function().add_jump(true);
        self.function().add_op(OpCode::Pop);
        self.compile_statement(true);
        let else_jump_address = self.function().add_jump(false);
        self.function().patch_jump(jump_address);
        self.function().add_op(OpCode::Pop);

        match self.scanner.peek() {
            Some(token) => match token.kind() {
                Kind::Else => {
                    self.scanner.next();
                    self.compile_statement(true);
                }

                _ => (),
            },
            None => (),
        };
        self.function().patch_jump(else_jump_address);
    }

    fn compile_expression(&mut self) {
        self.compile_term(true);
        loop {
            match self.scanner.peek() {
                Some(token) => match token.kind() {
                    Kind::Minus => {
                        self.compile_term(false);
                        self.function().add_op(OpCode::Add);
                    }

                    Kind::Plus => {
                        self.scanner.next();
                        self.compile_term(false);
                        self.function().add_op(OpCode::Add);
                    }

                    Kind::Concat => {
                        self.scanner.next();
                        self.compile_term(false);
                        self.function().add_op(OpCode::Concat)
                    }

                    Kind::Or => {
                        self.scanner.next();
                        let else_jump_address = self.function().add_jump(true);
                        let end_jump_address = self.function().add_jump(false);
                        self.function().patch_jump(else_jump_address);
                        self.function().add_op(OpCode::Pop);
                        self.compile_term(false);
                        self.function().patch_jump(end_jump_address);
                    }

                    _ => return,
                },

                None => self.errors.push(LoxError::new(
                    "Unexpected end of script",
                    ErrorContext::Compile,
                    None,
                )),
            };
        }
    }

    fn compile_term(&mut self, can_assign: bool) {
        self.compile_factor(can_assign);
        loop {
            match self.scanner.peek() {
                Some(token) => match token.kind() {
                    Kind::Star => {
                        self.scanner.next();
                        self.compile_factor(false);
                        self.function().add_op(OpCode::Multiply);
                    }

                    Kind::Slash => {
                        self.scanner.next();
                        self.compile_factor(false);
                        self.function().add_op(OpCode::Divide);
                    }

                    Kind::And => {
                        self.scanner.next();
                        let jump_address = self.function().add_jump(true);
                        self.function().add_op(OpCode::Pop);
                        self.compile_factor(false);
                        self.function().patch_jump(jump_address);
                    }

                    _ => return,
                },

                None => self.errors.push(LoxError::new(
                    "Unexpected end of script",
                    ErrorContext::Compile,
                    None,
                )),
            };
        }
    }

    fn compile_factor(&mut self, can_assign: bool) {
        match self.scanner.next() {
            Some(token) if token.kind() == Kind::Nil => self.function().add_op(OpCode::Nil),
            Some(token) if [Kind::Number, Kind::String].contains(&token.kind()) => {
                self.add_constant(token.value().unwrap())
            }
            Some(token) if token.kind() == Kind::True => self.add_constant(Value::Boolean(true)),
            Some(token) if token.kind() == Kind::False => self.add_constant(Value::Boolean(false)),

            Some(token) if token.kind() == Kind::Not => {
                self.compile_factor(can_assign);
                self.function().add_op(OpCode::Not);
            }

            Some(token) if token.kind() == Kind::Minus => {
                self.compile_factor(can_assign);
                self.function().add_op(OpCode::Negate);
            }

            Some(token) if token.kind() == Kind::LeftParen => {
                self.compile_expression();
                match self.scanner.peek() {
                    Some(token) => match token.kind() {
                        Kind::RightParen => {
                            self.scanner.next();
                        }

                        _ => self.errors.push(LoxError::new(
                            format!("unexpected {:?} #2", token).as_str(),
                            ErrorContext::Compile,
                            None,
                        )),
                    },

                    None => self.errors.push(LoxError::new(
                        "Unexpected end of script",
                        ErrorContext::Compile,
                        None,
                    )),
                }
            }

            Some(token) if token.kind() == Kind::Identifier => {
                let name: String = token.value().unwrap().into();
                let address = self.resolve_local(name);
                match self.scanner.peek() {
                    Some(token) if token.kind() == Kind::Equal && can_assign => {
                        let token = token.clone();
                        self.scanner.next();
                        self.compile_expression();
                        match address {
                            Some(address) => {
                                self.function().add_op(OpCode::SetLocal);
                                self.function().add_address(address as usize);
                            }
                            None => {
                                self.function().add_op(OpCode::SetGlobal);
                                self.add_constant(token.value().unwrap());
                            }
                        }
                    }

                    Some(token) if token.kind() == Kind::Equal => {
                        self.scanner.next();
                        self.errors.push(LoxError::new(
                            "Invalid assignment target",
                            ErrorContext::Compile,
                            None,
                        ));
                    }

                    Some(_token) if _token.kind() == Kind::LeftParen => {
                        self.scanner.next();
                        let mut args = 0;
                        loop {
                            match self.scanner.peek() {
                                Some(token) if token.kind() == Kind::RightParen => {
                                    self.scanner.next();
                                    break;
                                }

                                Some(_) => {
                                    self.compile_expression();
                                    args += 1;
                                    match self.scanner.peek() {
                                        Some(token) if token.kind() == Kind::Comma => {
                                            self.scanner.next();
                                            continue;
                                        }

                                        Some(token) if token.kind() == Kind::RightParen => {
                                            self.scanner.next();
                                            break;
                                        }

                                        None => self.errors.push(LoxError::new(
                                            "Unexpected end of script",
                                            ErrorContext::Compile,
                                            None,
                                        )),

                                        _ => self.errors.push(LoxError::new(
                                            format!("unexpected {:?} #1", token).as_str(),
                                            ErrorContext::Compile,
                                            None,
                                        )),
                                    }
                                }

                                None => self.errors.push(LoxError::new(
                                    "Unexpected end of script",
                                    ErrorContext::Compile,
                                    None,
                                )),
                            }
                        }

                        self.function().add_op(OpCode::Call);
                        self.add_constant(Value::Double(args.into()));
                        self.add_constant(token.value().unwrap());
                    }

                    _ => match address {
                        Some(address) => {
                            self.function().add_op(OpCode::GetLocal);
                            self.function().add_address(address as usize);
                        }
                        None => {
                            self.function().add_op(OpCode::GetGlobal);
                            self.add_constant(token.value().unwrap());
                        }
                    },
                }
            }

            Some(token) => self.errors.push(LoxError::new(
                format!("unexpected {:?} #3", token).as_str(),
                ErrorContext::Compile,
                None,
            )),

            None => self.errors.push(LoxError::new(
                "Unexpected end of script",
                ErrorContext::Compile,
                None,
            )),
        }
    }

    fn expect(&mut self, kind: Kind) {
        match self.scanner.peek() {
            Some(token) => match token.kind() {
                token_kind if token_kind == kind => {
                    self.scanner.next();
                }

                _ => self.errors.push(LoxError::new(
                    format!("unexpected {:?} #4", token).as_str(),
                    ErrorContext::Compile,
                    None,
                )),
            },

            None => self.errors.push(LoxError::new(
                "Unexpected end of script",
                ErrorContext::Compile,
                None,
            )),
        }
    }

    fn resolve_local(&mut self, name: String) -> Option<u128> {
        self.locals
            .iter()
            .enumerate()
            .rev()
            .find(|(_, item)| item.0 == name)
            .map(|(index, _)| index as u128)
    }

    fn function(&mut self) -> &mut Function {
        self.functions.last_mut().unwrap()
    }

    fn add_constant(&mut self, value: Value) {
        let address = self.vm.add_constant(value);
        self.function().add_op(OpCode::Constant);
        self.function().add_address(address);
    }

    fn new_function(&mut self, name: String, arity: u128) {
        let function = Function::new(name, arity);
        self.functions.push(function);
    }
}
