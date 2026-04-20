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
        Command::new(&exe_file).output()
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
