//! Тесты соответствия System V AMD64 ABI

use minic::codegen::generate_assembly;
use minic::compiler::compile_with_ir;

#[test]
fn test_abi_parameter_passing() {
    let source = r#"
        fn test(int a, int b, int c, int d, int e, int f, int g) -> int {
            return a + b + c + d + e + f + g;
        }

        fn main() -> int {
            return test(1, 2, 3, 4, 5, 6, 7);
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());

    let ir = ir_program.unwrap();
    let result = generate_assembly(&ir, false);

    assert!(result.assembly.contains("mov rdi,") || result.assembly.contains("mov edi,"));
    assert!(result.assembly.contains("mov rsi,") || result.assembly.contains("mov esi,"));
    assert!(result.assembly.contains("mov rdx,") || result.assembly.contains("mov edx,"));
    assert!(result.assembly.contains("mov rcx,") || result.assembly.contains("mov ecx,"));
    assert!(result.assembly.contains("mov r8,"));
    assert!(result.assembly.contains("mov r9,"));

    assert!(result.assembly.contains("push") || result.assembly.contains("sub rsp"));
}

#[test]
fn test_abi_return_value() {
    let source = r#"
        fn main() -> int {
            return 42;
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());

    let ir = ir_program.unwrap();
    let result = generate_assembly(&ir, false);

    assert!(result.assembly.contains("mov rax, 42") || result.assembly.contains("mov eax, 42"));
}

#[test]
fn test_abi_stack_alignment() {
    let source = r#"
        fn main() -> int {
            int a = 1;
            int b = 2;
            int c = 3;
            int d = 4;
            return a + b + c + d;
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());

    let ir = ir_program.unwrap();
    let result = generate_assembly(&ir, false);

    for line in result.assembly.lines() {
        if line.contains("sub rsp,") {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() > 1 {
                let size_str = parts[1].trim();
                if let Ok(size) = size_str.parse::<i32>() {
                    assert_eq!(size % 16, 0, "Stack size {} not aligned to 16", size);
                }
            }
        }
    }
}

#[test]
fn test_abi_callee_saved_registers() {
    let source = r#"
        fn nested() -> int {
            return 100;
        }

        fn main() -> int {
            return nested();
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());

    let ir = ir_program.unwrap();
    let result = generate_assembly(&ir, false);

    assert!(result.assembly.contains("push rbp"));
}

#[test]
fn test_abi_leaf_function_red_zone() {
    let source = r#"
        fn leaf() -> int {
            int x = 42;
            return x;
        }

        fn main() -> int {
            return leaf();
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());

    let ir = ir_program.unwrap();
    let result = generate_assembly(&ir, true);

    println!("Leaf function assembly:\n{}", result.assembly);
}
