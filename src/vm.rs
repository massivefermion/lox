use std::collections::HashMap;
use std::env::var_os;
use std::time::Instant;

use crate::chunk::Chunk;
use crate::compiler::Compiler;
use crate::error::InterpretResult;
use crate::function::Function;
use crate::nif::resolve_nif;
use crate::op::OpCode;
use crate::value::Value;

pub(crate) struct VM {
    #[cfg(test)]
    pub stdout: Vec<String>,

    start_time: Instant,
    stack: Vec<Vec<Value>>,
    constants: Chunk<Value>,
    globals: HashMap<String, Value>,
    functions: Vec<(Function, u128)>,
    loops: HashMap<String, Function>,
}

impl VM {
    pub(crate) fn new() -> VM {
        VM {
            #[cfg(test)]
            stdout: vec![],

            functions: vec![],
            stack: vec![vec![]],
            loops: HashMap::new(),
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

        let mut iterator = function.into_iter().peekable();
        while let Some(current) = iterator.next() {
            let op_code = OpCode::from(current as u8);

            if debug {
                println!("\n{} OpCode\n{:?}", function, op_code);
                println!("\n{}", self.stack.len());
                if self.stack.len() > 1 {
                    println!("{:#?}", self.stack.get(self.stack.len() - 2));
                }
                println!("{:#?}", self.stack.last());
            }

            match op_code {
                OpCode::Return => {
                    let return_value = match self.stack_pop() {
                        Some(value) => value,
                        None => Value::Nil,
                    };

                    if let Value::Function((address, _)) = return_value {
                        if let Some(returned_function) = self.functions.get_mut(address).cloned() {
                            self.functions.remove(address);
                            self.functions.insert(
                                address,
                                (returned_function.clone().0, returned_function.1 - 1),
                            );
                        };
                    };

                    if !function.is_loop() {
                        self.stack.pop();
                    }
                    self.stack_push(return_value);

                    return InterpretResult::Ok;
                }

                OpCode::Constant => {
                    let Some(address) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(constant) = self.get_constant(address) else {
                        return InterpretResult::RuntimeError;
                    };

                    match constant {
                        Value::Function((address, None)) => {
                            if let Some((function, _)) = self.functions.get(*address) {
                                self.stack_push(Value::Function((*address, Some(function.clone()))))
                            } else {
                                return InterpretResult::RuntimeError;
                            }
                        }
                        _ => self.stack_push(constant.clone()),
                    }
                }

                OpCode::Negate => {
                    let Some(Value::Number(value)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_push(Value::Number(-value));
                }

                OpCode::Not => {
                    let Some(value) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };

                    match value {
                        Value::Nil => self.stack_push(Value::Boolean(true)),
                        Value::Boolean(value) => self.stack_push(Value::Boolean(!value)),

                        Value::Number(value) if value == 0.0 => {
                            self.stack_push(Value::Boolean(true))
                        }
                        Value::Number(_) => self.stack_push(Value::Boolean(false)),

                        Value::String(value) if value.is_empty() => {
                            self.stack_push(Value::Boolean(true))
                        }
                        Value::String(_) => self.stack_push(Value::Boolean(false)),
                        _ => return InterpretResult::RuntimeError,
                    }
                }

                OpCode::Concat => {
                    let Some(right) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(left) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };

                    let right: String = right.into();
                    let left: String = left.into();

                    self.stack_push(Value::String(left + &right))
                }

                OpCode::Add => {
                    let Some(Value::Number(right)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(Value::Number(left)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };

                    self.stack_push(Value::Number(left + right))
                }

                OpCode::Multiply => {
                    let Some(Value::Number(right)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(Value::Number(left)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_push(Value::Number(left * right))
                }

                OpCode::Rem => {
                    let Some(Value::Number(right)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(Value::Number(left)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_push(Value::Number(left % right))
                }

                OpCode::Divide => {
                    let Some(Value::Number(right)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(Value::Number(left)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_push(Value::Number(left / right))
                }

                OpCode::Equal => {
                    let Some(right) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(left) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_push(Value::Boolean(left == right));
                }

                OpCode::NotEqual => {
                    let Some(right) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(left) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_push(Value::Boolean(left != right));
                }

                OpCode::GreaterEqual => {
                    let Some(right) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(left) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_push(Value::Boolean(left >= right));
                }

                OpCode::Greater => {
                    let Some(right) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(left) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_push(Value::Boolean(left > right));
                }

                OpCode::LessEqual => {
                    let Some(right) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(left) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_push(Value::Boolean(left <= right));
                }

                OpCode::Less => {
                    let Some(right) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(left) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_push(Value::Boolean(left < right));
                }

                OpCode::Pop => {
                    self.stack_pop();
                }

                OpCode::Nil => {
                    self.stack_push(Value::Nil);
                }

                OpCode::MakeClosure => {
                    iterator.next();
                    let Some(address) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(Value::Number(address)) = self.get_constant(address) else {
                        return InterpretResult::RuntimeError;
                    };

                    let address = *address;
                    let Some((ref mut function, _)) = self.functions.get_mut(address as usize) else {
                        return InterpretResult::RuntimeError;
                    };

                    function
                        .captures()
                        .iter()
                        .for_each(|(name, (frame, address, _))| {
                            function.populate_capture(
                                name.clone(),
                                self.stack
                                    .get(*frame)
                                    .unwrap()
                                    .get(*address)
                                    .cloned()
                                    .unwrap(),
                            );
                        });
                }

                OpCode::GetCaptured => {
                    iterator.next();
                    let Some(address) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(Value::String(variable_name)) = self.get_constant(address) else {
                        return InterpretResult::RuntimeError;
                    };

                    let Some(value) = function.get_capture(variable_name.clone()) else {                            
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_push(value.clone());
                }

                OpCode::DefGlobal => {
                    iterator.next();
                    let Some(address) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(Value::String(variable_name)) = self.get_constant(address) else {
                        return InterpretResult::RuntimeError;
                    };
                    let variable_name = variable_name.clone();

                    let Some(value) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };

                    self.globals.insert(variable_name, value.clone());
                }

                OpCode::SetGlobal => {
                    iterator.next();
                    let Some(address) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(Value::String(variable_name)) = self.get_constant(address) else {
                        return InterpretResult::RuntimeError;
                    };
                    let variable_name = variable_name.clone();

                    let Some(value) = self.stack_peek() else {
                        return InterpretResult::RuntimeError;
                    };

                    if self.globals.insert(variable_name, value).is_none() {
                        return InterpretResult::RuntimeError;
                    };
                }

                OpCode::GetGlobal => {
                    iterator.next();
                    let Some(address) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(Value::String(variable_name)) = self.get_constant(address) else {
                        return InterpretResult::RuntimeError;
                    };
                    let variable_name = variable_name.clone();

                    let Some(value) = self.globals.get(&variable_name) else {
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_push(value.clone());
                }

                OpCode::GetLocal => {
                    let Some(address) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(value) = self.stack_get(address) else {
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_push(value.clone());
                }

                OpCode::SetLocal => {
                    let Some(address) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(value) = self.stack_peek() else {
                        return InterpretResult::RuntimeError;
                    };

                    self.stack_insert(address, value);
                }

                OpCode::JumpIfFalse => {
                    let Some(value) = self.stack_peek() else {
                        return InterpretResult::RuntimeError;
                    };

                    let is_falsey = match self.is_falsey(&value) {
                        Some(result) => result,
                        None => return InterpretResult::RuntimeError,
                    };

                    let Some(size) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };

                    if is_falsey {
                        for _ in 0..size {
                            iterator.next();
                        }
                    }
                }

                OpCode::Jump => {
                    let  Some(size) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };

                    for _ in 0..size {
                        iterator.next();
                    }
                }

                OpCode::Loop => {
                    iterator.next();
                    let Some(address) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(Value::String(loop_name)) = self.get_constant(address) else {
                        return InterpretResult::RuntimeError;
                    };

                    let Some(lp) = self.get_loop(loop_name) else {
                        return InterpretResult::RuntimeError;
                    };

                    self.stack.push(vec![]);
                    let name = lp.name().clone();
                    match self.run(lp) {
                        InterpretResult::Ok => (),
                        _ => return InterpretResult::RuntimeError,
                    };
                    self.remove_loop(&name);
                }

                OpCode::Call => {
                    iterator.next();
                    let Some(address) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(Value::Number(scope)) = self.get_constant(address) else {
                        return InterpretResult::RuntimeError;
                    };
                    let scope = *scope as u128;

                    iterator.next();
                    let Some(address) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(Value::Number(args)) = self.get_constant(address) else {
                        return InterpretResult::RuntimeError;
                    };
                    let args = *args as u128;

                    iterator.next();
                    let Some(address) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(Value::String(function_name)) = self.get_constant(address) else {
                        return InterpretResult::RuntimeError;
                    };
                    let function_name = function_name.clone();

                    match resolve_nif(&function_name) {
                        Some(nif) => {
                            let arity = nif.arity();

                            if arity.is_some() && arity.unwrap() != args {
                                return InterpretResult::RuntimeError;
                            }

                            match nif.call(self, args as usize) {
                                Ok(_) => (),
                                _ => return InterpretResult::RuntimeError,
                            }
                        }

                        None => {
                            let function = self.resolve_function(&function_name, scope);

                            if function.is_none() {
                                return InterpretResult::RuntimeError;
                            }

                            let (function, _) = function.unwrap();

                            if function.arity() != args {
                                return InterpretResult::RuntimeError;
                            }

                            let mut substack = vec![];
                            for _ in 0..args {
                                substack.push(self.stack_pop().unwrap());
                            }
                            substack.reverse();
                            self.stack.push(substack);

                            match self.run(function.clone()) {
                                InterpretResult::Ok => (),
                                _ => return InterpretResult::RuntimeError,
                            }
                        }
                    }
                }

                _ => return InterpretResult::CompileError,
            }
        }

        InterpretResult::Ok
    }

    pub(crate) fn add_constant(&mut self, constant: Value) -> usize {
        self.constants.add(constant)
    }

    pub(crate) fn add_function(&mut self, scope_depth: u128, function: Function) -> usize {
        self.functions.push((function, scope_depth));
        self.functions.len() - 1
    }

    pub(crate) fn add_loop(&mut self, lp: Function) {
        self.loops.insert(lp.name(), lp);
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

    pub(crate) fn stack_peek(&mut self) -> Option<Value> {
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
            .find(|(_, (function, scope))| function.name() == *name && *scope <= given_scope)
            .map(|(address, (function, _))| (function.clone(), address))
    }

    #[cfg(test)]
    pub(crate) fn get_stdout(&mut self) -> &mut Vec<String> {
        &mut self.stdout
    }

    fn get_constant(&self, address: usize) -> Option<&Value> {
        self.constants.get(address)
    }

    fn get_loop(&self, name: &String) -> Option<Function> {
        self.loops.get(name).cloned()
    }

    fn remove_loop(&mut self, name: &String) {
        self.loops.remove(name);
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
