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
    globals: Vec<String>,
    errors: Vec<LoxError>,
    panicking: bool,
    functions: Vec<Function>,
    locals: Vec<Vec<(String, u128)>>,
    scanner: Peekable<Scanner<'a>>,
}

impl<'a> Compiler<'a> {
    pub(crate) fn new(vm: &'a mut VM, function: Function, source: &'a str) -> Compiler<'a> {
        Compiler {
            vm,
            errors: vec![],
            globals: vec![],
            panicking: false,
            locals: vec![vec![]],
            scope_depth: 0,
            functions: vec![function],
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

                Kind::For => {
                    self.scanner.next();
                    self.error(
                        "Feature 'for' is not yet implemented",
                        ErrorContext::Compile,
                        None,
                    );
                }

                Kind::Class => {
                    self.scanner.next();
                    self.error(
                        "Feature 'class' is not yet implemented",
                        ErrorContext::Compile,
                        None,
                    );
                }

                _ => self.compile_statement(true),
            },

            None => self.error("Unexpected end of script", ErrorContext::Compile, None),
        }

        if self.panicking {
            self.synchronize();
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

                    None => self.error("Unexpected end of script", ErrorContext::Compile, None),
                }
                self.expect(Kind::Semicolon);

                match self.scope_depth {
                    0 => {
                        self.globals.push(variable_name.clone().into());
                        self.function().add_op(OpCode::DefGlobal);
                        self.add_constant(variable_name);
                    }

                    _ => {
                        self.declare_local_variable(variable_name.into());
                    }
                }
            }

            Some(token) => self.error(
                format!("unexpected {:?} #1", token).as_str(),
                ErrorContext::Compile,
                None,
            ),

            None => self.error("Unexpected end of script", ErrorContext::Compile, None),
        }
    }

    fn declare_local_variable(&mut self, variable_name: String) {
        if variable_name != *"_" {
            let current_scope = self.scope_depth;
            match self
                .locals()
                .iter()
                .find(|(name, scope)| *name == variable_name && *scope == current_scope)
            {
                Some(_) => self.error(
                    format!("Variable {:?} is already defined", variable_name).as_str(),
                    ErrorContext::Compile,
                    None,
                ),
                None => self.locals().push((variable_name, current_scope)),
            }
        }
    }

    fn compile_fun(&mut self) {
        match self.scanner.next() {
            Some(token) if token.kind() == Kind::Identifier => {
                let function_name: String = token.value().unwrap().into();

                if self.vm.function_exists(self.scope_depth, &function_name)
                    || resolve_nif(&function_name).is_some()
                {
                    self.error(
                        format!("Function {} already exists", function_name).as_str(),
                        ErrorContext::Compile,
                        None,
                    );
                    return;
                }

                self.expect(Kind::LeftParen);
                self.scope_depth += 1;
                self.locals.push(vec![]);
                let arity = self.compile_parameters();

                self.new_function(function_name, arity);
                self.compile_statement(false);
                self.finalize_function();
            }

            None => self.error("Unexpected end of script", ErrorContext::Compile, None),

            Some(token) => self.error(
                format!("unexpected {:?} #1", token).as_str(),
                ErrorContext::Compile,
                None,
            ),
        }
    }

    fn compile_parameters(&mut self) -> u128 {
        let mut arity = 0;
        loop {
            match self.scanner.next() {
                Some(token) if token.kind() == Kind::Identifier => {
                    arity += 1;
                    let variable_name: String = token.value().unwrap().into();
                    let current_scope = self.scope_depth;
                    self.locals().push((variable_name, current_scope));

                    match self.scanner.peek() {
                        Some(token) if token.kind() == Kind::Comma => {
                            self.scanner.next();
                            continue;
                        }

                        Some(token) if token.kind() == Kind::RightParen => {
                            self.scanner.next();
                            break;
                        }

                        None => self.error("Unexpected end of script", ErrorContext::Compile, None),

                        _ => self.error(
                            "unexpected token in parameter list",
                            ErrorContext::Compile,
                            None,
                        ),
                    }
                }

                Some(token) if token.kind() == Kind::RightParen => {
                    break;
                }

                None => self.error("Unexpected end of script", ErrorContext::Compile, None),

                Some(token) => self.error(
                    format!("unexpected {:?} #1", token).as_str(),
                    ErrorContext::Compile,
                    None,
                ),
            }
        }
        arity
    }

    fn finalize_function(&mut self) {
        if let Some(false) = self.function().has_return() {
            self.function().add_op(OpCode::Nil);
            self.function().add_op(OpCode::Return);
        }
        self.scope_depth -= 1;
        let function = self.functions.pop().unwrap();
        self.locals.pop();
        let address = self.vm.add_function(self.scope_depth, function);
        if self.scope_depth > 0 {
            self.function().add_op(OpCode::MakeClosure);
            self.add_constant(Value::Number(address as f64));
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
                self.compile_block(manage_scope);
            }

            None => {
                self.error("Unexpected end of script", ErrorContext::Compile, None);
            }

            _ => {
                self.compile_expression();
                self.expect(Kind::Semicolon);
            }
        }
    }

    fn compile_block(&mut self, manage_scope: bool) {
        if manage_scope {
            self.scope_depth += 1;
        }
        loop {
            match self.scanner.peek() {
                Some(token) => match token.kind() {
                    Kind::RightBrace | Kind::Eof => break,
                    _ => (),
                },

                None => {
                    self.error("Unexpected end of script", ErrorContext::Compile, None);
                    break;
                }
            }
            self.compile_declaration();
        }
        self.expect(Kind::RightBrace);

        let current_scope = self.scope_depth;
        if manage_scope {
            let pop_count = self
                .locals()
                .iter()
                .filter(|(_, scope)| *scope == current_scope)
                .count();
            for _ in 0..pop_count {
                self.function().add_op(OpCode::Pop);
            }
        }
        self.locals().retain(|(_, scope)| *scope != current_scope);

        if manage_scope {
            self.scope_depth -= 1;
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

        if let Some(token) = self.scanner.peek() {
            if token.kind() == Kind::Else {
                self.scanner.next();
                self.compile_statement(true);
            }
        }

        self.function().patch_jump(else_jump_address);
    }

    fn compile_while(&mut self) {
        let loop_start = self.function().code_size();
        self.compile_expression();
        let exit_jump = self.function().add_jump(true);
        self.function().add_op(OpCode::Pop);
        self.compile_statement(true);
        self.function().add_jump_back(loop_start);
        self.function().patch_jump(exit_jump);
        self.function().add_op(OpCode::Pop);
    }

    fn compile_expression(&mut self) {
        self.compile_or(true);
    }

    fn compile_or(&mut self, can_assign: bool) {
        self.compile_and(can_assign);
        loop {
            match self.scanner.peek() {
                Some(token) if token.kind() == Kind::Or => {
                    self.scanner.next();
                    let else_jump_address = self.function().add_jump(true);
                    let end_jump_address = self.function().add_jump(false);
                    self.function().patch_jump(else_jump_address);
                    self.function().add_op(OpCode::Pop);
                    self.compile_and(false);
                    self.function().patch_jump(end_jump_address);
                }

                Some(_) => break,

                None => {
                    self.error("Unexpected end of script", ErrorContext::Compile, None);
                    break;
                }
            };
        }
    }

    fn compile_and(&mut self, can_assign: bool) {
        self.compile_equality(can_assign);
        loop {
            match self.scanner.peek() {
                Some(token) if token.kind() == Kind::And => {
                    self.scanner.next();
                    let jump_address = self.function().add_jump(true);
                    self.function().add_op(OpCode::Pop);
                    self.compile_equality(false);
                    self.function().patch_jump(jump_address);
                }

                Some(_) => break,

                None => {
                    self.error("Unexpected end of script", ErrorContext::Compile, None);
                    break;
                }
            };
        }
    }

    fn compile_equality(&mut self, can_assign: bool) {
        self.compile_comparison(can_assign);
        loop {
            match self.scanner.peek() {
                Some(token) if token.kind() == Kind::EqualEqual => {
                    self.scanner.next();
                    self.compile_comparison(false);
                    self.function().add_op(OpCode::Equal);
                }

                Some(token) if token.kind() == Kind::BangEqual => {
                    self.scanner.next();
                    self.compile_comparison(false);
                    self.function().add_op(OpCode::NotEqual);
                }

                Some(_) => break,

                None => {
                    self.error("Unexpected end of script", ErrorContext::Compile, None);
                    break;
                }
            };
        }
    }

    fn compile_comparison(&mut self, can_assign: bool) {
        self.compile_term(can_assign);
        loop {
            match self.scanner.peek() {
                Some(token) if token.kind() == Kind::Greater => {
                    self.scanner.next();
                    self.compile_term(false);
                    self.function().add_op(OpCode::Greater);
                }

                Some(token) if token.kind() == Kind::GreaterEqual => {
                    self.scanner.next();
                    self.compile_term(false);
                    self.function().add_op(OpCode::GreaterEqual);
                }

                Some(token) if token.kind() == Kind::Less => {
                    self.scanner.next();
                    self.compile_term(false);
                    self.function().add_op(OpCode::Less);
                }

                Some(token) if token.kind() == Kind::LessEqual => {
                    self.scanner.next();
                    self.compile_term(false);
                    self.function().add_op(OpCode::LessEqual);
                }

                Some(_) => break,

                None => {
                    self.error("Unexpected end of script", ErrorContext::Compile, None);
                    break;
                }
            };
        }
    }

    fn compile_term(&mut self, can_assign: bool) {
        self.compile_factor(can_assign);
        loop {
            match self.scanner.peek() {
                Some(token) if token.kind() == Kind::Plus => {
                    self.scanner.next();
                    self.compile_factor(false);
                    self.function().add_op(OpCode::Add);
                }

                Some(token) if token.kind() == Kind::Minus => {
                    self.scanner.next();
                    self.compile_factor(false);
                    self.function().add_op(OpCode::Subtract);
                }

                Some(token) if token.kind() == Kind::Concat => {
                    self.scanner.next();
                    self.compile_factor(false);
                    self.function().add_op(OpCode::Concat);
                }

                Some(_) => break,

                None => {
                    self.error("Unexpected end of script", ErrorContext::Compile, None);
                    break;
                }
            };
        }
    }

    fn compile_factor(&mut self, can_assign: bool) {
        self.compile_unary(can_assign);
        loop {
            match self.scanner.peek() {
                Some(token) if token.kind() == Kind::Star => {
                    self.scanner.next();
                    self.compile_unary(false);
                    self.function().add_op(OpCode::Multiply);
                }

                Some(token) if token.kind() == Kind::Slash => {
                    self.scanner.next();
                    self.compile_unary(false);
                    self.function().add_op(OpCode::Divide);
                }

                Some(token) if token.kind() == Kind::Percent => {
                    self.scanner.next();
                    self.compile_unary(false);
                    self.function().add_op(OpCode::Rem);
                }

                Some(_) => break,

                None => {
                    self.error("Unexpected end of script", ErrorContext::Compile, None);
                    break;
                }
            };
        }
    }

    fn compile_unary(&mut self, can_assign: bool) {
        match self.scanner.peek() {
            Some(token) if token.kind() == Kind::Not => {
                self.scanner.next();
                self.compile_unary(false);
                self.function().add_op(OpCode::Not);
            }

            Some(token) if token.kind() == Kind::Minus => {
                self.scanner.next();
                self.compile_unary(false);
                self.function().add_op(OpCode::Negate);
            }

            _ => self.compile_call(can_assign),
        }
    }

    fn compile_call(&mut self, can_assign: bool) {
        match self.scanner.peek() {
            Some(token) if token.kind() == Kind::Identifier => {
                let token = self.scanner.next().unwrap();
                let name: String = token.value().unwrap().into();

                match self.scanner.peek() {
                    Some(next) if next.kind() == Kind::LeftParen => {
                        self.scanner.next();
                        let args = self.compile_arguments();

                        self.function().add_op(OpCode::Call);
                        self.add_constant(Value::Number(self.scope_depth as f64));
                        self.add_constant(Value::Number(args.into()));
                        self.add_constant(token.value().unwrap());
                    }

                    _ => {
                        self.compile_identifier(name, can_assign);
                    }
                }
            }

            _ => self.compile_primary(),
        }
    }

    fn compile_arguments(&mut self) -> i32 {
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

                        None => self.error("Unexpected end of script", ErrorContext::Compile, None),

                        _ => self.error(
                            "unexpected token in argument list",
                            ErrorContext::Compile,
                            None,
                        ),
                    }
                }

                None => {
                    self.error("Unexpected end of script", ErrorContext::Compile, None);
                    break;
                }
            }
        }
        args
    }

    fn compile_identifier(&mut self, name: String, can_assign: bool) {
        let address = self.resolve_local(name.clone());

        match self.scanner.peek().cloned() {
            Some(token) if token.kind() == Kind::Equal && can_assign => {
                self.scanner.next();
                self.compile_expression();
                self.compile_assignment(name, address);
            }

            Some(token) if token.kind() == Kind::Equal => {
                self.scanner.next();
                self.error("Invalid assignment target", ErrorContext::Compile, None);
            }

            _ if address.is_some() => {
                self.function().add_op(OpCode::GetLocal);
                self.function().add_address(address.unwrap() as usize);
            }

            _ if self.vm.function_exists(self.scope_depth, &name) => {
                let (_, address) = self.vm.resolve_function(&name, self.scope_depth).unwrap();
                self.add_constant(Value::Function((address, None)));
            }

            _ => {
                self.resolve_captured_or_global(name);
            }
        }
    }

    fn compile_assignment(&mut self, name: String, address: Option<u128>) {
        match address {
            Some(address) => {
                self.function().add_op(OpCode::SetLocal);
                self.function().add_address(address as usize);
            }

            None => match self.globals.iter().find(|variable| **variable == name) {
                Some(_) => {
                    self.function().add_op(OpCode::SetGlobal);
                    self.add_constant(Value::String(name));
                }
                None => {
                    self.error(
                        "Cannot assign to captured variable",
                        ErrorContext::Compile,
                        None,
                    );
                }
            },
        }
    }

    fn resolve_captured_or_global(&mut self, name: String) {
        let captured = match self.locals.as_slice().split_last() {
            Some((_, captured_frames)) => captured_frames
                .iter()
                .enumerate()
                .rev()
                .map(|(index, frame)| (index, frame.iter().enumerate()))
                .find_map(|(frame_index, mut frame)| {
                    frame.find_map(|(index, item)| match item.0 == name {
                        true => Some((frame_index, index)),
                        false => None,
                    })
                }),

            None => None,
        };

        match captured {
            Some((frame, address)) => {
                self.function().add_op(OpCode::GetCaptured);
                self.add_constant(Value::String(name.clone()));
                self.function().add_capture(name, frame, address);
            }

            None => {
                self.function().add_op(OpCode::GetGlobal);
                self.add_constant(Value::String(name));
            }
        }
    }

    fn compile_primary(&mut self) {
        match self.scanner.next() {
            Some(token) if token.kind() == Kind::Nil => self.function().add_op(OpCode::Nil),
            Some(token) if [Kind::Number, Kind::String].contains(&token.kind()) => {
                self.add_constant(token.value().unwrap())
            }
            Some(token) if token.kind() == Kind::True => self.add_constant(Value::Boolean(true)),
            Some(token) if token.kind() == Kind::False => self.add_constant(Value::Boolean(false)),

            Some(token) if token.kind() == Kind::LeftParen => {
                self.compile_grouping(token);
            }

            Some(token) => self.compile_primary_token(token),

            None => self.error("Unexpected end of script", ErrorContext::Compile, None),
        }
    }

    fn compile_grouping(&mut self, open_token: crate::token::Token) {
        self.compile_expression();
        match self.scanner.peek() {
            Some(token) if token.kind() == Kind::RightParen => {
                self.scanner.next();
            }

            Some(_) => self.error(
                format!("unexpected {:?} #2", open_token).as_str(),
                ErrorContext::Compile,
                None,
            ),

            None => self.error("Unexpected end of script", ErrorContext::Compile, None),
        }
    }

    fn compile_primary_token(&mut self, token: crate::token::Token) {
        match token.kind() {
            Kind::This => self.error(
                "Feature 'this' is not yet implemented",
                ErrorContext::Compile,
                None,
            ),
            Kind::Super => self.error(
                "Feature 'super' is not yet implemented",
                ErrorContext::Compile,
                None,
            ),
            Kind::Expands => self.error(
                "Feature 'expands' is not yet implemented",
                ErrorContext::Compile,
                None,
            ),
            _ => self.error(
                format!("unexpected {:?} #3", token).as_str(),
                ErrorContext::Compile,
                None,
            ),
        }
    }

    fn expect(&mut self, kind: Kind) {
        match self.scanner.peek().cloned() {
            Some(token) if token.kind() == kind => {
                self.scanner.next();
            }

            Some(token) => self.error(
                format!("expected {:?}, got {:?}", kind, token).as_str(),
                ErrorContext::Compile,
                None,
            ),

            None => self.error("Unexpected end of script", ErrorContext::Compile, None),
        }
    }

    fn resolve_local(&mut self, name: String) -> Option<u128> {
        self.locals()
            .iter()
            .enumerate()
            .rev()
            .find(|(_, item)| item.0 == name)
            .map(|(index, _)| index as u128)
    }

    fn locals(&mut self) -> &mut Vec<(String, u128)> {
        self.locals.last_mut().unwrap()
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

    fn error(&mut self, msg: &str, context: ErrorContext, line: Option<usize>) {
        if !self.panicking {
            self.errors.push(LoxError::new(msg, context, line));
            self.panicking = true;
        }
    }

    fn synchronize(&mut self) {
        self.panicking = false;
        loop {
            match self.scanner.peek() {
                Some(token) => match token.kind() {
                    Kind::Semicolon => {
                        self.scanner.next();
                        return;
                    }
                    Kind::RightBrace
                    | Kind::Let
                    | Kind::Fun
                    | Kind::While
                    | Kind::If
                    | Kind::Return
                    | Kind::Eof => {
                        return;
                    }
                    _ => {
                        self.scanner.next();
                    }
                },
                None => return,
            }
        }
    }
}
