use std::iter::Peekable;

use rand::{distributions::Alphanumeric, Rng};

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
                    self.function().already_returns();
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
        match self.scanner.next() {
            Some(token) if token.kind() == Kind::Identifier => {
                let variable_name = token.value().unwrap();

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

            Some(token) => self.errors.push(LoxError::new(
                format!("unexpected {:?} #1", token).as_str(),
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
            Some(token) if token.kind() == Kind::If => {
                self.scanner.next();
                self.compile_if();
            }

            Some(token) if token.kind() == Kind::While => {
                self.scanner.next();
                self.compile_while();
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
                self.add_constant(Value::Number(scope as f64));

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
            }
        }
    }

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

    // TODO needs closures for full functionality!
    fn compile_while(&mut self) {
        let name: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();
        self.new_function(name.clone(), 0);

        self.compile_expression();

        let jump_address = self.function().add_jump(true);

        self.compile_statement(true);
        self.function().add_op(OpCode::Loop);
        self.add_constant(Value::String(name.clone()));

        self.function().patch_jump(jump_address);

        let function = self.functions.pop().unwrap();
        self.vm.add_loop(function);

        self.function().add_op(OpCode::Loop);
        self.add_constant(Value::String(name));
    }

    fn compile_expression(&mut self) {
        self.compile_term(true);
        loop {
            match self.scanner.peek() {
                Some(token) if token.kind() == Kind::Minus => {
                    self.compile_term(false);
                    self.function().add_op(OpCode::Add);
                }

                Some(token) if token.kind() == Kind::Plus => {
                    self.scanner.next();
                    self.compile_term(false);
                    self.function().add_op(OpCode::Add);
                }

                Some(token) if token.kind() == Kind::Concat => {
                    self.scanner.next();
                    self.compile_term(false);
                    self.function().add_op(OpCode::Concat)
                }

                Some(token) if token.kind() == Kind::Or => {
                    self.scanner.next();
                    let else_jump_address = self.function().add_jump(true);
                    let end_jump_address = self.function().add_jump(false);
                    self.function().patch_jump(else_jump_address);
                    self.function().add_op(OpCode::Pop);
                    self.compile_term(false);
                    self.function().patch_jump(end_jump_address);
                }

                Some(_) => break,

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
                Some(token) if token.kind() == Kind::Star => {
                    self.scanner.next();
                    self.compile_factor(false);
                    self.function().add_op(OpCode::Multiply);
                }

                Some(token) if token.kind() == Kind::Slash => {
                    self.scanner.next();
                    self.compile_factor(false);
                    self.function().add_op(OpCode::Divide);
                }

                Some(token) if token.kind() == Kind::Percent => {
                    self.scanner.next();
                    self.compile_factor(false);
                    self.function().add_op(OpCode::Rem);
                }

                Some(token) if token.kind() == Kind::And => {
                    self.scanner.next();
                    let jump_address = self.function().add_jump(true);
                    self.function().add_op(OpCode::Pop);
                    self.compile_factor(false);
                    self.function().patch_jump(jump_address);
                }

                Some(token) if token.kind() == Kind::EqualEqual => {
                    self.scanner.next();
                    self.compile_factor(false);
                    self.function().add_op(OpCode::Equal);
                }

                Some(token) if token.kind() == Kind::BangEqual => {
                    self.scanner.next();
                    self.compile_factor(false);
                    self.function().add_op(OpCode::NotEqual);
                }

                Some(token) if token.kind() == Kind::GreaterEqual => {
                    self.scanner.next();
                    self.compile_factor(false);
                    self.function().add_op(OpCode::GreaterEqual);
                }

                Some(token) if token.kind() == Kind::Greater => {
                    self.scanner.next();
                    self.compile_factor(false);
                    self.function().add_op(OpCode::Greater);
                }

                Some(token) if token.kind() == Kind::LessEqual => {
                    self.scanner.next();
                    self.compile_factor(false);
                    self.function().add_op(OpCode::LessEqual);
                }

                Some(token) if token.kind() == Kind::Less => {
                    self.scanner.next();
                    self.compile_factor(false);
                    self.function().add_op(OpCode::Less);
                }

                Some(_) => break,

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
                    Some(token) if token.kind() == Kind::RightParen => {
                        self.scanner.next();
                    }

                    Some(_) => self.errors.push(LoxError::new(
                        format!("unexpected {:?} #2", token).as_str(),
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

            Some(token) if token.kind() == Kind::Identifier => {
                let name: String = token.value().unwrap().into();
                let address = self.resolve_local(name.clone());
                match self.scanner.peek() {
                    Some(token) if token.kind() == Kind::Equal && can_assign => {
                        self.scanner.next();
                        self.compile_expression();
                        match address {
                            Some(address) => {
                                self.function().add_op(OpCode::SetLocal);
                                self.function().add_address(address as usize);
                            }
                            None => {
                                self.function().add_op(OpCode::SetGlobal);
                                self.add_constant(Value::String(name));
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
                        self.add_constant(Value::Number(self.scope_depth as f64));
                        self.add_constant(Value::Number(args.into()));
                        self.add_constant(token.value().unwrap());
                    }

                    _ => match address {
                        Some(address) => {
                            self.function().add_op(OpCode::GetLocal);
                            self.function().add_address(address as usize);
                        }

                        None if !self.vm.function_exists(self.scope_depth, &name) => {
                            self.function().add_op(OpCode::GetGlobal);
                            self.add_constant(token.value().unwrap());
                        }

                        _ => {
                            let function =
                                self.vm.resolve_function(&name, self.scope_depth).unwrap();
                            self.add_constant(Value::Function(function));
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
            Some(token) if token.kind() == kind => {
                self.scanner.next();
            }

            Some(token) => self.errors.push(LoxError::new(
                format!("unexpected {:?} #4", token).as_str(),
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
