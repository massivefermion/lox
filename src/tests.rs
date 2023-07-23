#[cfg(test)]
mod test {
    use crate::error::InterpretResult;
    use crate::vm::VM;

    #[test]
    fn hello_world() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(r#"println("Hello World!");"#.to_string()),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["Hello World!", "\n"]);
    }

    #[test]
    fn scope() {
        let mut vm = VM::new();
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
        assert_eq!(vm.stdout, vec!["global", "local", "\n"]);
    }

    #[test]
    fn nested_scope() {
        let mut vm = VM::new();
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
        assert_eq!(vm.stdout, vec!["global", "local1", "local2", "\n"]);
    }

    #[test]
    fn simple_function() {
        let mut vm = VM::new();
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
        assert_eq!(vm.stdout, vec!["5"]);
    }

    #[test]
    fn function_with_no_return() {
        let mut vm = VM::new();
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
        assert_eq!(vm.stdout, vec!["5", "\n", "nil"]);
    }

    #[test]
    fn inner_function() {
        let mut vm = VM::new();
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
        assert_eq!(vm.stdout, vec!["inside", "\n"]);
    }

    // #[test]
    // fn inner_function_only_seen_inside() {
    //     let mut vm = VM::new();
    //     assert_eq!(
    //         vm.interpret(
    //             r#"
    //                 fun outer() {
    //                     fun inner() {
    //                         println("inside");
    //                     }
    //                     inner();
    //                 }
    //                 inner();
    //             "#
    //             .to_string()
    //         ),
    //         InterpretResult::RuntimeError
    //     );
    // }

    #[test]
    fn local_variable() {
        let mut vm = VM::new();
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
        assert_eq!(vm.stdout, vec!["local", "\n"]);
    }

    #[test]
    fn first_class_function() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(
                r#"
                    fun make_closure() {
                        fun join(a, b) {
                            return a <> b;
                        }
                        return join;
                    }
                    let join = make_closure();
                    println(join("U-", 235));
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["U-235", "\n"]);
    }

    #[test]
    fn global_while() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(
                r#"
                    let a = 2;
                    let b = 5;
                    while a * b != -2 {
                        print(a, b);
                        a = a - 1;
                        b = b - 1;
                    }
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["2", "5", "1", "4", "0", "3",]);
    }

    // #[test]
    // fn local_while() {
    //     let mut vm = VM::new();
    //     assert_eq!(
    //         vm.interpret(
    //             r#"
    //                 {
    //                     let a = 2;
    //                     let b = 5;
    //                     while a * b != -2 {
    //                         print(a, b);
    //                         a = a - 1;
    //                         b = b - 1;
    //                     }
    //                 }
    //             "#
    //             .to_string()
    //         ),
    //         InterpretResult::Ok
    //     );
    //     assert_eq!(vm.stdout, vec!["2", "5", "1", "4", "0", "3",]);
    // }

    #[test]
    fn parse_test() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(
                r#"
                    print(parse("2" <> "5") + 5);
                    print(parse("2" <> ".5") + 1.5);
                    print(parse("2" <> ".5") + 2);
                    print(parse("false") and true);
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["30", "4", "4.5", "false"]);
    }

    #[test]
    fn closure_local() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(
                r#"
                    let x = "global";
                    let y = "global";
                    fun outer() {
                        let x = "local";
                        fun inner() {
                            println(x);
                            println(y);
                        }
                        inner();
                    }
                    outer();
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["local", "\n", "global", "\n"]);
    }

    #[test]
    fn closure_parameter() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(
                r#"
                    fun make_closure(a, b) {
                        fun join(c) {
                            return a <> b <> c;
                        }
                        return join;
                    }
                    let join = make_closure("U", "-");
                    println(join(235));
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["U-235", "\n"]);
    }

    // #[test]
    // fn return_in_while() {
    //     let mut vm = VM::new();
    //     assert_eq!(
    //         vm.interpret(
    //             r#"
    //                 fun make_closure() {
    //                     while true {
    //                         let i = "i";
    //                         fun show() print(i);
    //                         return show;
    //                     }
    //                 }
    //                 let closure = make_closure();
    //                 closure();
    //             "#
    //             .to_string()
    //         ),
    //         InterpretResult::Ok
    //     );
    //     assert_eq!(vm.stdout, vec!["i"]);
    // }

    // #[test]
    // fn closure_in_while() {
    //     let mut vm = VM::new();
    //     assert_eq!(
    //         vm.interpret(
    //             r#"
    //                 fun make_closures() {
    //                     let i = 5;
    //                     while i {
    //                         fun closure() { print(i); }
    //                         i = i - 1;
    //                         closure();
    //                     }
    //                 }
    //                 make_closures();
    //             "#
    //             .to_string()
    //         ),
    //         InterpretResult::Ok
    //     );
    //     assert_eq!(vm.stdout, vec!["5", "4", "3", "2", "1"]);
    // }
}
