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
    stack: Vec<Vec<Value>>,
    constants: Chunk<Value>,
    globals: HashMap<String, Value>,
    functions: Vec<(Function, u128)>,
    locals: u128,
    start_time: Instant,
}

impl VM {
    pub(crate) fn new() -> VM {
        VM {
            stack: vec![vec![]],
            constants: Chunk::new(),
            globals: HashMap::new(),
            functions: vec![],
            locals: 0,
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
        let debug = if var_os("DEBUG").is_some() {
            true
        } else {
            false
        };

        if debug {
            println!("Constants\n{:?}\n", self.constants);
            println!("OpCodes\n{:?}", function);
        }

        let mut iterator = function.into_iter().peekable();
        while let Some(current) = iterator.next() {
            let op_code = OpCode::from(current as u8);

            if debug {
                println!("\nOpCode\n{:?}", op_code);
                println!("\nStack\n{:?}\n", self.stack);
            }

            match op_code {
                OpCode::Return => {
                    let return_value = match self.stack_pop() {
                        Some(value) => value,
                        None => Value::Nil,
                    };
                    self.stack.pop();
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
                    self.stack_push(constant.clone());
                }

                OpCode::Negate => {
                    let Some(Value::Double(value)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_push(Value::Double(-value));
                }

                OpCode::Not => {
                    let Some(value) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };

                    match value {
                        Value::Nil => self.stack_push(Value::Boolean(true)),
                        Value::Boolean(value) => self.stack_push(Value::Boolean(!value)),

                        Value::Double(value) if value == 0.0 => {
                            self.stack_push(Value::Boolean(true))
                        }
                        Value::Double(_) => self.stack_push(Value::Boolean(false)),

                        Value::String(value) if value.is_empty() => {
                            self.stack_push(Value::Boolean(true))
                        }
                        Value::String(_) => self.stack_push(Value::Boolean(false)),
                        // _ => return InterpretResult::RuntimeError,
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
                    let Some(Value::Double(right)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(Value::Double(left)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };

                    self.stack_push(Value::Double(left + right))
                }

                OpCode::Multiply => {
                    let Some(Value::Double(right)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(Value::Double(left)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_push(Value::Double(left * right))
                }

                OpCode::Divide => {
                    let Some(Value::Double(right)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(Value::Double(left)) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_push(Value::Double(left / right))
                }

                // OpCode::Print => {
                //     let Some(value) = self.stack_pop() else {
                //         return InterpretResult::RuntimeError;
                //     };

                //     let value: String = value.into();
                //     match iterator.peek() {
                //         Some(op_code) if OpCode::from(*op_code as u8) == OpCode::NewLine => {
                //             iterator.next();
                //             println!("{}", value);
                //         }
                //         Some(_) => print!("{}", value),
                //         // None => return InterpretResult::RuntimeError,
                //         None => (),
                //     }
                // }
                OpCode::Pop => {
                    self.stack_pop();
                }

                OpCode::Nil => {
                    self.stack_push(Value::Nil);
                }

                OpCode::DefGlobal => {
                    iterator.next();
                    let Some( address) = iterator.next() else {
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

                    match self.globals.insert(variable_name, value) {
                        None => return InterpretResult::RuntimeError,
                        _ => (),
                    }
                }

                OpCode::GetGlobal => {
                    iterator.next();
                    let Some( address) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(Value::String(variable_name)) = self.get_constant(address) else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(value) = self.globals.get(variable_name) else {
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_push(value.clone());
                }

                OpCode::SetLocal => {
                    let Some( address) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(value) = self.stack_peek() else {
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_insert(address, value);
                }

                OpCode::GetLocal => {
                    let Some(address) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };

                    let Some(value) = self.stack_get(address) else {
                        return InterpretResult::RuntimeError;
                    };
                    self.locals += 1;
                    self.stack_push(value.clone());
                }

                OpCode::JumpIfFalse => {
                    let Some(value) = self.stack_peek() else {
                        return InterpretResult::RuntimeError;
                    };

                    let is_falsey = match value {
                        Value::String(value) if value.is_empty() => true,
                        Value::Double(value) if value == 0.0 => true,
                        Value::Boolean(value) => !value,
                        Value::Double(_) => false,
                        Value::String(_) => false,
                        Value::Nil => true,
                        // _ => return InterpretResult::RuntimeError,
                    };

                    let Some( size) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };

                    if is_falsey {
                        for _ in 0..size {
                            iterator.next();
                        }
                    }
                }

                OpCode::Jump => {
                    let  Some( size) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };

                    for _ in 0..size {
                        iterator.next();
                    }
                }

                OpCode::ClearScope => {
                    let Some(given_scope) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };

                    self.functions = self
                        .functions
                        .iter()
                        .filter(|(_, scope)| *scope as usize != given_scope)
                        .map(|item| item.clone())
                        .collect();
                }

                OpCode::Call => {
                    iterator.next();
                    let Some(address) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(Value::Double(args)) = self.get_constant(address as usize) else {
                        return InterpretResult::RuntimeError;
                    };
                    let args = *args as u128;

                    iterator.next();
                    let Some(address) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(Value::String(function_name)) = self.get_constant(address as usize) else {
                        return InterpretResult::RuntimeError;
                    };
                    let function_name = function_name.clone();

                    let mut substack = vec![];
                    for _ in 0..args {
                        substack.push(self.stack_pop().unwrap());
                    }
                    substack.reverse();
                    self.stack.push(substack);

                    match resolve_nif(&function_name) {
                        Some(nif) => {
                            let arity = nif.arity();

                            if arity.is_some() && arity.unwrap() != args {
                                return InterpretResult::RuntimeError;
                            }

                            nif.call(self, args as usize);
                        }

                        None => {
                            let Some(function) = self.resolve_function(&function_name) else {
                                return InterpretResult::RuntimeError;
                            };

                            if function.arity() != args {
                                return InterpretResult::RuntimeError;
                            }

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

    pub(crate) fn add_function(&mut self, scope_depth: u128, function: Function) {
        self.functions.push((function, scope_depth));
    }

    pub(crate) fn function_exists(&self, scope_depth: u128, name: &String) -> bool {
        self.functions
            .iter()
            .find(|(function, scope)| function.name() == *name && *scope == scope_depth)
            .is_some()
    }

    pub(crate) fn stack_push(&mut self, value: Value) {
        self.stack.last_mut().unwrap().push(value);
    }

    pub(crate) fn stack_pop(&mut self) -> Option<Value> {
        self.stack.last_mut().unwrap().pop()
    }

    pub(crate) fn stack_peek(&mut self) -> Option<Value> {
        self.stack.last().unwrap().last().map(|v| v.clone())
    }

    pub(crate) fn stack_get(&self, address: usize) -> Option<Value> {
        self.stack
            .last()
            .unwrap()
            .get(address)
            .map(|value| value.clone())
    }

    pub(crate) fn stack_insert(&mut self, address: usize, value: Value) {
        let frame = self.stack.last_mut().unwrap();
        frame.remove(address);
        frame.insert(address, value);
    }

    pub(crate) fn start_time(&self) -> Instant {
        self.start_time
    }

    fn get_constant(&self, address: usize) -> Option<&Value> {
        self.constants.get(address)
    }

    fn resolve_function(&self, name: &String) -> Option<Function> {
        self.functions
            .iter()
            .rev()
            .find(|(function, _)| function.name() == *name)
            .map(|(function, _)| function.clone())
    }
}
