"""Functional tests for the Lox interpreter CLI.

These tests exercise the compiled lox binary by writing Lox source to temp files
and running them through the interpreter, checking stdout and exit codes.
"""
import os
import subprocess
import tempfile
import pytest

WORKSPACE_DIR = os.environ.get("WORKSPACE_DIR", "/l2l/workspace")
LOX_DIR = os.path.join(WORKSPACE_DIR, "lox")
LOX_BINARY = os.path.join(LOX_DIR, "target", "debug", "lox")


def run_lox(source_code, timeout=30):
    """Write source to a temp file and run through the lox binary."""
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".lox", dir="/tmp", delete=False
    ) as f:
        f.write(source_code)
        f.flush()
        tmp_path = f.name
    try:
        result = subprocess.run(
            [LOX_BINARY, tmp_path],
            capture_output=True,
            text=True,
            timeout=timeout,
        )
        return result
    finally:
        os.unlink(tmp_path)


def run_lox_no_args(timeout=5):
    """Run the lox binary with no arguments (would start REPL, but we provide
    no input so it should fail or timeout)."""
    result = subprocess.run(
        [LOX_BINARY],
        capture_output=True,
        text=True,
        timeout=timeout,
        input="",
    )
    return result


def run_lox_too_many_args(timeout=5):
    """Run the lox binary with too many arguments."""
    result = subprocess.run(
        [LOX_BINARY, "/tmp/a.lox", "/tmp/b.lox"],
        capture_output=True,
        text=True,
        timeout=timeout,
    )
    return result


@pytest.fixture(autouse=True)
def check_binary_exists():
    """Ensure the lox binary is built before running tests."""
    assert os.path.isfile(LOX_BINARY), f"Binary not found at {LOX_BINARY}"


class TestBasicExecution:
    """Basic interpreter execution -- hello world, arithmetic, variables."""

    def test_hello_world(self):
        result = run_lox('println("hello world");')
        assert result.returncode == 0
        assert "hello world" in result.stdout

    def test_simple_arithmetic(self):
        result = run_lox("println(2 + 3);")
        assert result.returncode == 0
        assert "5" in result.stdout

    def test_variable_declaration_and_use(self):
        result = run_lox("let x = 42;\nprintln(x);")
        assert result.returncode == 0
        assert "42" in result.stdout

    def test_function_definition_and_call(self):
        source = """
fun add(a, b) {
    return a + b;
}
println(add(10, 20));
"""
        result = run_lox(source)
        assert result.returncode == 0
        assert "30" in result.stdout

    def test_string_concatenation(self):
        result = run_lox('println("hello" <> " " <> "world");')
        assert result.returncode == 0
        assert "hello world" in result.stdout

    def test_boolean_true(self):
        result = run_lox("println(true);")
        assert result.returncode == 0
        assert "true" in result.stdout

    def test_boolean_false(self):
        result = run_lox("println(false);")
        assert result.returncode == 0
        assert "false" in result.stdout

    def test_nil_value(self):
        result = run_lox("println(nil);")
        assert result.returncode == 0
        assert "nil" in result.stdout

    def test_multiple_statements(self):
        source = 'println("a");\nprintln("b");\nprintln("c");'
        result = run_lox(source)
        assert result.returncode == 0
        lines = result.stdout.strip().split("\n")
        assert len(lines) == 3
        assert lines[0] == "a"
        assert lines[1] == "b"
        assert lines[2] == "c"


class TestDivisionByZero:
    """Division by zero should return RuntimeError, not crash or produce NaN."""

    def test_divide_by_zero(self):
        result = run_lox("println(1 / 0);")
        assert result.returncode != 0

    def test_modulo_by_zero(self):
        result = run_lox("println(1 % 0);")
        assert result.returncode != 0

    def test_div_nif_by_zero(self):
        result = run_lox("println(div(1, 0));")
        assert result.returncode != 0

    def test_divide_normal(self):
        result = run_lox("println(10 / 2);")
        assert result.returncode == 0
        assert "5" in result.stdout

    def test_modulo_normal(self):
        result = run_lox("println(10 % 3);")
        assert result.returncode == 0
        assert "1" in result.stdout


class TestOperatorPrecedence:
    """Operator precedence: arithmetic before comparison/equality."""

    def test_addition_equality(self):
        result = run_lox("println(2 + 3 == 5);")
        assert result.returncode == 0
        assert "true" in result.stdout

    def test_multiplication_addition(self):
        result = run_lox("println(2 + 3 * 4);")
        assert result.returncode == 0
        assert "14" in result.stdout

    def test_comparison_with_arithmetic(self):
        result = run_lox("println(10 - 5 > 3);")
        assert result.returncode == 0
        assert "true" in result.stdout

    def test_complex_precedence(self):
        result = run_lox("println(2 * 3 + 4 * 5 == 26);")
        assert result.returncode == 0
        assert "true" in result.stdout

    def test_not_equal_with_arithmetic(self):
        result = run_lox("println(2 + 2 != 5);")
        assert result.returncode == 0
        assert "true" in result.stdout

    def test_grouped_expression(self):
        result = run_lox("println((2 + 3) * 4);")
        assert result.returncode == 0
        assert "20" in result.stdout


class TestSubtraction:
    """Subtraction should use proper OpCode::Subtract, not unary negation hack."""

    def test_simple_subtraction(self):
        result = run_lox("println(10 - 3);")
        assert result.returncode == 0
        assert "7" in result.stdout

    def test_subtraction_chained(self):
        result = run_lox("println(20 - 5 - 3);")
        assert result.returncode == 0
        assert "12" in result.stdout

    def test_subtraction_with_multiplication(self):
        result = run_lox("println(10 - 2 * 3);")
        assert result.returncode == 0
        assert "4" in result.stdout


class TestValueEquality:
    """Value equality uses type-dispatched comparison, not string-based."""

    def test_number_equality(self):
        result = run_lox("println(42 == 42);")
        assert result.returncode == 0
        assert "true" in result.stdout

    def test_number_inequality(self):
        result = run_lox("println(42 == 43);")
        assert result.returncode == 0
        assert "false" in result.stdout

    def test_string_equality(self):
        result = run_lox('println("hello" == "hello");')
        assert result.returncode == 0
        assert "true" in result.stdout

    def test_string_inequality(self):
        result = run_lox('println("hello" == "world");')
        assert result.returncode == 0
        assert "false" in result.stdout

    def test_bool_equality(self):
        result = run_lox("println(true == true);")
        assert result.returncode == 0
        assert "true" in result.stdout

    def test_nil_equality(self):
        result = run_lox("println(nil == nil);")
        assert result.returncode == 0
        assert "true" in result.stdout

    def test_mixed_type_inequality(self):
        result = run_lox('println(42 == "42");')
        assert result.returncode == 0
        assert "false" in result.stdout


class TestWhileLoops:
    """While loops work in local scopes, not just global."""

    def test_global_while_loop(self):
        source = """
let i = 0;
while (i < 5) {
    i = i + 1;
}
println(i);
"""
        result = run_lox(source)
        assert result.returncode == 0
        assert "5" in result.stdout

    def test_while_in_function(self):
        source = """
fun count_to(n) {
    let i = 0;
    while (i < n) {
        i = i + 1;
    }
    return i;
}
println(count_to(10));
"""
        result = run_lox(source)
        assert result.returncode == 0
        assert "10" in result.stdout

    def test_while_with_local_variables(self):
        source = """
fun sum_to(n) {
    let total = 0;
    let i = 1;
    while (i <= n) {
        total = total + i;
        i = i + 1;
    }
    return total;
}
println(sum_to(5));
"""
        result = run_lox(source)
        assert result.returncode == 0
        assert "15" in result.stdout


class TestNifReturnValues:
    """Print/println NIFs push Value::Nil for stack discipline."""

    def test_println_returns_nil(self):
        source = """
let x = println("hi");
println(x);
"""
        result = run_lox(source)
        assert result.returncode == 0
        lines = result.stdout.strip().split("\n")
        assert "hi" in lines[0]
        assert "nil" in lines[1]

    def test_print_returns_nil(self):
        source = """
let x = print("hi");
println(x);
"""
        result = run_lox(source)
        assert result.returncode == 0
        assert "nil" in result.stdout


class TestErrorRecovery:
    """Error recovery prevents cascading errors for multiple syntax issues."""

    def test_single_error(self):
        result = run_lox("let x = ;")
        assert result.returncode != 0

    def test_multiple_errors_no_cascade(self):
        source = """
let x = ;
let y = 42;
println(y);
"""
        result = run_lox(source)
        assert result.returncode != 0
        # The output should contain error messaging but not an overwhelming
        # cascade of unrelated errors.
        combined = result.stdout + result.stderr
        assert len(combined) < 5000, "Output too large, likely cascading errors"


class TestUnimplementedFeatures:
    """Unimplemented features produce clear error messages."""

    def test_for_loop_error(self):
        result = run_lox("for (let i = 0; i < 10; i = i + 1) { println(i); }")
        assert result.returncode != 0
        combined = (result.stdout + result.stderr).lower()
        assert "not" in combined or "implement" in combined or "error" in combined

    def test_class_error(self):
        result = run_lox("class Foo {}")
        assert result.returncode != 0
        combined = (result.stdout + result.stderr).lower()
        assert "not" in combined or "implement" in combined or "error" in combined


class TestInvalidArgs:
    """CLI argument handling."""

    def test_too_many_args(self):
        result = run_lox_too_many_args()
        assert result.returncode != 0

    def test_nonexistent_file(self):
        result = subprocess.run(
            [LOX_BINARY, "/tmp/nonexistent_file_xyz123.lox"],
            capture_output=True,
            text=True,
            timeout=10,
        )
        assert result.returncode != 0


class TestEdgeCases:
    """Edge cases and boundary conditions."""

    def test_empty_program(self):
        result = run_lox("")
        # Empty program may or may not succeed depending on parser
        # Just verify it does not crash (returns 0 or 1, not a signal)
        assert result.returncode in (0, 1)

    def test_only_comment(self):
        # If the language supports comments
        result = run_lox("// this is a comment")
        # May or may not succeed depending on parser handling
        # Just verify it doesn't crash
        assert result.returncode in (0, 1)

    def test_nested_arithmetic(self):
        result = run_lox("println(((1 + 2) * (3 + 4)) - 1);")
        assert result.returncode == 0
        assert "20" in result.stdout

    def test_negative_numbers(self):
        result = run_lox("println(-5);")
        assert result.returncode == 0
        assert "-5" in result.stdout

    def test_large_number(self):
        result = run_lox("println(999999999);")
        assert result.returncode == 0
        assert "999999999" in result.stdout

    def test_multi_function_calls(self):
        source = """
fun double(x) {
    return x * 2;
}
fun triple(x) {
    return x * 3;
}
println(double(triple(5)));
"""
        result = run_lox(source)
        assert result.returncode == 0
        assert "30" in result.stdout

    def test_if_else(self):
        source = """
if (true) {
    println("yes");
} else {
    println("no");
}
"""
        result = run_lox(source)
        assert result.returncode == 0
        assert "yes" in result.stdout


class TestClosures:
    """Closure capture and scoping behavior."""

    def test_simple_closure(self):
        source = """
fun make_adder(x) {
    fun adder(y) {
        return x + y;
    }
    return adder;
}
let add5 = make_adder(5);
println(add5(3));
"""
        result = run_lox(source)
        assert result.returncode == 0
        assert "8" in result.stdout

    def test_closure_captures_parameter(self):
        source = """
fun greet(name) {
    fun say_hi() {
        return "hello " <> name;
    }
    return say_hi;
}
let g = greet("world");
println(g());
"""
        result = run_lox(source)
        assert result.returncode == 0
        assert "hello world" in result.stdout

    def test_closure_in_local_scope(self):
        source = """
fun outer() {
    let x = 10;
    fun inner() {
        return x;
    }
    return inner;
}
let f = outer();
println(f());
"""
        result = run_lox(source)
        assert result.returncode == 0
        assert "10" in result.stdout


class TestLogicalOperators:
    """Logical and/or/not operators with short-circuit evaluation."""

    def test_and_true_true(self):
        result = run_lox("println(true and true);")
        assert result.returncode == 0
        assert "true" in result.stdout

    def test_and_true_false(self):
        result = run_lox("println(true and false);")
        assert result.returncode == 0
        assert "false" in result.stdout

    def test_and_false_true(self):
        result = run_lox("println(false and true);")
        assert result.returncode == 0
        assert "false" in result.stdout

    def test_or_false_false(self):
        result = run_lox("println(false or false);")
        assert result.returncode == 0
        assert "false" in result.stdout

    def test_or_true_false(self):
        result = run_lox("println(true or false);")
        assert result.returncode == 0
        assert "true" in result.stdout

    def test_or_false_true(self):
        result = run_lox("println(false or true);")
        assert result.returncode == 0
        assert "true" in result.stdout

    def test_not_true(self):
        result = run_lox("println(not true);")
        assert result.returncode == 0
        assert "false" in result.stdout

    def test_not_false(self):
        result = run_lox("println(not false);")
        assert result.returncode == 0
        assert "true" in result.stdout

    def test_not_nil(self):
        result = run_lox("println(not nil);")
        assert result.returncode == 0
        assert "true" in result.stdout

    def test_not_zero(self):
        result = run_lox("println(not 0);")
        assert result.returncode == 0
        assert "true" in result.stdout

    def test_not_empty_string(self):
        result = run_lox('println(not "");')
        assert result.returncode == 0
        assert "true" in result.stdout

    def test_not_number(self):
        result = run_lox("println(not 42);")
        assert result.returncode == 0
        assert "false" in result.stdout

    def test_not_string(self):
        result = run_lox('println(not "hello");')
        assert result.returncode == 0
        assert "false" in result.stdout

    def test_double_not(self):
        result = run_lox("println(not not true);")
        assert result.returncode == 0
        assert "true" in result.stdout


class TestComparisons:
    """Comparison operators: >, >=, <, <=."""

    def test_gt_true(self):
        result = run_lox("println(5 > 3);")
        assert result.returncode == 0
        assert "true" in result.stdout

    def test_gt_false(self):
        result = run_lox("println(3 > 5);")
        assert result.returncode == 0
        assert "false" in result.stdout

    def test_gte_true(self):
        result = run_lox("println(5 >= 5);")
        assert result.returncode == 0
        assert "true" in result.stdout

    def test_gte_false(self):
        result = run_lox("println(4 >= 5);")
        assert result.returncode == 0
        assert "false" in result.stdout

    def test_lt_true(self):
        result = run_lox("println(3 < 5);")
        assert result.returncode == 0
        assert "true" in result.stdout

    def test_lt_false(self):
        result = run_lox("println(5 < 3);")
        assert result.returncode == 0
        assert "false" in result.stdout

    def test_lte_true(self):
        result = run_lox("println(5 <= 5);")
        assert result.returncode == 0
        assert "true" in result.stdout

    def test_lte_false(self):
        result = run_lox("println(6 <= 5);")
        assert result.returncode == 0
        assert "false" in result.stdout


class TestFunctionReturnValues:
    """Function return behavior including early returns and recursion."""

    def test_return_value(self):
        source = """
fun square(x) {
    return x * x;
}
println(square(7));
"""
        result = run_lox(source)
        assert result.returncode == 0
        assert "49" in result.stdout

    def test_early_return(self):
        source = """
fun check(x) {
    if (x > 0) {
        return "positive";
    }
    return "non-positive";
}
println(check(5));
"""
        result = run_lox(source)
        assert result.returncode == 0
        assert "positive" in result.stdout

    def test_recursive_factorial(self):
        source = """
fun factorial(n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}
println(factorial(5));
"""
        result = run_lox(source)
        assert result.returncode == 0
        assert "120" in result.stdout
