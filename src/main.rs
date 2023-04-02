use std::env;
use std::fs::File;
use std::io::{stdin, BufRead, Read};

use vm::InterpretResult;

mod chunk;
mod compiler;
mod error;
mod op;
mod scanner;
mod token;
mod value;
mod vm;

fn main() -> Result<(), InterpretResult> {
    let mut vm = vm::VM::new();

    // vm.add_op(op::OpCode::Constant, 0);
    vm.add_constant(value::Value::Double(6.0));
    vm.add_op(op::OpCode::Negate, 1);

    // vm.add_op(op::OpCode::Constant, 0);
    vm.add_constant(value::Value::Double(2.0));

    vm.add_op(op::OpCode::Add, 1);

    // vm.add_op(op::OpCode::Constant, 0);
    vm.add_constant(value::Value::Double(4.0));

    vm.add_op(op::OpCode::Add, 1);

    vm.add_op(op::OpCode::Print, 1);
    vm.add_op(op::OpCode::Return, 1);

    vm.run();
    Ok(())

    // let args: Vec<String> = env::args().collect();

    // match &args[..] {
    //     [_] => repl(&mut vm),
    //     [_, path] => run_file(&mut vm, path),
    //     _ => Err(vm::InterpretResult::CliError),
    //     // _ => error::error_out(error::LoxError::new(
    //     //     "Usage: lox [script]",
    //     //     error::ErrorContext::Cli,
    //     //     None,
    //     // )),
    // }
}

fn repl(vm: &mut vm::VM) -> Result<(), InterpretResult> {
    for line in stdin().lock().lines() {
        print!("lox -> ");
        match line {
            Ok(line) => match vm.interpret(line) {
                vm::InterpretResult::Ok => println!("done!"),
                result => eprintln!("{:?}", result),
            },
            Err(_) => break,
        }
    }
    Ok(())
}

fn run_file(vm: &mut vm::VM, path: &String) -> Result<(), InterpretResult> {
    match File::open(path) {
        Ok(mut file) => {
            let mut script = String::new();
            match file.read_to_string(&mut script) {
                Ok(_) => match vm.interpret(script) {
                    vm::InterpretResult::Ok => Ok(()),
                    _result => Err(InterpretResult::RuntimeError),
                },
                Err(_error) => Err(InterpretResult::CliError),
            }
        }
        Err(_error) => Err(InterpretResult::CliError),
    }
}
