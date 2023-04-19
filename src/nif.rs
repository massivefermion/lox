use std::time::Instant;

use crate::error::InterpretResult;
use crate::value::Value;
use crate::vm::VM;

pub(crate) trait NIF {
    fn name(&self) -> String;
    fn arity(&self) -> Option<u128>;
    fn call(&self, vm: &mut VM, args_count: usize) -> Result<(), InterpretResult>;
}

pub(crate) fn resolve_nif(name: &String) -> Option<Box<dyn NIF>> {
    match name.as_str() {
        "div" => Some(Box::new(Div)),
        "clock" => Some(Box::new(Clock)),
        "parse" => Some(Box::new(Parse)),
        "print" => Some(Box::new(Print)),
        "type_of" => Some(Box::new(TypeOf)),
        "println" => Some(Box::new(PrintLn)),
        _ => None,
    }
}

struct Div;
struct Clock;
struct Parse;
struct Print;
struct TypeOf;
struct PrintLn;

impl NIF for Div {
    fn name(&self) -> String {
        "div".into()
    }

    fn arity(&self) -> Option<u128> {
        Some(2)
    }

    fn call(&self, vm: &mut VM, _args_count: usize) -> Result<(), InterpretResult> {
        let right = vm.stack_pop().unwrap();
        let left = vm.stack_pop().unwrap();

        match (left.clone(), right.clone()) {
            (Value::Number(_), Value::Number(_)) => {
                let left: i128 = left.into();
                let right: i128 = right.into();
                vm.stack_push(Value::Number((left / right) as f64));
                Ok(())
            }
            _ => Err(InterpretResult::RuntimeError),
        }
    }
}

impl NIF for Clock {
    fn name(&self) -> String {
        "clock".into()
    }

    fn arity(&self) -> Option<u128> {
        Some(0)
    }

    fn call(&self, vm: &mut VM, _args_count: usize) -> Result<(), InterpretResult> {
        let start_time = vm.start_time();
        let now = Instant::now();
        let elapsed = now.duration_since(start_time);
        let elapsed = elapsed.as_nanos();
        vm.stack_push(Value::Number(elapsed as f64));
        Ok(())
    }
}

impl NIF for Parse {
    fn name(&self) -> String {
        "parse".to_string()
    }

    fn arity(&self) -> Option<u128> {
        Some(1)
    }

    fn call(&self, vm: &mut VM, _args_count: usize) -> Result<(), InterpretResult> {
        let arg = vm.stack_pop().unwrap();

        let result = match arg {
            Value::String(value) if value.as_str() == "true" => Value::Boolean(true),
            Value::String(value) if value.as_str() == "false" => Value::Boolean(false),
            Value::String(value) if value.parse::<f64>().is_ok() => {
                Value::Number(value.parse::<f64>().unwrap())
            }
            value => value,
        };

        vm.stack_push(result);
        Ok(())
    }
}

impl NIF for Print {
    fn name(&self) -> String {
        "print".into()
    }

    fn arity(&self) -> Option<u128> {
        None
    }

    fn call(&self, vm: &mut VM, args_count: usize) -> Result<(), InterpretResult> {
        let mut args = vec![];

        for _ in 0..args_count {
            args.push(vm.stack_pop().unwrap());
        }

        match vm.get_stdout() {
            Some(stdout) => {
                args.iter()
                    .rev()
                    .map(|item| {
                        let item: String = item.clone().into();
                        item
                    })
                    .for_each(|item| stdout.push(item.clone()));
            }

            None => {
                args.iter()
                    .rev()
                    .map(|item| {
                        let item: String = item.clone().into();
                        item
                    })
                    .for_each(|item| print!("{}", item));
            }
        };
        Ok(())
    }
}

impl NIF for TypeOf {
    fn name(&self) -> String {
        "type_of".into()
    }

    fn arity(&self) -> Option<u128> {
        Some(1)
    }

    fn call(&self, vm: &mut VM, _args_count: usize) -> Result<(), InterpretResult> {
        let arg = vm.stack_pop().unwrap();

        let value_type = match arg {
            Value::Nil => "nil",
            Value::String(_) => "string",
            Value::Number(_) => "number",
            Value::Boolean(_) => "boolean",
            Value::Function(_) => "function",
        };

        vm.stack_push(Value::String(value_type.into()));
        Ok(())
    }
}

impl NIF for PrintLn {
    fn name(&self) -> String {
        "println".into()
    }

    fn arity(&self) -> Option<u128> {
        None
    }

    fn call(&self, vm: &mut VM, args_count: usize) -> Result<(), InterpretResult> {
        let _ = Print.call(vm, args_count);
        match vm.get_stdout() {
            Some(stdout) => {
                stdout.push("\n".to_string());
            }

            None => {
                println!();
            }
        };
        Ok(())
    }
}
