use std::time::{Instant, SystemTime};

use crate::value::Value;
use crate::vm::VM;

pub(crate) trait Nif {
    fn arity(&self) -> Option<u128>;
    fn name(&self) -> String;
    fn call(&self, vm: &mut VM, args_count: usize);
}

pub(crate) fn resolve_nif(name: &String) -> Option<Box<dyn Nif>> {
    match name.as_str() {
        "now" => Some(Box::new(Now)),
        "print" => Some(Box::new(Print)),
        "vm_now" => Some(Box::new(VMNow)),
        "println" => Some(Box::new(PrintLn)),
        _ => None,
    }
}

pub(crate) struct Now;
pub(crate) struct VMNow;
pub(crate) struct Print;
pub(crate) struct PrintLn;

impl Nif for Now {
    fn arity(&self) -> Option<u128> {
        Some(0)
    }

    fn name(&self) -> String {
        "now".into()
    }

    fn call(&self, vm: &mut VM, _args_count: usize) {
        let now = SystemTime::now();
        let elapsed = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let elapsed = elapsed.as_nanos();
        vm.stack_push(Value::Double(elapsed as f64));
    }
}

impl Nif for Print {
    fn arity(&self) -> Option<u128> {
        None
    }

    fn name(&self) -> String {
        "print".into()
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

        args.iter().for_each(|value| vm.stack_push(value.clone()));
    }
}

impl Nif for PrintLn {
    fn arity(&self) -> Option<u128> {
        None
    }

    fn name(&self) -> String {
        "println".into()
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

impl Nif for VMNow {
    fn arity(&self) -> Option<u128> {
        Some(0)
    }

    fn name(&self) -> String {
        "vm_now".into()
    }

    fn call(&self, vm: &mut VM, _args_count: usize) {
        let start_time = vm.start_time();
        let now = Instant::now();
        let elapsed = now.duration_since(start_time);
        let elapsed = elapsed.as_nanos();
        vm.stack_push(Value::Double(elapsed as f64));
    }
}
