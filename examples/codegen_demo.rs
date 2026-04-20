//! Демонстрация генерации кода

use minic::codegen::generate_assembly;
use minic::compiler::compile_with_ir;

fn main() {
    let source = r#"
        fn main() -> int {
            return 42;
        }
    "#;

    println!("=== Демонстрация кодогенерации ===\n");
    println!("Исходный код:\n{}\n", source);

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);

    if !parse_output.is_valid() {
        println!("Ошибки: {:?}", parse_output.errors);
        return;
    }

    let ir = ir_program.expect("IR не сгенерирован");
    let result = generate_assembly(&ir, false);

    println!("Сгенерированный ассемблер:\n{}", result.assembly);
    println!("Статистика:");
    println!("  Инструкций: {}", result.instruction_count);
    println!("  Размер фрейма: {} байт", result.frame_size);
}
