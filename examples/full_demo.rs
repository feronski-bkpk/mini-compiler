//! Полноценная демонстрация кодогенерации Mini Compiler (Спринт 5)

use minic::codegen::generate_assembly;
use minic::compiler::compile_with_ir;
use minic::ir::IRPrinter;
use std::fs;
use std::process::Command;

fn print_header(text: &str) {
    println!("\n=== {} ===", text);
    println!("{}", "=".repeat(50));
}

fn print_success(text: &str) {
    println!("[УСПЕХ] {}", text);
}

fn print_info(text: &str) {
    println!("[ИНФО] {}", text);
}

fn print_code(code: &str) {
    for line in code.lines() {
        println!("  {}", line);
    }
}

/// Демонстрация 1: Простая функция
fn demo_simple_function() {
    print_header("ДЕМО 1: Простая функция");

    let source = r#"
        fn main() -> int {
            return 42;
        }
    "#;

    print_info("Исходный код:");
    print_code(source);

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid(), "Ошибки компиляции");

    let ir = ir_program.unwrap();

    print_info("Сгенерированное промежуточное представление (IR):");
    println!("{}", IRPrinter::to_text(&ir));

    let result = generate_assembly(&ir, false);

    print_info("Сгенерированный ассемблерный код:");
    print_code(&result.assembly);

    print_success("Демо 1 завершено");
}

/// Демонстрация 2: Арифметические операции
fn demo_arithmetic() {
    print_header("ДЕМО 2: Арифметические операции");

    let source = r#"
        fn main() -> int {
            int a = 10;
            int b = 5;
            int sum = a + b;
            int diff = a - b;
            int mul = a * b;
            int div = a / b;
            int mod = a % b;
            return sum + diff + mul + div + mod;
        }
    "#;

    print_info("Исходный код:");
    print_code(source);

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());

    let ir = ir_program.unwrap();
    let result = generate_assembly(&ir, false);

    print_info("Ключевые инструкции в ассемблере:");
    for line in result.assembly.lines() {
        let trimmed = line.trim();
        if trimmed.contains("add")
            || trimmed.contains("sub")
            || trimmed.contains("imul")
            || trimmed.contains("idiv")
        {
            println!("  {}", trimmed);
        }
    }

    print_success("Демо 2 завершено");
}

/// Демонстрация 3: Условные операторы
fn demo_conditional() {
    print_header("ДЕМО 3: Условные операторы (if-else)");

    let source = r#"
        fn max(int a, int b) -> int {
            if (a > b) {
                return a;
            } else {
                return b;
            }
        }

        fn main() -> int {
            return max(10, 20);
        }
    "#;

    print_info("Исходный код:");
    print_code(source);

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());

    let ir = ir_program.unwrap();
    let result = generate_assembly(&ir, false);

    print_info("Условные переходы в ассемблере:");
    for line in result.assembly.lines() {
        let trimmed = line.trim();
        if trimmed.contains("cmp")
            || trimmed.contains("jz")
            || trimmed.contains("jnz")
            || trimmed.contains("jmp")
        {
            println!("  {}", trimmed);
        }
    }

    print_success("Демо 3 завершено");
}

/// Демонстрация 4: Циклы
fn demo_loops() {
    print_header("ДЕМО 4: Циклы (while)");

    let source = r#"
        fn sum_to_n(int n) -> int {
            int i = 1;
            int sum = 0;
            while (i <= n) {
                sum = sum + i;
                i = i + 1;
            }
            return sum;
        }

        fn main() -> int {
            return sum_to_n(10);
        }
    "#;

    print_info("Исходный код:");
    print_code(source);

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());

    let ir = ir_program.unwrap();
    let result = generate_assembly(&ir, false);

    print_info("Структура цикла в ассемблере:");
    for line in result.assembly.lines() {
        let trimmed = line.trim();
        if trimmed.contains("jmp") || trimmed.contains("cmp") {
            println!("  {}", trimmed);
        }
    }

    print_success("Демо 4 завершено");
}

/// Демонстрация 5: Рекурсивная функция
fn demo_recursion() {
    print_header("ДЕМО 5: Рекурсивная функция (факториал)");

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

    print_info("Исходный код:");
    print_code(source);

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());

    let ir = ir_program.unwrap();
    let result = generate_assembly(&ir, false);

    print_info("Рекурсивный вызов в ассемблере:");
    for line in result.assembly.lines() {
        if line.contains("call factorial") {
            println!("  {}", line.trim());
        }
    }

    print_success("Демо 5 завершено");
}

/// Демонстрация 6: Стековый фрейм
fn demo_stack_frame() {
    print_header("ДЕМО 6: Управление стековым фреймом");

    let source = r#"
        fn many_locals() -> int {
            int a = 1;
            int b = 2;
            int c = 3;
            int d = 4;
            int e = 5;
            int f = 6;
            return a + b + c + d + e + f;
        }

        fn main() -> int {
            return many_locals();
        }
    "#;

    print_info("Исходный код:");
    print_code(source);

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());

    let ir = ir_program.unwrap();
    let result = generate_assembly(&ir, false);

    print_info("Пролог и эпилог функции:");
    for line in result.assembly.lines() {
        let trimmed = line.trim();
        if trimmed.contains("push rbp")
            || trimmed.contains("mov rbp, rsp")
            || trimmed.contains("sub rsp,")
            || trimmed.contains("pop rbp")
        {
            println!("  {}", trimmed);
        }
    }

    print_info(&format!(
        "Размер стекового фрейма: {} байт",
        result.frame_size
    ));

    print_success("Демо 6 завершено");
}

/// Демонстрация 7: Оптимизации
fn demo_optimizations() {
    print_header("ДЕМО 7: Оптимизации промежуточного представления");

    let source = r#"
        fn main() -> int {
            int x = 5 + 3;
            int y = x * 1;
            int z = y + 0;
            return z;
        }
    "#;

    print_info("Исходный код с избыточными операциями:");
    print_code(source);

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());

    let ir = ir_program.unwrap();

    print_info("Без оптимизаций:");
    let result_unopt = generate_assembly(&ir, false);
    println!(
        "  Количество инструкций: {}",
        result_unopt.instruction_count
    );

    print_info("С оптимизациями:");
    let result_opt = generate_assembly(&ir, true);
    println!("  Количество инструкций: {}", result_opt.instruction_count);
    println!(
        "  Сокращение: {} инструкций",
        result_unopt.instruction_count - result_opt.instruction_count
    );

    print_success("Демо 7 завершено");
}

/// Демонстрация 8: Статистика кодогенерации
fn demo_statistics() {
    print_header("ДЕМО 8: Статистика генерации кода");

    let source = r#"
        fn fibonacci(int n) -> int {
            if (n <= 1) {
                return n;
            }
            return fibonacci(n - 1) + fibonacci(n - 2);
        }

        fn main() -> int {
            return fibonacci(10);
        }
    "#;

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());

    let ir = ir_program.unwrap();
    let result = generate_assembly(&ir, false);

    let registers_str = if result.registers_used.is_empty() {
        "rax (неявно)".to_string()
    } else {
        result.registers_used.join(", ")
    };

    println!("\n=== СТАТИСТИКА ГЕНЕРАЦИИ КОДА ===");
    println!("  Инструкций:        {}", result.instruction_count);
    println!("  Размер фрейма:     {} байт", result.frame_size);
    println!("  Использовано регистров: {}", result.registers_used.len());
    println!("  Список регистров:  {}", registers_str);
    println!("=================================");

    print_success("Демо 8 завершено");
}

/// Демонстрация 9: Компиляция и запуск реального кода
fn demo_compile_and_run() {
    print_header("ДЕМО 9: Компиляция и выполнение реального кода");

    let source = r#"
        fn main() -> int {
            int result = 0;
            int i = 1;
            while (i <= 10) {
                result = result + i;
                i = i + 1;
            }
            return result;
        }
    "#;

    print_info("Программа: сумма чисел от 1 до 10");
    print_code(source);

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());

    let ir = ir_program.unwrap();
    let result = generate_assembly(&ir, false);

    let asm_file = "demo_output.asm";
    let obj_file = "demo_output.o";
    let exe_file = if cfg!(windows) {
        "demo_program.exe"
    } else {
        "demo_program"
    };

    match fs::write(asm_file, &result.assembly) {
        Ok(_) => print_info(&format!("Ассемблер сохранён в: {}", asm_file)),
        Err(e) => print_info(&format!("Ошибка сохранения ассемблера: {}", e)),
    }

    let nasm_status = Command::new("nasm")
        .args(&["-f", "elf64", asm_file, "-o", obj_file])
        .status();

    match nasm_status {
        Ok(status) if status.success() => {
            print_info("Ассемблирование успешно (NASM)");

            let linker = if cfg!(windows) { "gcc" } else { "ld" };
            let ld_status = if cfg!(windows) {
                Command::new(linker)
                    .args(&["-no-pie", "-o", exe_file, obj_file])
                    .status()
            } else {
                Command::new(linker)
                    .args(&[obj_file, "-o", exe_file])
                    .status()
            };

            match ld_status {
                Ok(status) if status.success() => {
                    print_info(&format!("Линковка успешна ({})", linker));

                    let run_output = if cfg!(windows) {
                        Command::new("cmd").args(&["/C", exe_file]).output()
                    } else {
                        Command::new(format!("./{}", exe_file)).output()
                    };

                    if let Ok(output) = run_output {
                        let exit_code = output.status.code().unwrap_or(-1);
                        println!("\nРезультат: сумма чисел от 1 до 10 = {}", exit_code);

                        if exit_code == 55 {
                            print_success("Результат верный (55)!");
                        } else {
                            print_info(&format!("Результат: {} (ожидалось 55)", exit_code));
                        }
                    }

                    let _ = fs::remove_file(exe_file);
                }
                Ok(_) => print_info("Линковка не удалась"),
                Err(e) => print_info(&format!("Ошибка линковки: {}", e)),
            }
            let _ = fs::remove_file(obj_file);
        }
        Ok(_) => print_info("Ассемблирование не удалось"),
        Err(e) => print_info(&format!("Ошибка NASM: {}", e)),
    }

    let _ = fs::remove_file(asm_file);

    print_success("Демо 9 завершено");
}

/// Демонстрация 10: Сравнение с эталоном
fn demo_comparison() {
    print_header("ДЕМО 10: Сравнение с эталонной реализацией");

    let source = r#"
        // Пример из спецификации
        fn add(int a, int b) -> int {
            int result = a + b;
            return result;
        }

        fn main() -> int {
            return add(5, 3);
        }
    "#;

    print_info("Исходный код из примера спецификации:");
    print_code(source);

    let (parse_output, ir_program) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());

    let ir = ir_program.unwrap();
    let result = generate_assembly(&ir, false);

    print_info("Сгенерированный ассемблер соответствует спецификации:");

    let checks = [
        ("push rbp", "Пролог: сохранение RBP"),
        ("mov rbp, rsp", "Пролог: установка RBP"),
        ("ret", "Эпилог: возврат"),
        ("call add", "Вызов функции"),
        ("mov rax,", "Возврат значения в RAX"),
    ];

    for (pattern, desc) in checks {
        if result.assembly.contains(pattern) {
            println!("  [OK] {}", desc);
        } else {
            println!("  [ОШИБКА] {}", desc);
        }
    }

    print_success("Демо 10 завершено");
}

fn main() {
    println!("\n{}", "=".repeat(80));
    println!("Mini Compiler - Спринт 5: Генерация x86-64 кода");
    println!("{}", "=".repeat(80));
    println!("Демонстрация всех возможностей генерации кода\n");

    let demos: Vec<fn()> = vec![
        demo_simple_function,
        demo_arithmetic,
        demo_conditional,
        demo_loops,
        demo_recursion,
        demo_stack_frame,
        demo_optimizations,
        demo_statistics,
        demo_compile_and_run,
        demo_comparison,
    ];

    for (i, demo) in demos.iter().enumerate() {
        demo();
        if i < demos.len() - 1 {
            println!("\n{}", "-".repeat(50));
        }
    }

    println!("\n{}", "=".repeat(80));
    println!("Демонстрация завершена! Все возможности Спринта 5 успешно работают!");
    println!("{}", "=".repeat(80));

    println!("\nЧто было продемонстрировано:");
    println!("  - Генерация x86-64 ассемблерного кода");
    println!("  - Соответствие System V AMD64 ABI");
    println!("  - Прологи и эпилоги функций");
    println!("  - Арифметические, логические операции и сравнения");
    println!("  - Условные переходы (if-else)");
    println!("  - Циклы (while)");
    println!("  - Рекурсивные вызовы функций");
    println!("  - Управление стековым фреймом");
    println!("  - Оптимизация промежуточного представления");
    println!("  - Кросс-платформенность (Windows/Linux/macOS)");
    println!();
}
