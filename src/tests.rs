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
    fn recursive_function() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(
                r#"
                    fun sum_till_one(a) {
                        if a == 1 {
                            return 1;
                        }

                        return a + sum_till_one(a - 1);
                    }
                    print(sum_till_one(4));
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["10"]);
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

    #[test]
    fn inner_function_only_seen_inside() {
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
                    inner();
                "#
                .to_string()
            ),
            InterpretResult::RuntimeError
        );
    }

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

    #[test]
    fn local_while() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(
                r#"
                    {
                        let a = 2;
                        let b = 5;
                        while a * b != -2 {
                            print(a, b);
                            a = a - 1;
                            b = b - 1;
                        }
                    }
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["2", "5", "1", "4", "0", "3",]);
    }

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

    #[test]
    fn return_in_while() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(
                r#"
                    fun make_closure() {
                        while true {
                            let i = "i";
                            fun show() print(i);
                            return show;
                        }
                    }
                    let closure = make_closure();
                    closure();
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["i"]);
    }

    #[test]
    fn div_by_zero_nif() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(r#"print(div(1, 0));"#.to_string()),
            InterpretResult::RuntimeError
        );
    }

    #[test]
    fn divide_by_zero() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(r#"print(1 / 0);"#.to_string()),
            InterpretResult::RuntimeError
        );
    }

    #[test]
    fn rem_by_zero() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(r#"print(1 % 0);"#.to_string()),
            InterpretResult::RuntimeError
        );
    }

    #[test]
    fn operator_precedence() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(r#"print(2 + 3 == 5);"#.to_string()),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["true"]);
    }

    #[test]
    fn subtraction() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(r#"print(10 - 3);"#.to_string()),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["7"]);
    }

    #[test]
    fn mixed_precedence() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(r#"print(2 + 3 * 4);"#.to_string()),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["14"]);
    }

    #[test]
    fn error_recovery() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(
                r#"
                    let x = ;
                    let y = 5;
                    print(y);
                "#
                .to_string()
            ),
            InterpretResult::CompileError
        );
    }

    #[test]
    fn unimplemented_for() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(r#"for (let i = 0; i < 10; i = i + 1) { print(i); }"#.to_string()),
            InterpretResult::CompileError
        );
    }

    #[test]
    fn unimplemented_class() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(r#"class Foo {}"#.to_string()),
            InterpretResult::CompileError
        );
    }

    #[test]
    fn unimplemented_this() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(r#"this;"#.to_string()),
            InterpretResult::CompileError
        );
    }

    #[test]
    fn unimplemented_super() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(r#"super;"#.to_string()),
            InterpretResult::CompileError
        );
    }

    #[test]
    fn unimplemented_expands() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(r#"expands;"#.to_string()),
            InterpretResult::CompileError
        );
    }

    #[test]
    fn closure_in_while() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(
                r#"
                    fun make_closures() {
                        let i = 5;
                        while i {
                            fun closure() { print(i); }
                            i = i - 1;
                            closure();
                        }
                    }
                    make_closures();
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["5", "4", "3", "2", "1"]);
    }

    #[test]
    fn arithmetic_ops() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(
                r#"print(2 + 3); print(7 - 4); print(3 * 4); print(9 / 4); print(10 % 3);"#
                    .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["5", "3", "12", "2.25", "1"]);
    }

    #[test]
    fn comparison_ops() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(
                r#"
                    print(3 == 3);
                    print(3 != 4);
                    print(5 > 3);
                    print(5 >= 5);
                    print(3 < 5);
                    print(3 <= 3);
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(
            vm.stdout,
            vec!["true", "true", "true", "true", "true", "true"]
        );
    }

    #[test]
    fn variable_access_ops() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(
                r#"
                    let g = "init";
                    g = "updated";
                    print(g);
                    { let l = "local"; l = "changed"; print(l); }
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["updated", "changed"]);
    }

    #[test]
    fn control_flow_ops() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(
                r#"
                    if true { print("yes"); } else { print("no"); }
                    if false { print("no"); } else { print("yes"); }
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["yes", "yes"]);
    }

    #[test]
    fn closures_ops() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(
                r#"
                    fun make_adder(n) {
                        fun add(x) { return n + x; }
                        return add;
                    }
                    let add5 = make_adder(5);
                    print(add5(3));
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["8"]);
    }

    #[test]
    fn calls_ops() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(
                r#"
                    fun double(x) { return x * 2; }
                    print(double(7));
                    print(type_of(42));
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["14", "number"]);
    }

    #[test]
    fn scanner_string_with_newline() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret("print(\"line1\nline2\");".to_string()),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["line1\nline2"]);
    }

    #[test]
    fn scanner_number_with_decimal() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(r#"print(123.5 + 0.5);"#.to_string()),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["124"]);
    }

    #[test]
    fn scanner_multi_char_operators() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(
                r#"
                    print(1 == 1);
                    print(1 != 2);
                    print(2 <= 3);
                    print(3 >= 2);
                    print("a" <> "b");
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["true", "true", "true", "true", "ab"]);
    }

    #[test]
    fn scanner_keyword_vs_identifier() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(
                r#"
                    let truth = true;
                    let falsy = false;
                    let nothing = nil;
                    print(truth);
                    print(falsy);
                    print(nothing);
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["true", "false", "nil"]);
    }

    #[test]
    fn compiler_empty_params() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(
                r#"
                    fun no_args() { return 42; }
                    print(no_args());
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["42"]);
    }

    #[test]
    fn compiler_single_param() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(
                r#"
                    fun square(n) { return n * n; }
                    print(square(7));
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["49"]);
    }

    #[test]
    fn compiler_max_nesting_depth() {
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(
                r#"
                    let a = 1;
                    {
                        let b = 2;
                        {
                            let c = 3;
                            {
                                let d = 4;
                                print(a + b + c + d);
                            }
                            print(a + b + c);
                        }
                        print(a + b);
                    }
                    print(a);
                "#
                .to_string()
            ),
            InterpretResult::Ok
        );
        assert_eq!(vm.stdout, vec!["10", "6", "3", "1"]);
    }
}
