use std::time::Instant;

use crate::value::Value;
use crate::vm::VM;

pub(crate) trait NIF {
    fn name(&self) -> String;
    fn arity(&self) -> Option<u128>;
    fn call(&self, vm: &mut VM, args_count: usize);
}

pub(crate) fn resolve_nif(name: &String) -> Option<Box<dyn NIF>> {
    match name.as_str() {
        "clock" => Some(Box::new(Clock)),
        "parse" => Some(Box::new(Parse)),
        "print" => Some(Box::new(Print)),
        "type_of" => Some(Box::new(TypeOf)),
        "println" => Some(Box::new(PrintLn)),
        _ => None,
    }
}

struct Clock;
struct Parse;
struct Print;
struct TypeOf;
struct PrintLn;

impl NIF for Clock {
    fn name(&self) -> String {
        "clock".into()
    }

    fn arity(&self) -> Option<u128> {
        Some(0)
    }

    fn call(&self, vm: &mut VM, _args_count: usize) {
        let start_time = vm.start_time();
        let now = Instant::now();
        let elapsed = now.duration_since(start_time);
        let elapsed = elapsed.as_nanos();
        vm.stack_push(Value::Double(elapsed as f64));
    }
}

impl NIF for Parse {
    fn name(&self) -> String {
        "parse".to_string()
    }

    fn arity(&self) -> Option<u128> {
        Some(1)
    }

    fn call(&self, vm: &mut VM, _args_count: usize) {
        let arg = vm.stack_pop().unwrap();

        let result = match arg {
            Value::String(value) if value.as_str() == "true" => Value::Boolean(true),
            Value::String(value) if value.as_str() == "false" => Value::Boolean(false),
            Value::String(value) if value.parse::<f64>().is_ok() => {
                Value::Double(value.parse::<f64>().unwrap())
            }
            value => value,
        };

        vm.stack_push(result);
    }
}

impl NIF for Print {
    fn name(&self) -> String {
        "print".into()
    }

    fn arity(&self) -> Option<u128> {
        None
    }

    fn call(&self, vm: &mut VM, args_count: usize) {
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
    }
}

impl NIF for TypeOf {
    fn name(&self) -> String {
        "type_of".into()
    }

    fn arity(&self) -> Option<u128> {
        Some(1)
    }

    fn call(&self, vm: &mut VM, _args_count: usize) {
        let arg = vm.stack_pop().unwrap();

        let value_type = match arg {
            Value::Nil => "nil",
            Value::String(_) => "string",
            Value::Double(_) => "number",
            Value::Boolean(_) => "boolean",
            Value::Function(_) => "function",
        };

        vm.stack_push(Value::String(value_type.into()));
    }
}

impl NIF for PrintLn {
    fn name(&self) -> String {
        "println".into()
    }

    fn arity(&self) -> Option<u128> {
        None
    }

    fn call(&self, vm: &mut VM, args_count: usize) {
        Print.call(vm, args_count);
        match vm.get_stdout() {
            Some(stdout) => {
                stdout.push("\n".to_string());
            }

            None => {
                println!();
            }
        };
    }
}
