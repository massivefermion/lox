use std::time::Instant;

use crate::error::InterpretResult;
use crate::value::Value;
use crate::vm::VM;

pub(crate) trait Nif {
    fn name(&self) -> String;
    fn arity(&self) -> Option<u128>;
    fn call(&self, vm: &mut VM, args_count: usize) -> Result<(), InterpretResult>;
}

pub(crate) fn resolve_nif(name: &str) -> Option<Box<dyn Nif>> {
    match name {
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

impl Nif for Div {
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

impl Nif for Clock {
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

impl Nif for Parse {
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

impl Nif for Print {
    fn name(&self) -> String {
        "print".into()
    }

    fn arity(&self) -> Option<u128> {
        None
    }

    #[cfg(not(test))]
    fn call(&self, vm: &mut VM, args_count: usize) -> Result<(), InterpretResult> {
        let mut args = vec![];

        for _ in 0..args_count {
            args.push(vm.stack_pop().unwrap());
        }

        args.iter()
            .rev()
            .map(|item| {
                let item: String = item.clone().into();
                item
            })
            .for_each(|item| print!("{}", item));

        Ok(())
    }

    #[cfg(test)]
    fn call(&self, vm: &mut VM, args_count: usize) -> Result<(), InterpretResult> {
        let mut args = vec![];

        for _ in 0..args_count {
            args.push(vm.stack_pop().unwrap());
        }

        args.iter()
            .rev()
            .map(|item| {
                let item: String = item.clone().into();
                item
            })
            .for_each(|item| vm.get_stdout().push(item));

        Ok(())
    }
}

impl Nif for TypeOf {
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

impl Nif for PrintLn {
    fn name(&self) -> String {
        "println".into()
    }

    fn arity(&self) -> Option<u128> {
        None
    }

    #[cfg(not(test))]
    fn call(&self, vm: &mut VM, args_count: usize) -> Result<(), InterpretResult> {
        let _ = Print.call(vm, args_count);
        println!();
        Ok(())
    }

    #[cfg(test)]
    fn call(&self, vm: &mut VM, args_count: usize) -> Result<(), InterpretResult> {
        let mut args = vec![];

        for _ in 0..args_count {
            args.push(vm.stack_pop().unwrap());
        }

        args.iter()
            .rev()
            .map(|item| {
                let item: String = item.clone().into();
                item
            })
            .for_each(|item| vm.get_stdout().push(item));
        vm.get_stdout().push("\n".to_string());

        Ok(())
    }
}
