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

    assert!(
        result.assembly.contains("cmp")
            || result.assembly.contains("j") && result.assembly.contains("ret"),
        "Нет ни сравнения, ни условного перехода"
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

#[test]
fn test_if_statement_basic() {
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

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid(), "Ошибки: {:?}", parse_output.errors);

    let ir = ir_program.expect("IR не сгенерирован");

    use minic::ir::IRPrinter;
    println!("=== IR ===");
    println!("{}", IRPrinter::to_text(&ir));
    println!("=== END IR ===");

    let result = generate_assembly(&ir, false);
    println!("=== ASM ===\n{}", result.assembly);
}

#[test]
fn test_if_else_statement() {
    use minic::ir::IRInstruction;
    let source = r#"
        fn main() -> int {
            int x = 3;
            int result = 0;
            if (x > 5) {
                result = 1;
            } else {
                result = -1;
            }
            return result;
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid(), "Ошибки: {:?}", parse_output.errors);

    let ir = ir_program.expect("IR не сгенерирован");

    let mut has_comparison = false;
    for func in &ir.functions {
        for block in func.blocks.values() {
            for instr in &block.instructions {
                if matches!(
                    instr,
                    IRInstruction::CmpGt(_, _, _)
                        | IRInstruction::CmpLt(_, _, _)
                        | IRInstruction::CmpEq(_, _, _)
                        | IRInstruction::CmpNe(_, _, _)
                        | IRInstruction::CmpGe(_, _, _)
                        | IRInstruction::CmpLe(_, _, _)
                        | IRInstruction::JumpIf(_, _)
                        | IRInstruction::JumpIfNot(_, _)
                ) {
                    has_comparison = true;
                }
            }
        }
    }

    let result = generate_assembly(&ir, false);
    println!("=== If-Else Statement ===\n{}", result.assembly);

    assert!(
        has_comparison || result.assembly.contains("jmp"),
        "Должна быть структура if-else с переходами"
    );
}

#[test]
fn test_nested_conditionals() {
    let source = r#"
        fn main() -> int {
            int x = 10;
            int y = 20;
            int result = 0;
            if (x > 5) {
                if (y > 15) {
                    result = 2;
                } else {
                    result = 1;
                }
            } else {
                result = 0;
            }
            return result;
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid(), "Ошибки: {:?}", parse_output.errors);

    let ir = ir_program.expect("IR не сгенерирован");
    let result = generate_assembly(&ir, false);

    println!("=== Nested Conditionals ===\n{}", result.assembly);

    let cmp_count = result.assembly.matches("cmp").count();
    assert!(
        cmp_count >= 2,
        "Должно быть минимум 2 сравнения, найдено: {}",
        cmp_count
    );
}

#[test]
fn test_relational_operators() {
    let source = r#"
        fn main() -> int {
            int a = 5;
            int b = 10;
            int result = 0;
            if (a < b && b > 0 && a <= 5 && b >= 10 && a == 5 && a != b) {
                result = 1;
            }
            return result;
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid(), "Ошибки: {:?}", parse_output.errors);

    let ir = ir_program.expect("IR не сгенерирован");
    let result = generate_assembly(&ir, false);

    println!("=== Relational Operators ===\n{}", result.assembly);

    assert!(result.assembly.contains("cmp"), "Нет сравнений");
}

#[test]
fn test_while_loop_basic() {
    let source = r#"
        fn main() -> int {
            int i = 0;
            int sum = 0;
            while (i < 5) {
                sum = sum + i;
                i = i + 1;
            }
            return sum;
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid(), "Ошибки: {:?}", parse_output.errors);

    let ir = ir_program.expect("IR не сгенерирован");
    let result = generate_assembly(&ir, false);

    println!("=== While Loop Basic ===\n{}", result.assembly);

    assert!(result.assembly.contains("jmp"), "Нет перехода цикла");
    assert!(result.assembly.contains("cmp"), "Нет сравнения в цикле");
}

#[test]
fn test_while_loop_zero_iterations() {
    let source = r#"
        fn main() -> int {
            int i = 10;
            int sum = 0;
            while (i < 5) {
                sum = sum + i;
                i = i + 1;
            }
            return sum;
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid(), "Ошибки: {:?}", parse_output.errors);

    let ir = ir_program.expect("IR не сгенерирован");
    let result = generate_assembly(&ir, false);

    println!("=== While Zero Iterations ===\n{}", result.assembly);

    assert!(result.assembly.contains("cmp"), "Нет сравнения условия");
}

#[test]
fn test_for_loop_basic() {
    let source = r#"
        fn main() -> int {
            int sum = 0;
            int i;
            for (i = 0; i < 5; i = i + 1) {
                sum = sum + i;
            }
            return sum;
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid(), "Ошибки: {:?}", parse_output.errors);

    let ir = ir_program.expect("IR не сгенерирован");
    let result = generate_assembly(&ir, false);

    println!("=== For Loop Basic ===\n{}", result.assembly);

    assert!(result.assembly.contains("cmp"), "Нет сравнения в for");
    assert!(result.assembly.contains("jmp"), "Нет перехода в for");
}

#[test]
fn test_nested_loops() {
    let source = r#"
        fn main() -> int {
            int sum = 0;
            int i = 0;
            while (i < 3) {
                int j = 0;
                while (j < 2) {
                    sum = sum + 1;
                    j = j + 1;
                }
                i = i + 1;
            }
            return sum;
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid(), "Ошибки: {:?}", parse_output.errors);

    let ir = ir_program.expect("IR не сгенерирован");
    let result = generate_assembly(&ir, false);

    println!("=== Nested Loops ===\n{}", result.assembly);

    let jmp_count = result.assembly.matches("jmp").count();
    assert!(
        jmp_count >= 2,
        "Должно быть минимум 2 перехода для вложенных циклов, найдено: {}",
        jmp_count
    );
}

#[test]
fn test_short_circuit_and() {
    let source = r#"
        fn main() -> int {
            int a = 0;
            int b = 5;
            int result = 0;
            // a == 0, поэтому b/a не вычисляется (короткая схема)
            if (a != 0 && b / a > 2) {
                result = 1;
            }
            return result;
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid(), "Ошибки: {:?}", parse_output.errors);

    let ir = ir_program.expect("IR не сгенерирован");
    let result = generate_assembly(&ir, false);

    println!("=== Short-Circuit AND ===\n{}", result.assembly);

    assert!(
        result.assembly.contains("j"),
        "Нет условного перехода для короткой схемы"
    );
}

#[test]
fn test_short_circuit_or() {
    let source = r#"
        fn main() -> int {
            int a = 1;
            int result = 0;
            // a != 0, поэтому правая часть не вычисляется
            if (a != 0 || a / 0 > 2) {
                result = 1;
            }
            return result;
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid(), "Ошибки: {:?}", parse_output.errors);

    let ir = ir_program.expect("IR не сгенерирован");
    let result = generate_assembly(&ir, false);

    println!("=== Short-Circuit OR ===\n{}", result.assembly);

    assert!(
        result.assembly.contains("jmp"),
        "Нет перехода для короткой схемы"
    );
}

#[test]
fn test_complex_boolean_expression() {
    let source = r#"
        fn main() -> int {
            int a = 5;
            int b = 10;
            int c = 15;
            int result = 0;
            if ((a > 0 && b > 0) || c > 20) {
                result = 1;
            }
            return result;
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid(), "Ошибки: {:?}", parse_output.errors);

    let ir = ir_program.expect("IR не сгенерирован");
    let result = generate_assembly(&ir, false);

    println!("=== Complex Boolean ===\n{}", result.assembly);

    assert!(result.assembly.contains("cmp"), "Нет сравнений");
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

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid(), "Ошибки: {:?}", parse_output.errors);

    let ir = ir_program.expect("IR не сгенерирован");
    let result = generate_assembly(&ir, false);

    println!("=== NOT Operator ===\n{}", result.assembly);

    assert!(
        result.assembly.contains("xor") || result.assembly.contains("not"),
        "Нет операции NOT"
    );
}

#[test]
fn test_operator_precedence() {
    let source = r#"
        fn main() -> int {
            int a = 5;
            int b = 3;
            int c = 2;
            // (a + b) * c = 16
            int result = (a + b) * c;
            return result;
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid(), "Ошибки: {:?}", parse_output.errors);

    let ir = ir_program.expect("IR не сгенерирован");
    let result = generate_assembly(&ir, false);

    println!("=== Operator Precedence ===\n{}", result.assembly);

    assert!(result.assembly.contains("add"), "Нет сложения");
    assert!(result.assembly.contains("imul"), "Нет умножения");
}

#[test]
fn test_complex_arithmetic_expression() {
    let source = r#"
        fn main() -> int {
            int a = 6;
            int b = 4;
            int c = 2;
            // ((a + b) * c) / (a - b) = (10 * 2) / 2 = 10
            int result = ((a + b) * c) / (a - b);
            return result;
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid(), "Ошибки: {:?}", parse_output.errors);

    let ir = ir_program.expect("IR не сгенерирован");
    let result = generate_assembly(&ir, false);

    println!("=== Complex Arithmetic ===\n{}", result.assembly);

    assert!(result.assembly.contains("add"), "Нет сложения");
    assert!(result.assembly.contains("imul"), "Нет умножения");
    assert!(result.assembly.contains("idiv"), "Нет деления");
}

#[test]
fn test_factorial_function() {
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

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid(), "Ошибки: {:?}", parse_output.errors);

    let ir = ir_program.expect("IR не сгенерирован");
    let result = generate_assembly(&ir, false);

    println!("=== Factorial ===\n{}", result.assembly);

    assert!(
        result.assembly.contains("factorial:"),
        "Нет функции factorial"
    );
    assert!(
        result.assembly.contains("call factorial") || result.assembly.contains("call factorial\n"),
        "Нет рекурсивного вызова"
    );
}

#[test]
fn test_fibonacci_sequence() {
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

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid(), "Ошибки: {:?}", parse_output.errors);

    let ir = ir_program.expect("IR не сгенерирован");
    let result = generate_assembly(&ir, false);

    println!("=== Fibonacci ===\n{}", result.assembly);

    assert!(result.assembly.contains("fib:"), "Нет функции fib");
    assert!(
        result.assembly.contains("call fib"),
        "Нет рекурсивного вызова"
    );
}
