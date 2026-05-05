//! # Sprint 6 Demo
//!
//! Демонстрация всех новых возможностей Спринта 6:
//! - Условные операторы (if-else + вложенные)
//! - Циклы (while, for)
//! - Короткая схема вычислений (&&, ||)
//! - Float и приведение типов
//! - Break и Continue
//! - Switch-case-default
//!
//! Запуск: cargo run --example sprint6_demo

use minic::codegen::generate_assembly;
use minic::compiler::compile_with_ir;
use minic::ir::IRPrinter;

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║     Mini Compiler - Sprint 6: Демонстрация               ║");
    println!("║     Control Flow & Short-Circuit Evaluation              ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    demo_if_else();
    demo_while_loop();
    demo_for_loop();
    demo_short_circuit();
    demo_float_conversion();
    demo_break_continue();
    demo_switch();
    demo_nested_control_flow();

    println!("\nВсе демонстрации завершены успешно!");
}

fn compile_and_show(source: &str, title: &str) {
    println!("━━━ {} ━━━", title);
    println!("Исходный код:\n{}\n", source);

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);

    if !parse_output.is_valid() {
        eprintln!("Ошибки компиляции:");
        for error in &parse_output.errors.errors {
            eprintln!("  {}", error);
        }
        return;
    }

    if let Some(ir) = ir_program {
        println!("IR:");
        println!("{}", IRPrinter::to_text(&ir));

        let result = generate_assembly(&ir, false);
        println!("x86-64 Ассемблер:");
        println!("{}\n", result.assembly);
    }
}

fn demo_if_else() {
    let source = r#"fn main() -> int {
    int x = 10;
    if (x > 5) {
        return 1;
    } else {
        return 0;
    }
}"#;
    compile_and_show(source, "Демонстрация 1: If-Else");
}

fn demo_while_loop() {
    let source = r#"fn main() -> int {
    int sum = 0;
    int i = 0;
    while (i < 10) {
        sum = sum + i;
        i = i + 1;
    }
    return sum;
}"#;
    compile_and_show(source, "Демонстрация 2: While Loop");
}

fn demo_for_loop() {
    let source = r#"fn main() -> int {
    int result = 1;
    int i;
    for (i = 2; i <= 5; i = i + 1) {
        result = result * i;
    }
    return result;
}"#;
    compile_and_show(source, "Демонстрация 3: For Loop (счетный)");
}

fn demo_short_circuit() {
    let source = r#"fn main() -> int {
    int a = 0;
    int b = 5;
    // Короткая схема предотвращает деление на ноль
    if (a != 0 && b / a > 2) {
        return 1;
    }
    return 0;
}"#;
    compile_and_show(source, "Демонстрация 4: Short-Circuit AND");
}

fn demo_float_conversion() {
    let source = r#"fn main() -> int {
    int x = 5;
    float y = 3.14;
    // int автоматически приводится к float
    float z = x + y;
    return 15;
}"#;
    compile_and_show(source, "Демонстрация 5: Float + Int приведение");
}

fn demo_break_continue() {
    let source = r#"fn main() -> int {
    int i = 0;
    int sum = 0;
    while (true) {
        i = i + 1;
        if (i > 10) {
            break;           // выход из цикла
        }
        if (i % 2 == 0) {
            continue;        // пропуск четных
        }
        sum = sum + i;       // сумма нечетных
    }
    return sum;              // 1+3+5+7+9 = 25
}"#;
    compile_and_show(source, "Демонстрация 6: Break и Continue");
}

fn demo_switch() {
    let source = r#"fn main() -> int {
    int x = 2;
    int result = 0;
    switch (x) {
        case 1:
            result = 10;
        case 2:
            result = 20;
        case 3:
            result = 30;
        default:
            result = 0;
    }
    return result;
}"#;
    compile_and_show(source, "Демонстрация 7: Switch-Case-Default");
}

fn demo_nested_control_flow() {
    let source = r#"fn main() -> int {
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
}"#;
    compile_and_show(source, "Демонстрация 8: Смешанный поток управления");
}
