//! Тесты для генерации IR

use minic::compiler;
use minic::ir::*;

/// Тест генерации IR для простого выражения
#[test]
fn test_ir_simple_expression() {
    let source = r#"
        fn main() -> int {
            int x = 2 + 3;
            return x;
        }
    "#;

    let (_, ir_program) = compiler::compile_with_ir(source, vec![]);
    assert!(ir_program.is_some());

    let program = ir_program.unwrap();
    let main = program.get_function("main").unwrap();

    assert!(!main.blocks.is_empty());

    let mut has_add = false;
    for block in main.blocks.values() {
        for instr in &block.instructions {
            if matches!(instr, IRInstruction::Add(_, _, _)) {
                has_add = true;
            }
        }
    }
    assert!(has_add, "Должна быть инструкция ADD");
}

/// Тест генерации IR для if-else
#[test]
fn test_ir_if_statement() {
    let source = r#"
        fn main() -> int {
            int x = 5;
            int y;
            if (x > 0) {
                y = 10;
            } else {
                y = 20;
            }
            return y;
        }
    "#;

    let (_, ir_program) = compiler::compile_with_ir(source, vec![]);
    assert!(ir_program.is_some());

    let program = ir_program.unwrap();
    let main = program.get_function("main").unwrap();

    println!("\n=== IR for if statement test ===");
    println!("{}", IRPrinter::to_text(&program));
    println!("================================\n");

    let mut has_conditional_jump = false;
    for block in main.blocks.values() {
        for instr in &block.instructions {
            match instr {
                IRInstruction::JumpIf(_, _) | IRInstruction::JumpIfNot(_, _) => {
                    has_conditional_jump = true;
                    println!("Found conditional jump: {}", instr);
                }
                _ => {}
            }
        }
    }

    assert!(
        has_conditional_jump,
        "Должна быть инструкция JUMP_IF или JUMP_IF_NOT"
    );
}

/// Тест генерации IR для while цикла
#[test]
fn test_ir_while_loop() {
    let source = r#"
        fn main() -> int {
            int i = 0;
            int sum = 0;
            while (i < 10) {
                sum = sum + i;
                i = i + 1;
            }
            return sum;
        }
    "#;

    let (_, ir_program) = compiler::compile_with_ir(source, vec![]);
    assert!(ir_program.is_some());

    let program = ir_program.unwrap();
    let main = program.get_function("main").unwrap();

    let mut has_cmp = false;
    let mut has_jump = false;
    let mut has_jump_if = false;

    for block in main.blocks.values() {
        for instr in &block.instructions {
            match instr {
                IRInstruction::CmpLt(_, _, _) => has_cmp = true,
                IRInstruction::Jump(_) => has_jump = true,
                IRInstruction::JumpIf(_, _) | IRInstruction::JumpIfNot(_, _) => has_jump_if = true,
                _ => {}
            }
        }
    }

    assert!(has_cmp, "Должна быть инструкция CMP_LT");
    assert!(has_jump, "Должна быть инструкция JUMP");
    assert!(
        has_jump_if,
        "Должна быть инструкция JUMP_IF или JUMP_IF_NOT"
    );
}

/// Тест генерации IR для рекурсивной функции
#[test]
fn test_ir_recursive_function() {
    let source = r#"
        fn factorial(int n) -> int {
            if (n <= 1) {
                return 1;
            } else {
                return n * factorial(n - 1);
            }
        }
    "#;

    let (_, ir_program) = compiler::compile_with_ir(source, vec![]);
    assert!(ir_program.is_some());

    let program = ir_program.unwrap();
    let factorial = program.get_function("factorial").unwrap();

    println!("=== IR for factorial ===");
    println!("{}", IRPrinter::to_text(&program));
    println!("========================\n");

    let mut has_call = false;
    for block in factorial.blocks.values() {
        for instr in &block.instructions {
            if let IRInstruction::Call(_, func, _) = instr {
                if let Operand::Variable(name) = func {
                    if name == "factorial" {
                        has_call = true;
                        println!("Found recursive call: {}", instr);
                    }
                }
            }
        }
    }

    assert!(has_call, "Должен быть рекурсивный вызов factorial");
}

/// Тест статистики IR
#[test]
fn test_ir_statistics() {
    let source = r#"
        fn main() -> int {
            int x = 2 + 3;
            int y = x * 4;
            return y;
        }
    "#;

    let (_, ir_program) = compiler::compile_with_ir(source, vec![]);
    assert!(ir_program.is_some());

    let program = ir_program.unwrap();

    println!("=== IR for statistics test ===");
    println!("{}", IRPrinter::to_text(&program));
    println!("================================\n");

    let stats = IRStatistics::compute(&program);
    println!("Statistics: {:?}", stats);

    assert!(stats.total_instructions > 0);
    assert!(stats.basic_block_count > 0);
    assert!(
        stats.temporary_count > 0,
        "temporary_count = {}",
        stats.temporary_count
    );
}

/// Тест валидации IR (проверка, что все блоки заканчиваются терминатором)
#[test]
fn test_ir_validation_terminators() {
    let source = r#"
        fn main() -> int {
            return 42;
        }
    "#;

    let (_, ir_program) = compiler::compile_with_ir(source, vec![]);
    assert!(ir_program.is_some());

    let program = ir_program.unwrap();

    // Используем итератор по Vec вместо HashMap
    for func in &program.functions {
        for block in func.blocks.values() {
            if block.instructions.is_empty() {
                continue;
            }
            let last = block.instructions.last().unwrap();
            assert!(
                last.is_terminator(),
                "Блок {} должен заканчиваться терминатором, но заканчивается на: {}",
                block.label,
                last
            );
        }
    }
}

/// Тест типов в IR
#[test]
fn test_ir_type_information() {
    let source = r#"
        fn main() -> int {
            int x = 42;
            float y = 3.14;
            bool z = true;
            return x;
        }
    "#;

    let (_, ir_program) = compiler::compile_with_ir(source, vec![]);
    assert!(ir_program.is_some());

    let program = ir_program.unwrap();
    let main = program.get_function("main").unwrap();

    let mut has_int = false;
    let mut has_float = false;
    let mut has_bool = false;

    for (name, typ) in &main.locals {
        println!("DEBUG: local var {}: {}", name, typ);
        match typ.as_str() {
            "int" => has_int = true,
            "float" => has_float = true,
            "bool" => has_bool = true,
            _ => {}
        }
    }

    assert!(has_int, "Должна быть int переменная");
    assert!(has_float, "Должна быть float переменная");
    assert!(has_bool, "Должна быть bool переменная");
}
