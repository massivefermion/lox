use std::time::Instant;

use crate::value::Value;
use crate::vm::VM;

pub(crate) trait Nif {
    fn arity(&self) -> Option<u128>;
    fn name(&self) -> String;
    fn call(&self, vm: &mut VM, args_count: usize);
}

pub(crate) fn resolve_nif(name: &String) -> Option<Box<dyn Nif>> {
    match name.as_str() {
        "clock" => Some(Box::new(Clock)),
        "print" => Some(Box::new(Print)),
        "println" => Some(Box::new(PrintLn)),
        _ => None,
    }
}

pub(crate) struct Clock;
pub(crate) struct Print;
pub(crate) struct PrintLn;

impl Nif for Clock {
    fn arity(&self) -> Option<u128> {
        Some(0)
    }

    fn name(&self) -> String {
        "clock".to_string()
    }

    fn call(&self, vm: &mut VM, _args_count: usize) {
        let start_time = vm.start_time();
        let now = Instant::now();
        let elapsed = now.duration_since(start_time);
        let elapsed = elapsed.as_nanos();
        vm.stack_push(Value::Double(elapsed as f64));
    }
}

impl Nif for Print {
    fn arity(&self) -> Option<u128> {
        None
    }

    fn name(&self) -> String {
        "print".to_string()
    }

    fn call(&self, vm: &mut VM, args_count: usize) {
        let mut args = vec![];

        for _ in 0..args_count {
            args.push(vm.stack_peek().unwrap());
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

impl Nif for PrintLn {
    fn arity(&self) -> Option<u128> {
        None
    }

    fn name(&self) -> String {
        "println".to_string()
    }

    fn call(&self, vm: &mut VM, args_count: usize) {
        let mut args = vec![];

        for _ in 0..args_count {
            args.push(vm.stack_peek().unwrap());
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
                stdout.push("\n".to_string());
            }
            None => {
                args.iter()
                    .rev()
                    .map(|item| {
                        let item: String = item.clone().into();
                        item
                    })
                    .for_each(|item| print!("{}", item));
                println!();
            }
        };
    }
}
