use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn is_nasm_installed() -> bool {
    Command::new("nasm")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn compile_and_run(source: &str, expected_exit_code: i32) -> bool {
    if !is_nasm_installed() {
        eprintln!("NASM not installed, skipping test");
        return false;
    }

    let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let asm_file = format!("test_output_{}.asm", test_id);
    let obj_file = format!("test_output_{}.o", test_id);
    let exe_file = if cfg!(windows) {
        format!("test_program_{}.exe", test_id)
    } else {
        format!("test_program_{}", test_id)
    };

    let (parse_output, ir_program) = minic::compiler::compile_with_ir(source, vec![]);
    if !parse_output.is_valid() {
        eprintln!("Parse errors: {:?}", parse_output.errors);
        return false;
    }

    let ir = ir_program.expect("IR не сгенерирован");
    let result = minic::codegen::generate_assembly(&ir, false);

    println!("\n=== GENERATED ASM ===\n{}", result.assembly);
    println!("=== END ASM ===\n");

    fs::write(&asm_file, &result.assembly).expect("Failed to write ASM");

    let nasm_status = Command::new("nasm")
    .args(&["-f", "elf64", &asm_file, "-o", &obj_file])
    .output();

    let nasm_output = match nasm_status {
        Ok(out) => out,
        Err(e) => {
            eprintln!("NASM error: {}", e);
            return false;
        }
    };

    if !nasm_output.status.success() {
        eprintln!(
            "NASM stderr:\n{}",
            String::from_utf8_lossy(&nasm_output.stderr)
        );
        return false;
    }

    let linker = if cfg!(windows) { "gcc" } else { "ld" };

    let ld_status = if cfg!(windows) {
        Command::new(linker)
        .args(&["-no-pie", "-o", &exe_file, &obj_file])
        .output()
    } else {
        Command::new(linker)
        .args(&[&obj_file, "-o", &exe_file])
        .output()
    };

    let ld_output = match ld_status {
        Ok(out) => out,
        Err(e) => {
            eprintln!("{} error: {}", linker, e);
            return false;
        }
    };

    if !ld_output.status.success() {
        eprintln!(
            "{} stderr:\n{}",
            linker,
            String::from_utf8_lossy(&ld_output.stderr)
        );
        return false;
    }

    let run_output = if cfg!(windows) {
        Command::new("cmd").args(&["/C", &exe_file]).output()
    } else {
        Command::new(format!("./{}", exe_file)).output()  // ИСПРАВЛЕНО: добавил ./
    };

    let run_result = match run_output {
        Ok(out) => out,
        Err(e) => {
            eprintln!("Run error: {}", e);
            return false;
        }
    };

    let exit_code = run_result.status.code().unwrap_or(-1);
    println!("Exit code: {}, expected: {}", exit_code, expected_exit_code);

    let _ = fs::remove_file(&asm_file);
    let _ = fs::remove_file(&obj_file);
    let _ = fs::remove_file(&exe_file);

    thread::sleep(Duration::from_millis(100));

    exit_code == expected_exit_code
}

#[test]
fn test_integration_simple_return() {
    let source = r#"
        fn main() -> int {
            return 42;
        }
    "#;
    assert!(compile_and_run(source, 42));
}

#[test]
fn test_integration_addition() {
    let source = r#"
        fn main() -> int {
            int x = 5;
            int y = 3;
            return x + y;
        }
    "#;
    assert!(compile_and_run(source, 8));
}

#[test]
fn test_integration_multiplication() {
    let source = r#"
        fn main() -> int {
            return 6 * 7;
        }
    "#;
    assert!(compile_and_run(source, 42));
}

#[test]
fn test_integration_function_call() {
    let source = r#"
        fn add(int a, int b) -> int {
            return a + b;
        }

        fn main() -> int {
            return add(10, 20);
        }
    "#;
    assert!(compile_and_run(source, 30));
}

#[test]
fn test_integration_if_statement() {
    let source = r#"
        fn main() -> int {
            int x = 10;
            int result = 0;
            if (x > 5) {
                result = 1;
            }
            return result;
        }
    "#;
    assert!(compile_and_run(source, 1));
}

#[test]
fn test_integration_if_else_true_branch() {
    let source = r#"
        fn main() -> int {
            int x = 10;
            if (x > 5) {
                return 100;
            } else {
                return 0;
            }
        }
    "#;
    assert!(compile_and_run(source, 100));
}

#[test]
fn test_integration_if_else_false_branch() {
    let source = r#"
        fn main() -> int {
            int x = 3;
            if (x > 5) {
                return 100;
            } else {
                return 0;
            }
        }
    "#;
    assert!(compile_and_run(source, 0));
}

#[test]
fn test_integration_nested_if() {
    let source = r#"
        fn main() -> int {
            int x = 10;
            int y = 20;
            if (x > 5) {
                if (y > 15) {
                    return 2;
                } else {
                    return 1;
                }
            } else {
                return 0;
            }
        }
    "#;
    assert!(compile_and_run(source, 2));
}

#[test]
fn test_integration_nested_if_second_case() {
    let source = r#"
        fn main() -> int {
            int x = 10;
            int y = 10;
            if (x > 5) {
                if (y > 15) {
                    return 2;
                } else {
                    return 1;
                }
            } else {
                return 0;
            }
        }
    "#;
    assert!(compile_and_run(source, 1));
}

#[test]
fn test_integration_while_loop_sum() {
    let source = r#"
        fn main() -> int {
            int sum = 0;
            int i = 0;
            while (i < 10) {
                sum = sum + i;
                i = i + 1;
            }
            return sum;
        }
    "#;
    assert!(compile_and_run(source, 45));
}

#[test]
fn test_integration_while_loop_single_iteration() {
    let source = r#"
        fn main() -> int {
            int i = 0;
            int count = 0;
            while (i < 1) {
                count = count + 1;
                i = i + 1;
            }
            return count;
        }
    "#;
    assert!(compile_and_run(source, 1));
}

#[test]
fn test_integration_while_loop_zero_iterations() {
    let source = r#"
        fn main() -> int {
            int i = 10;
            int count = 0;
            while (i < 5) {
                count = count + 1;
                i = i + 1;
            }
            return count;
        }
    "#;
    assert!(compile_and_run(source, 0));
}

#[test]
fn test_integration_for_loop_sum() {
    let source = r#"
        fn main() -> int {
            int sum = 0;
            int i;
            for (i = 1; i <= 5; i = i + 1) {
                sum = sum + i;
            }
            return sum;
        }
    "#;
    assert!(compile_and_run(source, 15));
}

#[test]
fn test_integration_for_loop_factorial() {
    let source = r#"
        fn main() -> int {
            int result = 1;
            int i;
            for (i = 2; i <= 5; i = i + 1) {
                result = result * i;
            }
            return result;
        }
    "#;
    assert!(compile_and_run(source, 120));
}

#[test]
fn test_integration_for_loop_counting_down() {
    let source = r#"
        fn main() -> int {
            int sum = 0;
            int i;
            for (i = 5; i >= 1; i = i - 1) {
                sum = sum + i;
            }
            return sum;
        }
    "#;
    assert!(compile_and_run(source, 15));
}

#[test]
fn test_integration_short_circuit_and_true() {
    let source = r#"
        fn main() -> int {
            int a = 5;
            int b = 10;
            int result = 0;
            if (a != 0 && b > 5) {
                result = 1;
            }
            return result;
        }
    "#;
    assert!(compile_and_run(source, 1));
}

#[test]
fn test_integration_short_circuit_and_false() {
    let source = r#"
        fn main() -> int {
            int a = 0;
            int b = 10;
            int result = 0;
            if (a != 0 && b / a > 2) {
                result = 1;
            }
            return result;
        }
    "#;
    assert!(compile_and_run(source, 0));
}

#[test]
fn test_integration_short_circuit_or() {
    let source = r#"
        fn main() -> int {
            int a = 5;
            int result = 0;
            if (a != 0 || a / 0 > 2) {
                result = 1;
            }
            return result;
        }
    "#;
    assert!(compile_and_run(source, 1));
}

#[test]
fn test_integration_complex_boolean() {
    let source = r#"
        fn main() -> int {
            int a = 5;
            int b = 10;
            int c = 3;
            int result = 0;
            if ((a > 0 && b > 0) || c > 10) {
                result = 1;
            }
            return result;
        }
    "#;
    assert!(compile_and_run(source, 1));
}

#[test]
fn test_integration_not_operator() {
    let source = r#"
        fn main() -> int {
            bool flag = false;
            int result = 0;
            if (!flag) {
                result = 1;
            }
            return result;
        }
    "#;
    assert!(compile_and_run(source, 1));
}

#[test]
fn test_integration_not_operator_false() {
    let source = r#"
        fn main() -> int {
            bool flag = true;
            int result = 0;
            if (!flag) {
                result = 1;
            }
            return result;
        }
    "#;
    assert!(compile_and_run(source, 0));
}

#[test]
fn test_integration_complex_expression_precedence() {
    let source = r#"
        fn main() -> int {
            int a = 6;
            int b = 4;
            int c = 2;
            return ((a + b) * c) / (a - b);
        }
    "#;
    assert!(compile_and_run(source, 10));
}

#[test]
fn test_integration_loop_with_conditional() {
    let source = r#"
        fn main() -> int {
            int sum = 0;
            int i = 0;
            while (i < 5) {
                if (i % 2 == 0) {
                    sum = sum + i;
                } else {
                    sum = sum + 1;
                }
                i = i + 1;
            }
            return sum;
        }
    "#;
    assert!(compile_and_run(source, 8));
}

#[test]
fn test_integration_factorial_recursive() {
    let source = r#"
        fn factorial(int n) -> int {
            if (n <= 1) {
                return 1;
            }
            return n * factorial(n - 1);
        }

        fn main() -> int {
            return factorial(5);
        }
    "#;
    assert!(compile_and_run(source, 120));
}

#[test]
fn test_integration_fibonacci() {
    let source = r#"
        fn fib(int n) -> int {
            if (n <= 1) {
                return n;
            }
            return fib(n - 1) + fib(n - 2);
        }

        fn main() -> int {
            return fib(6);
        }
    "#;
    assert!(compile_and_run(source, 8));
}

#[test]
fn test_integration_multiple_conditions() {
    let source = r#"
        fn main() -> int {
            int score = 85;
            int grade = 0;
            if (score >= 90) {
                grade = 5;
            } else {
                if (score >= 80) {
                    grade = 4;
                } else {
                    if (score >= 70) {
                        grade = 3;
                    } else {
                        grade = 2;
                    }
                }
            }
            return grade;
        }
    "#;
    assert!(compile_and_run(source, 4));
}

#[test]
fn test_integration_assignment_operators() {
    let source = r#"
        fn main() -> int {
            int x = 10;
            x = x + 5;
            return x;
        }
    "#;
    assert!(compile_and_run(source, 15));
}

#[test]
fn test_integration_relational_operators_eq() {
    let source = r#"
        fn main() -> int {
            int result = 0;
            if (5 == 5) {
                result = 1;
            }
            return result;
        }
    "#;
    assert!(compile_and_run(source, 1));
}

#[test]
fn test_integration_relational_operators_ne() {
    let source = r#"
        fn main() -> int {
            int result = 0;
            if (5 != 3) {
                result = 1;
            }
            return result;
        }
    "#;
    assert!(compile_and_run(source, 1));
}

#[test]
fn test_integration_relational_operators_lt() {
    let source = r#"
        fn main() -> int {
            int result = 0;
            if (3 < 5) {
                result = 1;
            }
            return result;
        }
    "#;
    assert!(compile_and_run(source, 1));
}

#[test]
fn test_integration_relational_operators_le() {
    let source = r#"
        fn main() -> int {
            int result = 0;
            if (5 <= 5) {
                result = 1;
            }
            return result;
        }
    "#;
    assert!(compile_and_run(source, 1));
}

#[test]
fn test_integration_relational_operators_gt() {
    let source = r#"
        fn main() -> int {
            int result = 0;
            if (5 > 3) {
                result = 1;
            }
            return result;
        }
    "#;
    assert!(compile_and_run(source, 1));
}

#[test]
fn test_integration_relational_operators_ge() {
    let source = r#"
        fn main() -> int {
            int result = 0;
            if (5 >= 5) {
                result = 1;
            }
            return result;
        }
    "#;
    assert!(compile_and_run(source, 1));
}
