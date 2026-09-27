use crate::compiler::op::OpCode;
use crate::errors::error::{ErrorContext, InterpretResult, LoxError};
use crate::runtime::function::Function;
use crate::runtime::nif::resolve_nif;
use crate::runtime::value::Value;
use crate::runtime::vm::VM;
use crate::syntax::ast::*;

pub(crate) struct Compiler<'a> {
    vm: &'a mut VM,
    scope_depth: u128,
    globals: Vec<String>,
    errors: Vec<LoxError>,
    panicking: bool,
    functions: Vec<Function>,
    locals: Vec<Vec<(String, u128)>>,
}

impl<'a> Compiler<'a> {
    pub(crate) fn new(vm: &'a mut VM, function: Function) -> Compiler<'a> {
        Compiler {
            vm,
            errors: vec![],
            globals: vec![],
            panicking: false,
            locals: vec![vec![]],
            scope_depth: 0,
            functions: vec![function],
        }
    }

    pub(crate) fn compile(&mut self, program: &Program) -> Result<Function, InterpretResult> {
        for stmt in &program.stmts {
            self.compile_declaration(stmt);
        }

        match self.errors.len() {
            0 => Ok(self.function().clone()),
            _ => {
                self.errors.iter().for_each(|e| eprintln!("{}", e));
                Err(InterpretResult::CompileError)
            }
        }
    }

    fn compile_declaration(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { .. } => self.compile_let(stmt),
            Stmt::Fun { .. } => self.compile_fun(stmt),
            Stmt::Return { value, .. } => {
                self.compile_expr(value);
                self.function().add_op(OpCode::Return);
                self.function().already_returns();
            }
            _ => self.compile_stmt(stmt),
        }
        if self.panicking {
            self.panicking = false;
        }
    }

    fn compile_let(&mut self, stmt: &Stmt) {
        let Stmt::Let {
            name, initializer, ..
        } = stmt
        else {
            return;
        };

        match initializer {
            Some(expr) => self.compile_expr(expr),
            None => self.function().add_op(OpCode::Nil),
        }

        match self.scope_depth {
            0 => {
                self.globals.push(name.clone());
                self.function().add_op(OpCode::DefGlobal);
                self.add_constant(Value::String(name.clone()));
            }
            _ => {
                self.declare_local_variable(name.clone());
            }
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
                    &format!("Variable {:?} is already defined", variable_name),
                    ErrorContext::Compile,
                    None,
                ),
                None => self.locals().push((variable_name, current_scope)),
            }
        }
    }

    fn compile_fun(&mut self, stmt: &Stmt) {
        let Stmt::Fun {
            name, params, body, ..
        } = stmt
        else {
            return;
        };

        if self.vm.function_exists(self.scope_depth, name) || resolve_nif(name).is_some() {
            self.error(
                &format!("Function {} already exists", name),
                ErrorContext::Compile,
                None,
            );
            return;
        }

        self.scope_depth += 1;
        self.locals.push(vec![]);

        let current_scope = self.scope_depth;
        for param in params {
            self.locals().push((param.clone(), current_scope));
        }

        self.new_function(name.clone(), params.len() as u128);

        for s in body {
            self.compile_declaration(s);
        }

        self.finalize_function();
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

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expression { expr, .. } => {
                self.compile_expr(expr);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.compile_expr(condition);
                let jump_address = self.function().add_jump(true);
                self.function().add_op(OpCode::Pop);
                self.compile_stmt(then_branch);
                let else_jump_address = self.function().add_jump(false);
                self.function().patch_jump(jump_address);
                self.function().add_op(OpCode::Pop);

                if let Some(else_stmt) = else_branch {
                    self.compile_stmt(else_stmt);
                }

                self.function().patch_jump(else_jump_address);
            }
            Stmt::While {
                condition, body, ..
            } => {
                let loop_start = self.function().code_size();
                self.compile_expr(condition);
                let exit_jump = self.function().add_jump(true);
                self.function().add_op(OpCode::Pop);
                self.compile_stmt(body);
                self.function().add_jump_back(loop_start);
                self.function().patch_jump(exit_jump);
                self.function().add_op(OpCode::Pop);
            }
            Stmt::Block { stmts, .. } => {
                self.scope_depth += 1;
                for s in stmts {
                    self.compile_declaration(s);
                }
                let current_scope = self.scope_depth;
                let pop_count = self
                    .locals()
                    .iter()
                    .filter(|(_, scope)| *scope == current_scope)
                    .count();
                for _ in 0..pop_count {
                    self.function().add_op(OpCode::Pop);
                }
                self.locals().retain(|(_, scope)| *scope != current_scope);
                self.scope_depth -= 1;
            }
            _ => unreachable!(
                "Return/Let/Fun are handled by compile_declaration and never reach compile_stmt"
            ),
        }
    }

    fn compile_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal { value, .. } => match value {
                Value::Nil => self.function().add_op(OpCode::Nil),
                Value::Boolean(true) => self.add_constant(Value::Boolean(true)),
                Value::Boolean(false) => self.add_constant(Value::Boolean(false)),
                _ => self.add_constant(value.clone()),
            },
            Expr::Variable { name, .. } => {
                self.compile_variable_read(name);
            }
            Expr::Assignment { name, value, .. } => {
                self.compile_expr(value);
                self.compile_variable_write(name);
            }
            Expr::Unary {
                operator, operand, ..
            } => {
                self.compile_expr(operand);
                match operator {
                    UnaryOp::Negate => self.function().add_op(OpCode::Negate),
                    UnaryOp::Not => self.function().add_op(OpCode::Not),
                }
            }
            Expr::Binary {
                operator,
                left,
                right,
                ..
            } => {
                self.compile_expr(left);
                self.compile_expr(right);
                match operator {
                    BinaryOp::Add => self.function().add_op(OpCode::Add),
                    BinaryOp::Subtract => self.function().add_op(OpCode::Subtract),
                    BinaryOp::Multiply => self.function().add_op(OpCode::Multiply),
                    BinaryOp::Divide => self.function().add_op(OpCode::Divide),
                    BinaryOp::Rem => self.function().add_op(OpCode::Rem),
                    BinaryOp::Concat => self.function().add_op(OpCode::Concat),
                    BinaryOp::Equal => self.function().add_op(OpCode::Equal),
                    BinaryOp::NotEqual => self.function().add_op(OpCode::NotEqual),
                    BinaryOp::Greater => self.function().add_op(OpCode::Greater),
                    BinaryOp::GreaterEqual => self.function().add_op(OpCode::GreaterEqual),
                    BinaryOp::Less => self.function().add_op(OpCode::Less),
                    BinaryOp::LessEqual => self.function().add_op(OpCode::LessEqual),
                }
            }
            Expr::Logical {
                operator,
                left,
                right,
                ..
            } => match operator {
                LogicalOp::Or => {
                    self.compile_expr(left);
                    let else_jump_address = self.function().add_jump(true);
                    let end_jump_address = self.function().add_jump(false);
                    self.function().patch_jump(else_jump_address);
                    self.function().add_op(OpCode::Pop);
                    self.compile_expr(right);
                    self.function().patch_jump(end_jump_address);
                }
                LogicalOp::And => {
                    self.compile_expr(left);
                    let jump_address = self.function().add_jump(true);
                    self.function().add_op(OpCode::Pop);
                    self.compile_expr(right);
                    self.function().patch_jump(jump_address);
                }
            },
            Expr::Grouping { expr, .. } => {
                self.compile_expr(expr);
            }
            Expr::Call { callee, args, .. } => {
                for arg in args {
                    self.compile_expr(arg);
                }
                self.function().add_op(OpCode::Call);
                self.add_constant(Value::Number(self.scope_depth as f64));
                self.add_constant(Value::Number(args.len() as f64));
                self.add_constant(Value::String(callee.clone()));
            }
            Expr::Narrowing { .. } => {
                self.error(
                    "Narrowing is not yet implemented",
                    ErrorContext::Compile,
                    None,
                );
            }
        }
    }

    fn compile_variable_read(&mut self, name: &str) {
        let address = self.resolve_local(name.to_string());

        if let Some(addr) = address {
            self.function().add_op(OpCode::GetLocal);
            self.function().add_address(addr as usize);
        } else if self.vm.function_exists(self.scope_depth, &name.to_string()) {
            let (_, address) = self
                .vm
                .resolve_function(&name.to_string(), self.scope_depth)
                .unwrap();
            self.add_constant(Value::Function((address, None)));
        } else {
            self.resolve_captured_or_global(name.to_string());
        }
    }

    fn compile_variable_write(&mut self, name: &str) {
        let address = self.resolve_local(name.to_string());
        match address {
            Some(address) => {
                self.function().add_op(OpCode::SetLocal);
                self.function().add_address(address as usize);
            }
            None => match self.globals.iter().find(|variable| **variable == name) {
                Some(_) => {
                    self.function().add_op(OpCode::SetGlobal);
                    self.add_constant(Value::String(name.to_string()));
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
}
