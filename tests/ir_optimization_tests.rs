//! Тесты для оптимизаций IR

use minic::compiler;
use minic::ir::*;

/// Тест свертки констант
#[test]
fn test_constant_folding() {
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

    let mut program = ir_program.unwrap();

    let before_stats = IRStatistics::compute(&program);
    let before_add_mul = before_stats.instruction_counts.get("ADD").unwrap_or(&0)
        + before_stats.instruction_counts.get("MUL").unwrap_or(&0)
        + before_stats.instruction_counts.get("SUB").unwrap_or(&0);

    let report = PeepholeOptimizer::optimize(&mut program);

    let after_stats = IRStatistics::compute(&program);
    let after_add_mul = after_stats.instruction_counts.get("ADD").unwrap_or(&0)
        + after_stats.instruction_counts.get("MUL").unwrap_or(&0)
        + after_stats.instruction_counts.get("SUB").unwrap_or(&0);

    assert!(
        after_add_mul < before_add_mul || report.simplifications_applied > 0,
        "Константы должны быть свернуты: было {} арифметических операций, стало {}",
        before_add_mul,
        after_add_mul
    );

    assert!(
        report.changes_made > 0 || report.simplifications_applied > 0,
        "Должны быть сделаны оптимизации"
    );
}

/// Тест алгебраических упрощений
#[test]
fn test_algebraic_simplifications() {
    let source = r#"
        fn main() -> int {
            int x = 5;
            int a = x + 0;
            int b = x * 1;
            int c = x * 0;
            return a + b + c;
        }
    "#;

    let (_, ir_program) = compiler::compile_with_ir(source, vec![]);
    assert!(ir_program.is_some());

    let mut program = ir_program.unwrap();

    let before_stats = IRStatistics::compute(&program);
    let before_arithmetic = *before_stats.instruction_counts.get("ADD").unwrap_or(&0)
        + *before_stats.instruction_counts.get("SUB").unwrap_or(&0)
        + *before_stats.instruction_counts.get("MUL").unwrap_or(&0)
        + *before_stats.instruction_counts.get("DIV").unwrap_or(&0);

    let report = PeepholeOptimizer::optimize(&mut program);

    let after_stats = IRStatistics::compute(&program);
    let after_arithmetic = *after_stats.instruction_counts.get("ADD").unwrap_or(&0)
        + *after_stats.instruction_counts.get("SUB").unwrap_or(&0)
        + *after_stats.instruction_counts.get("MUL").unwrap_or(&0)
        + *after_stats.instruction_counts.get("DIV").unwrap_or(&0);

    assert!(
        after_arithmetic < before_arithmetic,
        "Арифметических операций должно стать меньше: было {}, стало {}",
        before_arithmetic,
        after_arithmetic
    );

    assert!(
        report.simplifications_applied > 0,
        "Должны быть применены алгебраические упрощения"
    );
}

/// Тест удаления мертвого кода
#[test]
fn test_dead_code_elimination() {
    let source = r#"
        fn main() -> int {
            int x = 5;
            int y = 10;
            int z = x + y;
            int w = z * 2;
            // w не используется
            return x;
        }
    "#;

    let (_, ir_program) = compiler::compile_with_ir(source, vec![]);
    assert!(ir_program.is_some());

    let mut program = ir_program.unwrap();

    println!("\n=== IR ДО ОПТИМИЗАЦИИ ===");
    println!("{}", IRPrinter::to_text(&program));

    let before_stats = IRStatistics::compute(&program);
    println!("Статистика до: {:?}", before_stats.instruction_counts);

    let report = PeepholeOptimizer::optimize(&mut program);

    println!("\n=== IR ПОСЛЕ ОПТИМИЗАЦИИ ===");
    println!("{}", IRPrinter::to_text(&program));

    let after_stats = IRStatistics::compute(&program);
    println!("Статистика после: {:?}", after_stats.instruction_counts);
    println!("Отчет: {:?}", report);

    assert!(
        after_stats.total_instructions < before_stats.total_instructions,
        "Мертвый код должен быть удален: было {} инструкций, стало {}",
        before_stats.total_instructions,
        after_stats.total_instructions
    );

    assert!(
        report.instructions_removed > 0 || report.dead_code_removed > 0,
        "Должны быть удалены инструкции"
    );
}
