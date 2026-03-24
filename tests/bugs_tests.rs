//! Тесты для выявленных ошибок и неточностей

use minic::compiler;
use minic::ir::*;

/// Тест 1: Инициализация переменных константами должна генерировать MOVE инструкции
#[test]
fn test_variable_initialization() {
    let source = r#"
        fn main() -> int {
            int a = 5;
            int b = 3;
            int c = a + b;
            return c;
        }
    "#;

    let (_, ir_program) = compiler::compile_with_ir(source, vec![]);
    assert!(ir_program.is_some());

    let program = ir_program.unwrap();
    let main = program.get_function("main").unwrap();

    println!("=== ALL INSTRUCTIONS ===");
    for block in main.blocks.values() {
        for instr in &block.instructions {
            println!("  {}", instr);
        }
    }
    println!("========================");

    let mut has_move_for_a = false;
    let mut has_move_for_b = false;
    let mut has_add = false;

    for block in main.blocks.values() {
        for instr in &block.instructions {
            match instr {
                IRInstruction::Move(dest, _src) => {
                    if let Operand::Variable(name) = dest {
                        if name == "a" {
                            has_move_for_a = true;
                            println!("Found move for a: {}", instr);
                        }
                        if name == "b" {
                            has_move_for_b = true;
                            println!("Found move for b: {}", instr);
                        }
                    }
                }
                IRInstruction::Add(_, _, _) => has_add = true,
                _ => {}
            }
        }
    }

    assert!(has_move_for_a, "Должна быть инструкция MOVE для a");
    assert!(has_move_for_b, "Должна быть инструкция MOVE для b");
    assert!(has_add, "Должна быть инструкция ADD для a + b");
}

/// Тест 2: Семантический анализ должен показывать локальные переменные
#[test]
fn test_semantic_analysis_shows_locals() {
    let source = r#"
        fn main() -> int {
            int x = 5;
            int y = 10;
            return x + y;
        }
    "#;

    let parse_output = compiler::syntactic_analysis(source);
    assert!(parse_output.ast.is_some());

    let mut analyzer = minic::semantic::SemanticAnalyzer::new();
    let output = analyzer.analyze(parse_output.ast.unwrap());

    assert!(
        !output.has_errors(),
        "Семантический анализ должен пройти без ошибок"
    );

    let errors_str = output.errors.to_string();
    assert!(
        !errors_str.contains("не объявлена"),
        "Не должно быть ошибок о необъявленных переменных: {}",
        errors_str
    );
}

/// Тест 3: Вложенные if должны генерировать правильный CFG без мертвого кода
#[test]
fn test_nested_if_no_dead_code() {
    let source = r#"
        fn main() -> int {
            int x = 10;
            int y = 20;
            int z;
            if (x > 5) {
                if (y > 15) {
                    z = 100;
                } else {
                    z = 200;
                }
            } else {
                z = 300;
            }
            return z;
        }
    "#;

    let (_, ir_program) = compiler::compile_with_ir(source, vec![]);
    assert!(ir_program.is_some());

    let program = ir_program.unwrap();
    let main = program.get_function("main").unwrap();

    println!("=== IR for nested if ===");
    println!("{}", IRPrinter::to_text(&program));
    println!("========================");

    for block in main.blocks.values() {
        let instructions = &block.instructions;
        for i in 0..instructions.len() {
            if let IRInstruction::Jump(_) = &instructions[i] {
                assert_eq!(
                    i,
                    instructions.len() - 1,
                    "JUMP должен быть последней инструкцией в блоке. Блок {} имеет {} инструкций, JUMP на позиции {}",
                    block.label,
                    instructions.len(),
                    i
                );
            }
        }
    }
}

/// Тест 4: Свертка констант должна работать
#[test]
fn test_constant_folding_in_ir() {
    let source = r#"
        fn main() -> int {
            int x = 2 + 3;
            int y = 5 * 4;
            int z = 10 - 3;
            return x + y + z;
        }
    "#;

    let (_, ir_program) = compiler::compile_with_ir(source, vec![]);
    assert!(ir_program.is_some());

    let program = ir_program.unwrap();

    println!("=== IR BEFORE OPTIMIZATION ===");
    println!("{}", IRPrinter::to_text(&program));
    println!("==============================");

    let mut program = program;

    let before_stats = IRStatistics::compute(&program);
    let before_arithmetic = *before_stats.instruction_counts.get("ADD").unwrap_or(&0)
        + *before_stats.instruction_counts.get("SUB").unwrap_or(&0)
        + *before_stats.instruction_counts.get("MUL").unwrap_or(&0);

    println!("Before arithmetic ops: {}", before_arithmetic);

    let report = PeepholeOptimizer::optimize(&mut program);

    let after_stats = IRStatistics::compute(&program);
    let after_arithmetic = *after_stats.instruction_counts.get("ADD").unwrap_or(&0)
        + *after_stats.instruction_counts.get("SUB").unwrap_or(&0)
        + *after_stats.instruction_counts.get("MUL").unwrap_or(&0);

    println!("After arithmetic ops: {}", after_arithmetic);
    println!("Report: {:?}", report);

    assert!(
        after_arithmetic < before_arithmetic || report.simplifications_applied > 0,
        "Константы должны быть свернуты: было {} арифметических операций, стало {}",
        before_arithmetic,
        after_arithmetic
    );
}
