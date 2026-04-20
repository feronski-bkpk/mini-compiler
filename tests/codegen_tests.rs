use minic::codegen::generate_assembly;
use minic::compiler::compile_with_ir;

#[test]
fn test_simple_addition_codegen() {
    let source = r#"
        fn main() -> int {
            int x = 5;
            int y = 3;
            return x + y;
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(
        parse_output.is_valid(),
        "Ошибки парсинга: {:?}",
        parse_output.errors
    );

    let ir = ir_program.expect("IR не сгенерирован");
    let result = generate_assembly(&ir, false);

    println!("=== Сгенерированный ассемблер ===\n{}", result.assembly);

    assert!(result.assembly.contains("main:"), "Нет метки main");
    assert!(result.assembly.contains("push rbp"), "Нет пролога");
    assert!(result.assembly.contains("ret"), "Нет ret");
}

#[test]
fn test_function_call_codegen() {
    let source = r#"
        fn add(int a, int b) -> int {
            return a + b;
        }

        fn main() -> int {
            return add(5, 3);
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(
        parse_output.is_valid(),
        "Ошибки парсинга: {:?}",
        parse_output.errors
    );

    let ir = ir_program.expect("IR не сгенерирован");
    let result = generate_assembly(&ir, false);

    println!("=== Сгенерированный ассемблер ===\n{}", result.assembly);

    assert!(result.assembly.contains("add:"), "Нет функции add");
    assert!(result.assembly.contains("main:"), "Нет функции main");
    assert!(
        result.assembly.contains("call add") || result.assembly.contains("call add\n"),
        "Нет вызова add"
    );
}

#[test]
fn test_conditional_codegen() {
    let source = r#"
        fn main() -> int {
            int x = 10;
            if (x > 5) {
                return 1;
            } else {
                return 0;
            }
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());

    let ir = ir_program.expect("IR не сгенерирован");
    let result = generate_assembly(&ir, false);

    println!("=== Сгенерированный ассемблер ===\n{}", result.assembly);

    assert!(result.assembly.contains("cmp"), "Нет сравнения");
    assert!(
        result.assembly.contains("jz") || result.assembly.contains("jnz"),
        "Нет условного перехода"
    );
}

#[test]
fn test_loop_codegen() {
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

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());

    let ir = ir_program.expect("IR не сгенерирован");
    let result = generate_assembly(&ir, false);

    println!("=== Сгенерированный ассемблер ===\n{}", result.assembly);

    assert!(result.assembly.contains("cmp"), "Нет сравнения");
    assert!(result.assembly.contains("jmp"), "Нет перехода");
}

#[test]
fn test_stack_frame_allocation() {
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

    let ir = ir_program.expect("IR не сгенерирован");
    let result = generate_assembly(&ir, false);

    println!("=== Сгенерированный ассемблер ===\n{}", result.assembly);

    assert!(result.assembly.contains("sub rsp,") || result.frame_size > 0);
    println!("Размер фрейма: {} байт", result.frame_size);
}

#[test]
fn test_optimization_flag() {
    let source = r#"
        fn main() -> int {
            return 5 + 3;
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());

    let ir = ir_program.expect("IR не сгенерирован");
    let result_unoptimized = generate_assembly(&ir, false);
    let result_optimized = generate_assembly(&ir, true);

    println!("=== Без оптимизаций ===\n{}", result_unoptimized.assembly);
    println!("=== С оптимизациями ===\n{}", result_optimized.assembly);

    println!(
        "Без оптимизаций: {} инструкций",
        result_unoptimized.instruction_count
    );
    println!(
        "С оптимизациями: {} инструкций",
        result_optimized.instruction_count
    );
}

#[test]
fn test_multiple_functions() {
    let source = r#"
        fn first() -> int {
            return 1;
        }

        fn second() -> int {
            return 2;
        }

        fn third() -> int {
            return 3;
        }

        fn main() -> int {
            return first() + second() + third();
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());

    let ir = ir_program.expect("IR не сгенерирован");
    let result = generate_assembly(&ir, false);

    println!("=== Сгенерированный ассемблер ===\n{}", result.assembly);

    assert!(result.assembly.contains("first:"), "Нет функции first");
    assert!(result.assembly.contains("second:"), "Нет функции second");
    assert!(result.assembly.contains("third:"), "Нет функции third");
    assert!(result.assembly.contains("main:"), "Нет функции main");
}

#[test]
fn test_codegen_result_fields() {
    let source = r#"
        fn main() -> int {
            return 42;
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());

    let ir = ir_program.expect("IR не сгенерирован");
    let result = generate_assembly(&ir, false);

    assert!(!result.assembly.is_empty(), "Ассемблер пуст");
    assert!(result.instruction_count > 0, "Нет инструкций");

    println!("Статистика кодогенерации:");
    println!("  Инструкций: {}", result.instruction_count);
    println!("  Регистров: {:?}", result.registers_used);
    println!("  Размер фрейма: {} байт", result.frame_size);
}
