use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static TEST_ID: AtomicUsize = AtomicUsize::new(0);

fn compile_and_run_file(path: &str) -> i32 {
    let source = fs::read_to_string(path).expect(&format!("Cannot read {}", path));
    compile_and_run(&source)
}

fn compile_and_run(source: &str) -> i32 {
    let id = TEST_ID.fetch_add(1, Ordering::SeqCst);
    let asm_file = format!("test_cf_{}.asm", id);
    let obj_file = format!("test_cf_{}.o", id);
    let exe_file = format!("test_cf_{}", id);

    let (parse_output, ir_program) = minic::compiler::compile_with_ir(source, vec![]);
    assert!(
        parse_output.is_valid(),
        "Parse errors: {:?}",
        parse_output.errors
    );
    let ir = ir_program.expect("IR не сгенерирован");
    let result = minic::codegen::generate_assembly(&ir, false);

    fs::write(&asm_file, &result.assembly).unwrap();

    let nasm = Command::new("nasm")
        .args(&["-f", "elf64", &asm_file, "-o", &obj_file])
        .status()
        .expect("nasm not found");
    assert!(nasm.success(), "NASM failed for {}", asm_file);

    let ld = Command::new("ld")
        .args(&[&obj_file, "-o", &exe_file])
        .status()
        .expect("ld not found");
    assert!(ld.success(), "ld failed for {}", obj_file);

    let output = Command::new(format!("./{}", exe_file))
        .output()
        .expect("run failed");

    let _ = fs::remove_file(&asm_file);
    let _ = fs::remove_file(&obj_file);
    let _ = fs::remove_file(&exe_file);

    output.status.code().unwrap_or(-1)
}

#[test]
fn test_simple_if() {
    assert_eq!(
        compile_and_run_file("tests/control_flow/valid/conditionals/simple_if.src"),
        1
    );
}

#[test]
fn test_if_else() {
    assert_eq!(
        compile_and_run_file("tests/control_flow/valid/conditionals/if_else.src"),
        0
    );
}

#[test]
fn test_nested_if() {
    assert_eq!(
        compile_and_run_file("tests/control_flow/valid/conditionals/nested_if.src"),
        2
    );
}

#[test]
fn test_switch() {
    assert_eq!(
        compile_and_run_file("tests/control_flow/valid/conditionals/switch.src"),
        20
    );
}

#[test]
fn test_unsigned_cmp() {
    let source = r#"
        fn main() -> int {
            int a = 5;
            int b = 10;
            int result = 0;
            if (a < b) {
                result = 1;
            }
            return result;
        }
    "#;
    assert_eq!(compile_and_run(source), 1);
}

#[test]
fn test_while_sum() {
    assert_eq!(
        compile_and_run_file("tests/control_flow/valid/loops/while_sum.src"),
        45
    );
}

#[test]
fn test_for_factorial() {
    assert_eq!(
        compile_and_run_file("tests/control_flow/valid/loops/for_factorial.src"),
        120
    );
}

#[test]
fn test_nested_loops() {
    assert_eq!(
        compile_and_run_file("tests/control_flow/valid/loops/nested_loops.src"),
        6
    );
}

#[test]
fn test_break_while() {
    assert_eq!(
        compile_and_run_file("tests/control_flow/valid/loops/break_while.src"),
        101
    );
}

#[test]
fn test_continue_while() {
    let source = r#"
        fn main() -> int {
            int i = 0;
            int sum = 0;
            while (i < 10) {
                i = i + 1;
                if (i % 2 == 0) {
                    continue;
                }
                sum = sum + i;
            }
            return sum;
        }
    "#;
    assert_eq!(compile_and_run(source), 25);
}

#[test]
fn test_short_circuit_and() {
    assert_eq!(
        compile_and_run_file("tests/control_flow/valid/logical_ops/short_circuit_and.src"),
        0
    );
}

#[test]
fn test_short_circuit_or() {
    assert_eq!(
        compile_and_run_file("tests/control_flow/valid/logical_ops/short_circuit_or.src"),
        1
    );
}

#[test]
fn test_complex_boolean() {
    assert_eq!(
        compile_and_run_file("tests/control_flow/valid/logical_ops/complex_boolean.src"),
        1
    );
}

#[test]
fn test_not_operator() {
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
    assert_eq!(compile_and_run(source), 1);
}

#[test]
fn test_precedence() {
    assert_eq!(
        compile_and_run_file("tests/control_flow/valid/complex_expressions/precedence.src"),
        10
    );
}

#[test]
fn test_mixed_types() {
    assert_eq!(
        compile_and_run_file("tests/control_flow/valid/complex_expressions/mixed_types.src"),
        15
    );
}

#[test]
fn test_int_to_float_conversion() {
    let source = r#"
        fn main() -> int {
            int x = 5;
            float y = 3.14;
            float z = x + y;
            return 15;
        }
    "#;
    assert_eq!(compile_and_run(source), 15);
}

#[test]
fn test_type_promotion_int_float() {
    let source = r#"
        fn main() -> int {
            int a = 10;
            float b = 2.5;
            float c = a + b;
            return 12;
        }
    "#;
    assert_eq!(compile_and_run(source), 12);
}

#[test]
fn test_empty_loop() {
    let source = r#"
        fn main() -> int {
            int i = 0;
            while (i < 10) {
                i = i + 1;
            }
            return i;
        }
    "#;
    assert_eq!(compile_and_run(source), 10);
}

#[test]
fn test_zero_iterations() {
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
    assert_eq!(compile_and_run(source), 0);
}

#[test]
fn test_nested_break() {
    let source = r#"
        fn main() -> int {
            int sum = 0;
            int i = 0;
            while (i < 5) {
                int j = 0;
                while (j < 5) {
                    if (i == 2 && j == 2) {
                        break;
                    }
                    j = j + 1;
                }
                sum = sum + j;
                i = i + 1;
            }
            return sum;
        }
    "#;
    assert_eq!(compile_and_run(source), 20);
}

#[test]
fn test_pointer_comparison() {
    let source = r#"
        fn main() -> int {
            int a = 5;
            int b = 5;
            if (a == b) {
                return 1;
            }
            return 0;
        }
    "#;
    assert_eq!(compile_and_run(source), 1);
}

#[test]
fn test_bool_direct_test() {
    let source = r#"
        fn main() -> int {
            bool flag = true;
            if (flag) {
                return 1;
            }
            return 0;
        }
    "#;
    assert_eq!(compile_and_run(source), 1);
}

#[test]
fn test_switch_with_default() {
    let source = r#"
        fn main() -> int {
            int x = 99;
            int result = 0;
            switch (x) {
                case 1:
                    result = 10;
                case 2:
                    result = 20;
                default:
                    result = 99;
            }
            return result;
        }
    "#;
    assert_eq!(compile_and_run(source), 99);
}
