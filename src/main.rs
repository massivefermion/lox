use std::env;
use std::fs::File;
use std::io::Read;

mod chunk;
mod compiler;
mod error;
mod function;
mod nif;
mod op;
mod scanner;
mod tests;
mod token;
mod value;
mod vm;

use error::InterpretResult;

use rustyline::DefaultEditor;

fn main() -> Result<InterpretResult, InterpretResult> {
    let args: Vec<String> = env::args().collect();

    let mut vm = vm::VM::new();
    match &args[..] {
        [_] => repl(&mut vm),
        [_, path] => run_file(&mut vm, path),
        _ => Err(InterpretResult::CliError),
        // _ => error::error_out(error::LoxError::new(
        //     "Usage: lox [script]",
        //     error::ErrorContext::Cli,
        //     None,
        // )),
    }
}

fn repl(vm: &mut vm::VM) -> Result<InterpretResult, InterpretResult> {
    match DefaultEditor::new() {
        Ok(mut rl) => {
            loop {
                let line = rl.readline("lox -> ");
                match line {
                    Ok(mut line) => {
                        let mut result = Ok(());
                        if line.ends_with('{') {
                            result = loop {
                                let new_line = rl.readline("......  ");
                                match new_line {
                                    Ok(new_line) => {
                                        line += &new_line;
                                        if line.ends_with('}') {
                                            break Ok(());
                                        }
                                    }
                                    _ => break Err(()),
                                }
                            };
                        }
                        match result {
                            Ok(()) => {
                                vm.interpret(line);
                            }

                            // let line_function = Function::new_main("##MAIN##".to_string());
                            // let mut compiler = Compiler::new(vm, line_function, &line);
                            // match compiler.compile() {
                            //     Ok(main_function) => match vm.run(main_function, 0) {
                            //         // InterpretResult::Ok => break Ok(InterpretResult::Ok),
                            //         InterpretResult::Ok => continue,
                            //         // _ => break Err(InterpretResult::RuntimeError),
                            //         _ => continue,
                            //     },
                            //     // _ => break Err(InterpretResult::CompileError),
                            //     _ => continue,
                            // }
                            _ => break Err(InterpretResult::CliError),
                        }
                    }
                    _ => break Err(InterpretResult::CliError),
                }
            }
        }
        Err(_) => Err(InterpretResult::CliError),
    }
}

// fn repl(vm: &mut vm::VM) -> Result<(), InterpretResult> {
//     for line in stdin().lock().lines() {
//         print!("lox -> ");
//         match line {
//             Ok(line) => match vm.interpret(line) {
//                 vm::InterpretResult::Ok => (),
//                 result => eprintln!("{:?}", result),
//             },
//             Err(_) => break,
//         }
//     }
//     Ok(())
// }

fn run_file(vm: &mut vm::VM, path: &String) -> Result<InterpretResult, InterpretResult> {
    match File::open(path) {
        Ok(mut file) => {
            let mut script = String::new();
            match file.read_to_string(&mut script) {
                Ok(_) => match vm.interpret(script) {
                    InterpretResult::Ok => Ok(InterpretResult::Ok),
                    _result => Err(InterpretResult::RuntimeError),
                },
                Err(_error) => Err(InterpretResult::CliError),
            }
        }
        Err(_error) => Err(InterpretResult::CliError),
    }
}
