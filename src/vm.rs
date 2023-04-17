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
    loops: HashMap<String, Function>,
    start_time: Instant,
    stdout: Option<Vec<String>>,
}

impl VM {
    pub(crate) fn new() -> VM {
        VM {
            stack: vec![vec![]],
            constants: Chunk::new(),
            globals: HashMap::new(),
            functions: vec![],
            loops: HashMap::new(),
            start_time: Instant::now(),
            stdout: None,
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
            println!("{} => Constants\n{:?}\n", function, self.constants);
        }

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

                OpCode::EqualEqual => {
                    let Some(right) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(left) = self.stack_pop() else {
                        return InterpretResult::RuntimeError;
                    };
                    self.stack_push(Value::Boolean(left == right));
                }

                OpCode::BangEqual => {
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

                    match self.globals.insert(variable_name, value) {
                        None => return InterpretResult::RuntimeError,
                        _ => (),
                    }
                }

                OpCode::GetGlobal => {
                    iterator.next();
                    let Some(address) = iterator.next() else {
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
                    let Some(address) = iterator.next() else {
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
                    self.stack_push(value.clone());
                }

                OpCode::JumpIfFalse => {
                    let Some(value) = self.stack_pop() else {
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
                    let Some(Value::String(loop_name)) = self.get_constant(address as usize) else {
                        return InterpretResult::RuntimeError;
                    };

                    let Some(lp) = self.get_loop(&loop_name) else {
                        return InterpretResult::RuntimeError;
                    };

                    let mut substack = vec![];
                    substack.push(self.stack_pop().unwrap());
                    self.stack.push(substack);
                    let name = lp.name().clone();

                    match self.run(lp) {
                        InterpretResult::Ok => {
                            self.stack.pop();
                            self.remove_loop(&name);
                        }
                        _ => return InterpretResult::RuntimeError,
                    }
                }

                OpCode::Call => {
                    iterator.next();
                    let Some(address) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(Value::Double(scope)) = self.get_constant(address as usize) else {
                        return InterpretResult::RuntimeError;
                    };
                    let scope = *scope as u128;

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

                    match resolve_nif(&function_name) {
                        Some(nif) => {
                            let arity = nif.arity();

                            if arity.is_some() && arity.unwrap() != args {
                                return InterpretResult::RuntimeError;
                            }

                            nif.call(self, args as usize);
                        }

                        None => {
                            let Some(function) = self.resolve_function(&function_name, scope) else {
                                return InterpretResult::RuntimeError;
                            };

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

                OpCode::ClearScope => {
                    iterator.next();
                    let Some(address) = iterator.next() else {
                        return InterpretResult::RuntimeError;
                    };
                    let Some(Value::Double(scope)) = self.get_constant(address as usize) else {
                        return InterpretResult::RuntimeError;
                    };
                    let scope = *scope as u128;
                    self.clear_scope_functions(scope);
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

    pub(crate) fn add_loop(&mut self, lp: Function) {
        self.loops.insert(lp.name(), lp);
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

    pub(crate) fn resolve_function(&self, name: &String, given_scope: u128) -> Option<Function> {
        match self.get_function_from_functions(name, given_scope) {
            Some(function) => Some(function),
            None => self.get_function_from_constants(name),
        }
    }

    pub(crate) fn get_stdout(&mut self) -> Option<&mut Vec<String>> {
        self.stdout.as_mut()
    }

    fn get_function_from_functions(&self, name: &String, given_scope: u128) -> Option<Function> {
        self.functions
            .iter()
            .rev()
            .find(|(function, scope)| function.name() == *name && *scope <= given_scope)
            .map(|(function, _)| function.clone())
    }

    fn get_function_from_constants(&self, name: &String) -> Option<Function> {
        self.constants
            .into_iter()
            .filter(|value| match value {
                Value::Function(_) => true,
                _ => false,
            })
            .find(|function| {
                let Value::Function(function) = function else {
                    panic!();
                };
                function.name() == *name
            })
            .map(|function| {
                let Value::Function(function) = function else {
                    panic!();
                };
                return function.clone();
            })
    }

    fn clear_scope_functions(&mut self, given_scope: u128) {
        self.functions = self
            .functions
            .iter()
            .filter(|(_, scope)| *scope != given_scope)
            .map(|item| item.clone())
            .collect();
    }

    fn get_constant(&self, address: usize) -> Option<&Value> {
        self.constants.get(address)
    }

    fn get_loop(&self, name: &String) -> Option<Function> {
        self.loops.get(name).map(|lp| lp.clone())
    }

    fn remove_loop(&mut self, name: &String) {
        self.loops.remove(name);
    }

    fn is_falsey(&self, value: &Value) -> Option<bool> {
        match value {
            Value::String(value) if value.is_empty() => Some(true),
            Value::Double(value) if *value == 0.0 => Some(true),
            Value::Boolean(value) => Some(!value),
            Value::Double(_) => Some(false),
            Value::String(_) => Some(false),
            Value::Nil => Some(true),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::time::Instant;

    use crate::chunk::Chunk;
    use crate::error::InterpretResult;

    fn new_for_test() -> super::VM {
        super::VM {
            stack: vec![vec![]],
            constants: Chunk::new(),
            globals: HashMap::new(),
            functions: vec![],
            loops: HashMap::new(),
            start_time: Instant::now(),
            stdout: Some(vec![]),
        }
    }

    #[test]
    fn hello_world() {
        let mut vm = new_for_test();
        assert_eq!(
            vm.interpret(r#"println("Hello World!");"#.to_string()),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout.unwrap(), vec!["Hello World!", "\n"]);
    }

    #[test]
    fn scope() {
        let mut vm = new_for_test();
        assert_eq!(
            vm.interpret(
                r#"
                    let a = "global";
                    {
                        let b = "local";
                        print(a);
                        println(b);
                    }
                    print(b);
                "#
                .to_string()
            ),
            InterpretResult::RuntimeError
        );
        assert_eq!(vm.stdout.unwrap(), vec!["global", "local", "\n"]);
    }

    #[test]
    fn nested_scope() {
        let mut vm = new_for_test();
        assert_eq!(
            vm.interpret(
                r#"
                    let a = "global";
                    {
                        let b = "local1";
                        {
                            let c = "local2";
                            print(a, b);
                            println(c);
                        }
                        print(c);
                    }
                    print(b);
                "#
                .to_string()
            ),
            InterpretResult::RuntimeError
        );
        assert_eq!(vm.stdout.unwrap(), vec!["global", "local1", "local2", "\n"]);
    }

    #[test]
    fn simple_function() {
        let mut vm = new_for_test();
        assert_eq!(
            vm.interpret(
                r#"
                    fun add(a, b) {
                        return a+b;
                    }
                    print(add(2, 3));
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout.unwrap(), vec!["5"]);
    }

    #[test]
    fn function_with_no_return() {
        let mut vm = new_for_test();
        assert_eq!(
            vm.interpret(
                r#"
                    fun add(a, b) {
                        println(a+b);
                    }
                    print(add(2, 3));
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout.unwrap(), vec!["5", "\n", "nil"]);
    }

    #[test]
    fn inner_function() {
        let mut vm = new_for_test();
        assert_eq!(
            vm.interpret(
                r#"
                    fun outer() {
                        fun inner() {
                            println("inside");
                        }
                        inner();
                    }
                    outer();
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout.unwrap(), vec!["inside", "\n"]);
    }

    #[test]
    fn inner_function_only_seen_inside() {
        let mut vm = new_for_test();
        assert_eq!(
            vm.interpret(
                r#"
                    fun outer() {
                        fun inner() {
                            println("inside");
                        }
                        inner();
                    }
                    inner();
                "#
                .to_string()
            ),
            InterpretResult::RuntimeError
        );
    }

    #[test]
    fn local_variable() {
        let mut vm = new_for_test();
        assert_eq!(
            vm.interpret(
                r#"
                    let x = "global";
                    fun outer() {
                        let x = "local";
                        println(x);
                    }
                    outer();
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout.unwrap(), vec!["local", "\n"]);
    }

    #[test]
    fn first_class_function() {
        let mut vm = new_for_test();
        assert_eq!(
            vm.interpret(
                r#"
                    fun creator() {
                        fun join(a, b) {
                            return a <> b;
                        }
                        return join;
                    }
                    let join = creator();
                    println(join("U-", 235));
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout.unwrap(), vec!["U-235", "\n"]);
    }

    #[test]
    fn closure_local() {
        let mut vm = new_for_test();
        assert_eq!(
            vm.interpret(
                r#"
                    let x = "global";
                    fun outer() {
                        let x = "local";
                        fun inner() {
                            println(x);
                        }
                        inner();
                    }
                    outer();
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout.unwrap(), vec!["local", "\n"]);
    }

    #[test]
    fn closure_parameter() {
        let mut vm = new_for_test();
        assert_eq!(
            vm.interpret(
                r#"
                    fun creator(a) {
                        fun join(b) {
                            return a <> b;
                        }
                        return join;
                    }
                    let join = creator("U-");
                    println(235);
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout.unwrap(), vec!["U-235", "\n"]);
    }
}
