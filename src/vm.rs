use std::collections::HashMap;
use std::env::var_os;
use std::time::Instant;

use crate::chunk::Chunk;
use crate::compiler::Compiler;
use crate::error::InterpretResult;
use crate::function::Function;
use crate::nif::{resolve_nif, Nif};
use crate::op::OpCode;
use crate::value::Value;

fn read_operand(function: &Function, ip: &mut usize) -> Option<usize> {
    let value = function.get_code(*ip)?;
    *ip += 1;
    Some(value)
}

fn read_constant_operand(function: &Function, ip: &mut usize) -> Option<usize> {
    *ip += 1; // skip Constant opcode
    let value = function.get_code(*ip)?;
    *ip += 1;
    Some(value)
}

pub(crate) struct VM {
    #[cfg(test)]
    pub stdout: Vec<String>,

    start_time: Instant,
    stack: Vec<Vec<Value>>,
    constants: Chunk<Value>,
    globals: HashMap<String, Value>,
    functions: Vec<(Function, u128)>,
}

impl VM {
    pub(crate) fn new() -> VM {
        VM {
            #[cfg(test)]
            stdout: vec![],

            functions: vec![],
            stack: vec![vec![]],
            constants: Chunk::new(),
            globals: HashMap::new(),
            start_time: Instant::now(),
        }
    }

    pub(crate) fn interpret(&mut self, source: String) -> InterpretResult {
        let main_function = Function::new_main("##MAIN##".to_string());
        let mut compiler = Compiler::new(self, main_function, &source);
        match compiler.compile() {
            Ok(main_function) => self.run(main_function),
            _ => InterpretResult::CompileError,
        }
    }

    pub(crate) fn run(&mut self, function: Function) -> InterpretResult {
        let debug = var_os("DEBUG").is_some();
        let mut ip: usize = 0;
        let code_size = function.code_size();

        while ip < code_size {
            let Some(current) = function.get_code(ip) else {
                return InterpretResult::RuntimeError;
            };
            ip += 1;
            let op_code = OpCode::from(current as u8);

            if debug {
                self.debug_print(&function, &op_code);
            }

            if let Some(result) = self.execute_op(op_code, &function, &mut ip) {
                return result;
            }
        }

        InterpretResult::Ok
    }

    fn debug_print(&self, function: &Function, op_code: &OpCode) {
        println!("\n{} OpCode\n{:?}", function, op_code);
        println!("\n{}", self.stack.len());
        if self.stack.len() > 1 {
            println!("{:#?}", self.stack.get(self.stack.len() - 2));
        }
        println!("{:#?}", self.stack.last());
    }

    fn execute_op(
        &mut self,
        op_code: OpCode,
        function: &Function,
        ip: &mut usize,
    ) -> Option<InterpretResult> {
        match op_code {
            OpCode::Return => Some(self.exec_return()),
            OpCode::Constant => self.exec_constant(function, ip),
            OpCode::Negate => self.exec_negate(),
            OpCode::Not => self.exec_not(),
            OpCode::Concat => self.exec_concat(),
            OpCode::Add | OpCode::Subtract | OpCode::Multiply => self.exec_arithmetic(op_code),
            OpCode::Divide | OpCode::Rem => self.exec_checked_arithmetic(op_code),
            OpCode::Equal
            | OpCode::NotEqual
            | OpCode::Greater
            | OpCode::GreaterEqual
            | OpCode::Less
            | OpCode::LessEqual => self.exec_comparison(op_code),
            OpCode::Pop => {
                self.stack_pop();
                None
            }
            OpCode::Nil => {
                self.stack_push(Value::Nil);
                None
            }
            OpCode::MakeClosure => self.exec_make_closure(function, ip),
            OpCode::GetCaptured => self.exec_get_captured(function, ip),
            OpCode::DefGlobal | OpCode::SetGlobal | OpCode::GetGlobal => {
                self.exec_global_op(op_code, function, ip)
            }
            OpCode::GetLocal | OpCode::SetLocal => self.exec_local_op(op_code, function, ip),
            OpCode::JumpIfFalse => self.exec_jump_if_false(function, ip),
            OpCode::Jump => self.exec_jump(function, ip),
            OpCode::JumpBack => self.exec_jump_back(function, ip),
            OpCode::Call => self.exec_call(function, ip),
            _ => Some(InterpretResult::CompileError),
        }
    }

    fn exec_return(&mut self) -> InterpretResult {
        let return_value = match self.stack_pop() {
            Some(value) => value,
            None => Value::Nil,
        };

        if let Value::Function((address, _)) = return_value {
            if let Some(returned_function) = self.functions.get_mut(address).cloned() {
                self.functions.remove(address);
                self.functions
                    .insert(address, (returned_function.0, returned_function.1 - 1));
            };
        };

        self.stack.pop();
        self.stack_push(return_value);
        InterpretResult::Ok
    }

    fn exec_constant(&mut self, function: &Function, ip: &mut usize) -> Option<InterpretResult> {
        let Some(address) = read_operand(function, ip) else {
            return Some(InterpretResult::RuntimeError);
        };
        let Some(constant) = self.get_constant(address) else {
            return Some(InterpretResult::RuntimeError);
        };

        match constant {
            Value::Function((address, None)) => {
                if let Some((func, _)) = self.functions.get(*address) {
                    self.stack_push(Value::Function((*address, Some(func.clone()))))
                } else {
                    return Some(InterpretResult::RuntimeError);
                }
            }
            _ => self.stack_push(constant.clone()),
        }
        None
    }

    fn exec_negate(&mut self) -> Option<InterpretResult> {
        let Some(Value::Number(value)) = self.stack_pop() else {
            return Some(InterpretResult::RuntimeError);
        };
        self.stack_push(Value::Number(-value));
        None
    }

    fn exec_not(&mut self) -> Option<InterpretResult> {
        let Some(value) = self.stack_pop() else {
            return Some(InterpretResult::RuntimeError);
        };
        match self.is_falsey(&value) {
            Some(result) => {
                self.stack_push(Value::Boolean(result));
                None
            }
            None => Some(InterpretResult::RuntimeError),
        }
    }

    fn exec_concat(&mut self) -> Option<InterpretResult> {
        let Some(right) = self.stack_pop() else {
            return Some(InterpretResult::RuntimeError);
        };
        let Some(left) = self.stack_pop() else {
            return Some(InterpretResult::RuntimeError);
        };
        let right: String = right.into();
        let left: String = left.into();
        self.stack_push(Value::String(left + &right));
        None
    }

    fn exec_arithmetic(&mut self, op_code: OpCode) -> Option<InterpretResult> {
        let Some(Value::Number(right)) = self.stack_pop() else {
            return Some(InterpretResult::RuntimeError);
        };
        let Some(Value::Number(left)) = self.stack_pop() else {
            return Some(InterpretResult::RuntimeError);
        };
        let result = match op_code {
            OpCode::Add => left + right,
            OpCode::Subtract => left - right,
            _ => left * right,
        };
        self.stack_push(Value::Number(result));
        None
    }

    fn exec_checked_arithmetic(&mut self, op_code: OpCode) -> Option<InterpretResult> {
        let Some(Value::Number(right)) = self.stack_pop() else {
            return Some(InterpretResult::RuntimeError);
        };
        if right == 0.0 {
            return Some(InterpretResult::RuntimeError);
        }
        let Some(Value::Number(left)) = self.stack_pop() else {
            return Some(InterpretResult::RuntimeError);
        };
        let result = match op_code {
            OpCode::Divide => left / right,
            _ => left % right,
        };
        self.stack_push(Value::Number(result));
        None
    }

    fn exec_comparison(&mut self, op_code: OpCode) -> Option<InterpretResult> {
        let Some(right) = self.stack_pop() else {
            return Some(InterpretResult::RuntimeError);
        };
        let Some(left) = self.stack_pop() else {
            return Some(InterpretResult::RuntimeError);
        };
        let result = match op_code {
            OpCode::Equal => left == right,
            OpCode::NotEqual => left != right,
            OpCode::Greater => left > right,
            OpCode::GreaterEqual => left >= right,
            OpCode::Less => left < right,
            _ => left <= right,
        };
        self.stack_push(Value::Boolean(result));
        None
    }

    fn exec_make_closure(
        &mut self,
        function: &Function,
        ip: &mut usize,
    ) -> Option<InterpretResult> {
        let Some(address) = read_constant_operand(function, ip) else {
            return Some(InterpretResult::RuntimeError);
        };
        let Some(Value::Number(address)) = self.get_constant(address) else {
            return Some(InterpretResult::RuntimeError);
        };

        let address = *address;
        let Some((ref mut func, _)) = self.functions.get_mut(address as usize) else {
            return Some(InterpretResult::RuntimeError);
        };

        func.captures()
            .iter()
            .for_each(|(name, (frame, address, _))| {
                func.populate_capture(
                    name.clone(),
                    self.stack
                        .get(*frame)
                        .unwrap()
                        .get(*address)
                        .cloned()
                        .unwrap(),
                );
            });
        None
    }

    fn exec_get_captured(
        &mut self,
        function: &Function,
        ip: &mut usize,
    ) -> Option<InterpretResult> {
        let Some(address) = read_constant_operand(function, ip) else {
            return Some(InterpretResult::RuntimeError);
        };
        let Some(Value::String(variable_name)) = self.get_constant(address) else {
            return Some(InterpretResult::RuntimeError);
        };

        let Some(value) = function.get_capture(variable_name.clone()) else {
            return Some(InterpretResult::RuntimeError);
        };
        self.stack_push(value.clone());
        None
    }

    fn exec_global_op(
        &mut self,
        op_code: OpCode,
        function: &Function,
        ip: &mut usize,
    ) -> Option<InterpretResult> {
        let Some(address) = read_constant_operand(function, ip) else {
            return Some(InterpretResult::RuntimeError);
        };
        let Some(Value::String(variable_name)) = self.get_constant(address) else {
            return Some(InterpretResult::RuntimeError);
        };
        let variable_name = variable_name.clone();

        match op_code {
            OpCode::DefGlobal => {
                let Some(value) = self.stack_pop() else {
                    return Some(InterpretResult::RuntimeError);
                };
                self.globals.insert(variable_name, value.clone());
            }
            OpCode::SetGlobal => {
                let Some(value) = self.stack_peek() else {
                    return Some(InterpretResult::RuntimeError);
                };
                if self.globals.insert(variable_name, value).is_none() {
                    return Some(InterpretResult::RuntimeError);
                };
            }
            _ => {
                let Some(value) = self.globals.get(&variable_name) else {
                    return Some(InterpretResult::RuntimeError);
                };
                self.stack_push(value.clone());
            }
        }
        None
    }

    fn exec_local_op(
        &mut self,
        op_code: OpCode,
        function: &Function,
        ip: &mut usize,
    ) -> Option<InterpretResult> {
        let Some(address) = read_operand(function, ip) else {
            return Some(InterpretResult::RuntimeError);
        };
        match op_code {
            OpCode::GetLocal => {
                let Some(value) = self.stack_get(address) else {
                    return Some(InterpretResult::RuntimeError);
                };
                self.stack_push(value.clone());
            }
            _ => {
                let Some(value) = self.stack_peek() else {
                    return Some(InterpretResult::RuntimeError);
                };
                self.stack_insert(address, value);
            }
        }
        None
    }

    fn exec_jump_if_false(
        &mut self,
        function: &Function,
        ip: &mut usize,
    ) -> Option<InterpretResult> {
        let Some(value) = self.stack_peek() else {
            return Some(InterpretResult::RuntimeError);
        };

        let is_falsey = match self.is_falsey(&value) {
            Some(result) => result,
            None => return Some(InterpretResult::RuntimeError),
        };

        let Some(size) = read_operand(function, ip) else {
            return Some(InterpretResult::RuntimeError);
        };

        if is_falsey {
            *ip += size;
        }
        None
    }

    fn exec_jump(&mut self, function: &Function, ip: &mut usize) -> Option<InterpretResult> {
        let Some(size) = read_operand(function, ip) else {
            return Some(InterpretResult::RuntimeError);
        };
        *ip += size;
        None
    }

    fn exec_jump_back(&mut self, function: &Function, ip: &mut usize) -> Option<InterpretResult> {
        let Some(target) = function.get_code(*ip) else {
            return Some(InterpretResult::RuntimeError);
        };
        *ip = target;
        None
    }

    fn exec_call(&mut self, function: &Function, ip: &mut usize) -> Option<InterpretResult> {
        let Some(address) = read_constant_operand(function, ip) else {
            return Some(InterpretResult::RuntimeError);
        };
        let Some(Value::Number(scope)) = self.get_constant(address) else {
            return Some(InterpretResult::RuntimeError);
        };
        let scope = *scope as u128;

        let Some(address) = read_constant_operand(function, ip) else {
            return Some(InterpretResult::RuntimeError);
        };
        let Some(Value::Number(args)) = self.get_constant(address) else {
            return Some(InterpretResult::RuntimeError);
        };
        let args = *args as u128;

        let Some(address) = read_constant_operand(function, ip) else {
            return Some(InterpretResult::RuntimeError);
        };
        let Some(Value::String(function_name)) = self.get_constant(address) else {
            return Some(InterpretResult::RuntimeError);
        };
        let function_name = function_name.clone();

        match resolve_nif(&function_name) {
            Some(nif) => self.call_nif(nif, args),
            None => self.call_user_function(&function_name, scope, args),
        }
    }

    fn call_nif(&mut self, nif: Box<dyn Nif>, args: u128) -> Option<InterpretResult> {
        let arity = nif.arity();
        if arity.is_some() && arity.unwrap() != args {
            return Some(InterpretResult::RuntimeError);
        }
        match nif.call(self, args as usize) {
            Ok(_) => None,
            _ => Some(InterpretResult::RuntimeError),
        }
    }

    fn call_user_function(
        &mut self,
        function_name: &str,
        scope: u128,
        args: u128,
    ) -> Option<InterpretResult> {
        let callee =
            if let Some((func, _)) = self.resolve_function(&function_name.to_string(), scope) {
                func
            } else if let Some(Value::Function((_, Some(func)))) = self.globals.get(function_name) {
                func.clone()
            } else {
                return Some(InterpretResult::RuntimeError);
            };

        if callee.arity() != args {
            return Some(InterpretResult::RuntimeError);
        }

        let mut substack = vec![];
        for _ in 0..args {
            substack.push(self.stack_pop().unwrap());
        }
        substack.reverse();
        self.stack.push(substack);

        match self.run(callee.clone()) {
            InterpretResult::Ok => None,
            _ => Some(InterpretResult::RuntimeError),
        }
    }

    pub(crate) fn add_constant(&mut self, constant: Value) -> usize {
        self.constants.add(constant)
    }

    pub(crate) fn add_function(&mut self, scope_depth: u128, function: Function) -> usize {
        self.functions.push((function, scope_depth));
        self.functions.len() - 1
    }

    pub(crate) fn function_exists(&self, scope_depth: u128, name: &String) -> bool {
        self.functions
            .iter()
            .any(|(function, scope)| function.name() == *name && *scope == scope_depth)
    }

    pub(crate) fn stack_push(&mut self, value: Value) {
        self.stack.last_mut().unwrap().push(value);
    }

    pub(crate) fn stack_pop(&mut self) -> Option<Value> {
        self.stack.last_mut().unwrap().pop()
    }

    pub(crate) fn stack_peek(&self) -> Option<Value> {
        self.stack.last().unwrap().last().cloned()
    }

    pub(crate) fn stack_get(&self, address: usize) -> Option<Value> {
        self.stack.last().unwrap().get(address).cloned()
    }

    pub(crate) fn stack_insert(&mut self, address: usize, value: Value) {
        let frame = self.stack.last_mut().unwrap();
        frame.remove(address);
        frame.insert(address, value);
    }

    pub(crate) fn start_time(&self) -> Instant {
        self.start_time
    }

    pub(crate) fn resolve_function(
        &self,
        name: &String,
        given_scope: u128,
    ) -> Option<(Function, usize)> {
        self.functions
            .iter()
            .enumerate()
            .rev()
            .find(|(_, (function, scope))| (function.name() == *name) && *scope <= given_scope)
            .map(|(address, (function, _))| (function.clone(), address))
    }

    #[cfg(test)]
    pub(crate) fn get_stdout(&mut self) -> &mut Vec<String> {
        &mut self.stdout
    }

    fn get_constant(&self, address: usize) -> Option<&Value> {
        self.constants.get(address)
    }

    fn is_falsey(&self, value: &Value) -> Option<bool> {
        match value {
            Value::String(value) if value.is_empty() => Some(true),
            Value::Number(value) if *value == 0.0 => Some(true),
            Value::Boolean(value) => Some(!value),
            Value::Number(_) => Some(false),
            Value::String(_) => Some(false),
            Value::Nil => Some(true),
            _ => None,
        }
    }
}
