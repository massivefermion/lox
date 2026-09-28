"""Functional parity tests for the Lox interpreter CLI.

These tests run Lox programs through the interpreter binary and capture
exact outputs (stdout, stderr, exit code) for origin/target comparison.
The binary path is controlled by the LOX_BINARY env var or defaults to
the target binary at $WORKSPACE_DIR/lox/target/debug/lox.
"""
import os
import subprocess
import tempfile
import pytest

WORKSPACE_DIR = os.environ.get("WORKSPACE_DIR", "/l2l/workspace")
LOX_BINARY = os.environ.get(
    "LOX_BINARY", os.path.join(WORKSPACE_DIR, "lox", "target", "debug", "lox")
)


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


def run_lox_raw(*args, input_text=None, timeout=10):
    """Run the lox binary with arbitrary arguments."""
    result = subprocess.run(
        [LOX_BINARY, *args],
        capture_output=True,
        text=True,
        timeout=timeout,
        input=input_text,
    )
    return result


@pytest.fixture(autouse=True)
def check_binary_exists():
    """Ensure the lox binary exists before running tests."""
    assert os.path.isfile(LOX_BINARY), f"Binary not found at {LOX_BINARY}"


# ---------------------------------------------------------------------------
# HAPPY_PATH: Basic execution
# ---------------------------------------------------------------------------
class TestBasicExecution:
    """Basic interpreter execution: hello world, arithmetic, variables, functions."""

    def test_hello_world(self):
        result = run_lox('println("hello world");')
        assert result.returncode == 0
        assert result.stdout.strip() == "hello world"

    def test_integer_arithmetic(self):
        result = run_lox("println(2 + 3);")
        assert result.returncode == 0
        assert result.stdout.strip() == "5"

    def test_float_arithmetic(self):
        result = run_lox("println(1.5 + 2.5);")
        assert result.returncode == 0
        assert result.stdout.strip() == "4"

    def test_variable_declaration(self):
        result = run_lox("let x = 42;\nprintln(x);")
        assert result.returncode == 0
        assert result.stdout.strip() == "42"

    def test_variable_reassignment(self):
        source = "let x = 1;\nx = 2;\nprintln(x);"
        result = run_lox(source)
        assert result.returncode == 0
        assert result.stdout.strip() == "2"

    def test_function_definition_and_call(self):
        source = """
fun add(a, b) {
    return a + b;
}
println(add(10, 20));
"""
        result = run_lox(source)
        assert result.returncode == 0
        assert result.stdout.strip() == "30"

    def test_string_concatenation_operator(self):
        result = run_lox('println("hello" <> " " <> "world");')
        assert result.returncode == 0
        assert result.stdout.strip() == "hello world"

    def test_boolean_true(self):
        result = run_lox("println(true);")
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_boolean_false(self):
        result = run_lox("println(false);")
        assert result.returncode == 0
        assert result.stdout.strip() == "false"

    def test_nil_value(self):
        result = run_lox("println(nil);")
        assert result.returncode == 0
        assert result.stdout.strip() == "nil"

    def test_multiple_statements(self):
        source = 'println("a");\nprintln("b");\nprintln("c");'
        result = run_lox(source)
        assert result.returncode == 0
        lines = result.stdout.strip().split("\n")
        assert lines == ["a", "b", "c"]

    def test_multiplication(self):
        result = run_lox("println(6 * 7);")
        assert result.returncode == 0
        assert result.stdout.strip() == "42"

    def test_nested_function_calls(self):
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
        assert result.stdout.strip() == "30"


# ---------------------------------------------------------------------------
# HAPPY_PATH: Division and modulo
# ---------------------------------------------------------------------------
class TestDivisionAndModulo:
    """Division and modulo operations including error cases."""

    def test_integer_division(self):
        result = run_lox("println(10 / 2);")
        assert result.returncode == 0
        assert result.stdout.strip() == "5"

    def test_modulo(self):
        result = run_lox("println(10 % 3);")
        assert result.returncode == 0
        assert result.stdout.strip() == "1"

    def test_divide_by_zero_error(self):
        result = run_lox("println(1 / 0);")
        assert result.returncode != 0

    def test_modulo_by_zero_error(self):
        result = run_lox("println(1 % 0);")
        assert result.returncode != 0

    def test_div_nif_by_zero_error(self):
        result = run_lox("println(div(1, 0));")
        assert result.returncode != 0


# ---------------------------------------------------------------------------
# HAPPY_PATH: Operator precedence
# ---------------------------------------------------------------------------
class TestOperatorPrecedence:
    """Operator precedence: arithmetic before comparison/equality."""

    def test_add_mul_precedence(self):
        result = run_lox("println(2 + 3 * 4);")
        assert result.returncode == 0
        assert result.stdout.strip() == "14"

    def test_grouping_override(self):
        result = run_lox("println((2 + 3) * 4);")
        assert result.returncode == 0
        assert result.stdout.strip() == "20"

    def test_complex_precedence(self):
        result = run_lox("println(2 * 3 + 4 * 5 == 26);")
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_comparison_with_arithmetic(self):
        result = run_lox("println(10 - 5 > 3);")
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_addition_equality(self):
        result = run_lox("println(2 + 3 == 5);")
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_not_equal_with_arithmetic(self):
        result = run_lox("println(2 + 2 != 5);")
        assert result.returncode == 0
        assert result.stdout.strip() == "true"


# ---------------------------------------------------------------------------
# HAPPY_PATH: Subtraction
# ---------------------------------------------------------------------------
class TestSubtraction:
    """Subtraction operator (not unary negation hack)."""

    def test_simple(self):
        result = run_lox("println(10 - 3);")
        assert result.returncode == 0
        assert result.stdout.strip() == "7"

    def test_chained(self):
        result = run_lox("println(20 - 5 - 3);")
        assert result.returncode == 0
        assert result.stdout.strip() == "12"

    def test_with_multiplication(self):
        result = run_lox("println(10 - 2 * 3);")
        assert result.returncode == 0
        assert result.stdout.strip() == "4"


# ---------------------------------------------------------------------------
# HAPPY_PATH: Value equality
# ---------------------------------------------------------------------------
class TestValueEquality:
    """Value equality uses type-dispatched comparison."""

    def test_number_equal(self):
        result = run_lox("println(42 == 42);")
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_number_not_equal(self):
        result = run_lox("println(42 == 43);")
        assert result.returncode == 0
        assert result.stdout.strip() == "false"

    def test_string_equal(self):
        result = run_lox('println("hello" == "hello");')
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_string_not_equal(self):
        result = run_lox('println("hello" == "world");')
        assert result.returncode == 0
        assert result.stdout.strip() == "false"

    def test_bool_equal(self):
        result = run_lox("println(true == true);")
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_nil_equal(self):
        result = run_lox("println(nil == nil);")
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_mixed_type_not_equal(self):
        result = run_lox('println(42 == "42");')
        assert result.returncode == 0
        assert result.stdout.strip() == "false"


# ---------------------------------------------------------------------------
# HAPPY_PATH: While loops
# ---------------------------------------------------------------------------
class TestWhileLoops:
    """While loops in global and local scopes."""

    def test_global_loop(self):
        source = """
let i = 0;
while (i < 5) {
    i = i + 1;
}
println(i);
"""
        result = run_lox(source)
        assert result.returncode == 0
        assert result.stdout.strip() == "5"

    def test_loop_in_function(self):
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
        assert result.stdout.strip() == "10"

    def test_loop_with_accumulator(self):
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
        assert result.stdout.strip() == "15"


# ---------------------------------------------------------------------------
# HAPPY_PATH: NIF return values
# ---------------------------------------------------------------------------
class TestNifReturnValues:
    """Print/println NIFs push Value::Nil for stack discipline."""

    def test_println_returns_nil(self):
        source = 'let x = println("hi");\nprintln(x);'
        result = run_lox(source)
        assert result.returncode == 0
        lines = result.stdout.strip().split("\n")
        assert lines[0] == "hi"
        assert lines[1] == "nil"

    def test_print_returns_nil(self):
        source = 'let x = print("hi");\nprintln(x);'
        result = run_lox(source)
        assert result.returncode == 0
        assert "nil" in result.stdout


# ---------------------------------------------------------------------------
# HAPPY_PATH: Closures
# ---------------------------------------------------------------------------
class TestClosures:
    """Closure capture and scoping behavior."""

    def test_make_adder(self):
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
        assert result.stdout.strip() == "8"

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
        assert result.stdout.strip() == "hello world"

    def test_closure_captures_local(self):
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
        assert result.stdout.strip() == "10"


# ---------------------------------------------------------------------------
# HAPPY_PATH: Logical operators
# ---------------------------------------------------------------------------
class TestLogicalOperators:
    """Logical and/or/not operators."""

    def test_and_true_true(self):
        result = run_lox("println(true and true);")
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_and_true_false(self):
        result = run_lox("println(true and false);")
        assert result.returncode == 0
        assert result.stdout.strip() == "false"

    def test_and_false_short_circuit(self):
        result = run_lox("println(false and true);")
        assert result.returncode == 0
        assert result.stdout.strip() == "false"

    def test_or_false_false(self):
        result = run_lox("println(false or false);")
        assert result.returncode == 0
        assert result.stdout.strip() == "false"

    def test_or_true_false(self):
        result = run_lox("println(true or false);")
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_or_false_true(self):
        result = run_lox("println(false or true);")
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_not_true(self):
        result = run_lox("println(not true);")
        assert result.returncode == 0
        assert result.stdout.strip() == "false"

    def test_not_false(self):
        result = run_lox("println(not false);")
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_not_nil(self):
        result = run_lox("println(not nil);")
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_not_zero(self):
        result = run_lox("println(not 0);")
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_not_empty_string(self):
        result = run_lox('println(not "");')
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_not_number(self):
        result = run_lox("println(not 42);")
        assert result.returncode == 0
        assert result.stdout.strip() == "false"

    def test_not_string(self):
        result = run_lox('println(not "hello");')
        assert result.returncode == 0
        assert result.stdout.strip() == "false"

    def test_double_not(self):
        result = run_lox("println(not not true);")
        assert result.returncode == 0
        assert result.stdout.strip() == "true"


# ---------------------------------------------------------------------------
# HAPPY_PATH: Comparisons
# ---------------------------------------------------------------------------
class TestComparisons:
    """Comparison operators: >, >=, <, <=."""

    def test_gt_true(self):
        result = run_lox("println(5 > 3);")
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_gt_false(self):
        result = run_lox("println(3 > 5);")
        assert result.returncode == 0
        assert result.stdout.strip() == "false"

    def test_gte_equal(self):
        result = run_lox("println(5 >= 5);")
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_gte_less(self):
        result = run_lox("println(4 >= 5);")
        assert result.returncode == 0
        assert result.stdout.strip() == "false"

    def test_lt_true(self):
        result = run_lox("println(3 < 5);")
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_lt_false(self):
        result = run_lox("println(5 < 3);")
        assert result.returncode == 0
        assert result.stdout.strip() == "false"

    def test_lte_equal(self):
        result = run_lox("println(5 <= 5);")
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_lte_greater(self):
        result = run_lox("println(6 <= 5);")
        assert result.returncode == 0
        assert result.stdout.strip() == "false"


# ---------------------------------------------------------------------------
# HAPPY_PATH: Function return values
# ---------------------------------------------------------------------------
class TestFunctionReturns:
    """Function return behavior: values, early return, recursion."""

    def test_return_value(self):
        source = """
fun square(x) {
    return x * x;
}
println(square(7));
"""
        result = run_lox(source)
        assert result.returncode == 0
        assert result.stdout.strip() == "49"

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
        assert result.stdout.strip() == "positive"

    def test_early_return_else_branch(self):
        source = """
fun check(x) {
    if (x > 0) {
        return "positive";
    }
    return "non-positive";
}
println(check(-1));
"""
        result = run_lox(source)
        assert result.returncode == 0
        assert result.stdout.strip() == "non-positive"

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
        assert result.stdout.strip() == "120"

    def test_fibonacci(self):
        source = """
fun fib(n) {
    if (n <= 1) {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}
println(fib(10));
"""
        result = run_lox(source)
        assert result.returncode == 0
        assert result.stdout.strip() == "55"


# ---------------------------------------------------------------------------
# HAPPY_PATH: If/else
# ---------------------------------------------------------------------------
class TestIfElse:
    """If/else conditional execution."""

    def test_if_true_branch(self):
        source = """
if (true) {
    println("yes");
} else {
    println("no");
}
"""
        result = run_lox(source)
        assert result.returncode == 0
        assert result.stdout.strip() == "yes"

    def test_if_false_branch(self):
        source = """
if (false) {
    println("yes");
} else {
    println("no");
}
"""
        result = run_lox(source)
        assert result.returncode == 0
        assert result.stdout.strip() == "no"

    def test_if_without_else(self):
        source = """
if (true) {
    println("only if");
}
println("done");
"""
        result = run_lox(source)
        assert result.returncode == 0
        lines = result.stdout.strip().split("\n")
        assert lines == ["only if", "done"]

    def test_nested_if(self):
        source = """
let x = 10;
if (x > 5) {
    if (x > 8) {
        println("big");
    } else {
        println("medium");
    }
} else {
    println("small");
}
"""
        result = run_lox(source)
        assert result.returncode == 0
        assert result.stdout.strip() == "big"


# ---------------------------------------------------------------------------
# HAPPY_PATH: Scoping
# ---------------------------------------------------------------------------
class TestScoping:
    """Block scoping and variable shadowing."""

    def test_block_scope(self):
        source = """
let x = 1;
{
    let x = 2;
    println(x);
}
println(x);
"""
        result = run_lox(source)
        assert result.returncode == 0
        lines = result.stdout.strip().split("\n")
        assert lines == ["2", "1"]

    def test_nested_scope_access(self):
        source = """
let x = 10;
{
    let y = 20;
    println(x + y);
}
"""
        result = run_lox(source)
        assert result.returncode == 0
        assert result.stdout.strip() == "30"


# ---------------------------------------------------------------------------
# HAPPY_PATH: Edge cases and boundary
# ---------------------------------------------------------------------------
class TestEdgeCases:
    """Edge cases and boundary conditions."""

    def test_empty_program(self):
        result = run_lox("")
        assert result.returncode in (0, 1)

    def test_comment_only(self):
        result = run_lox("// this is a comment")
        assert result.returncode in (0, 1)

    def test_nested_arithmetic(self):
        result = run_lox("println(((1 + 2) * (3 + 4)) - 1);")
        assert result.returncode == 0
        assert result.stdout.strip() == "20"

    def test_unary_negation(self):
        result = run_lox("println(-5);")
        assert result.returncode == 0
        assert result.stdout.strip() == "-5"

    def test_large_number(self):
        result = run_lox("println(999999999);")
        assert result.returncode == 0
        assert result.stdout.strip() == "999999999"

    def test_zero(self):
        result = run_lox("println(0);")
        assert result.returncode == 0
        assert result.stdout.strip() == "0"

    def test_negative_result(self):
        result = run_lox("println(3 - 10);")
        assert result.returncode == 0
        assert result.stdout.strip() == "-7"


# ---------------------------------------------------------------------------
# INVALID_ARGS: CLI argument handling
# ---------------------------------------------------------------------------
class TestInvalidArgs:
    """CLI argument validation."""

    def test_too_many_args(self):
        result = run_lox_raw("/tmp/a.lox", "/tmp/b.lox")
        assert result.returncode != 0

    def test_nonexistent_file(self):
        result = run_lox_raw("/tmp/nonexistent_file_xyz123.lox")
        assert result.returncode != 0


# ---------------------------------------------------------------------------
# INVALID_INPUT: Error handling
# ---------------------------------------------------------------------------
class TestErrorHandling:
    """Syntax errors and unimplemented features."""

    def test_syntax_error(self):
        result = run_lox("let x = ;")
        assert result.returncode != 0

    def test_for_loop_unimplemented(self):
        result = run_lox("for (let i = 0; i < 10; i = i + 1) { println(i); }")
        assert result.returncode != 0
        combined = (result.stdout + result.stderr).lower()
        assert "not" in combined or "implement" in combined or "error" in combined

    def test_class_unimplemented(self):
        result = run_lox("class Foo {}")
        assert result.returncode != 0
        combined = (result.stdout + result.stderr).lower()
        assert "not" in combined or "implement" in combined or "error" in combined

    def test_multiple_errors_no_cascade(self):
        source = "let x = ;\nlet y = 42;\nprintln(y);"
        result = run_lox(source)
        assert result.returncode != 0
        combined = result.stdout + result.stderr
        assert len(combined) < 5000


# ---------------------------------------------------------------------------
# HAPPY_PATH: NIF built-in functions
# ---------------------------------------------------------------------------
class TestBuiltinNifs:
    """Built-in native functions (NIFs)."""

    def test_type_of_number(self):
        result = run_lox("println(type_of(42));")
        assert result.returncode == 0
        assert result.stdout.strip() == "number"

    def test_type_of_string(self):
        result = run_lox('println(type_of("hello"));')
        assert result.returncode == 0
        assert result.stdout.strip() == "string"

    def test_type_of_bool(self):
        result = run_lox("println(type_of(true));")
        assert result.returncode == 0
        assert result.stdout.strip() == "boolean"

    def test_type_of_nil(self):
        result = run_lox("println(type_of(nil));")
        assert result.returncode == 0
        assert result.stdout.strip() == "nil"

    def test_is_nil_true(self):
        result = run_lox("println(is_nil(nil));")
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_is_nil_false(self):
        result = run_lox("println(is_nil(42));")
        assert result.returncode == 0
        assert result.stdout.strip() == "false"

    def test_is_number_true(self):
        result = run_lox("println(is_number(42));")
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_is_number_false(self):
        result = run_lox('println(is_number("hello"));')
        assert result.returncode == 0
        assert result.stdout.strip() == "false"

    def test_is_string_true(self):
        result = run_lox('println(is_string("hi"));')
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_is_string_false(self):
        result = run_lox("println(is_string(42));")
        assert result.returncode == 0
        assert result.stdout.strip() == "false"

    def test_is_boolean_true(self):
        result = run_lox("println(is_boolean(true));")
        assert result.returncode == 0
        assert result.stdout.strip() == "true"

    def test_is_boolean_false(self):
        result = run_lox("println(is_boolean(42));")
        assert result.returncode == 0
        assert result.stdout.strip() == "false"

    def test_parse_number_string(self):
        result = run_lox('println(parse("42"));')
        assert result.returncode == 0
        assert result.stdout.strip() == "42"

    def test_div_integer(self):
        result = run_lox("println(div(10, 3));")
        assert result.returncode == 0
        assert result.stdout.strip() == "3"
