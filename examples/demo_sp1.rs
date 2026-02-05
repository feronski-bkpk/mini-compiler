//! Демонстрация всех возможностей Mini Compiler (Спринт 1)

use minic::{compiler, preprocessor::Preprocessor};

fn main() {
    println!("=== ДЕМОНСТРАЦИЯ MINI COMPILER (СПРИНТ 1) ===\n");

    demo_lexical_analysis();

    demo_preprocessor();
}

fn demo_lexical_analysis() {
    println!("2. ЛЕКСИЧЕСКИЙ АНАЛИЗ:");

    let source1 = r#"fn main() {
    int x = 42;
    return x + 10;
}"#;

    println!("   Пример 1: Простая программа");
    println!("   Исходный код:");
    println!("   ```");
    println!("{}", source1);
    println!("   ```");

    let (tokens1, errors1) = compiler::lexical_analysis(source1);
    println!(
        "   Результат ({} токенов, {} ошибок):",
        tokens1.len(),
        errors1.len()
    );

    for token in &tokens1 {
        if !token.is_eof() {
            println!("     {}", token);
        }
    }
    println!();

    let source2 = r#"// Демонстрация всех типов токенов
if (x > 0 && y <= 10.5) {
    result = (a + b) * c / 2;
    string msg = "Hello, World!";
    bool flag = true;
}"#;

    println!("   Пример 2: Все типы токенов");
    let (tokens2, _errors2) = compiler::lexical_analysis(source2);
    println!("   Уникальные типы токенов:");

    use std::collections::HashSet;
    let mut token_types = HashSet::new();
    for token in &tokens2 {
        if !token.is_eof() {
            token_types.insert(token.type_name());
        }
    }

    for ttype in token_types {
        println!("     - {}", ttype);
    }
    println!();

    let source3 = "x = @invalid; y = 123abc;";
    println!("   Пример 3: Обработка ошибок");
    let (_tokens3, errors3) = compiler::lexical_analysis(source3);

    if !errors3.is_empty() {
        println!("   Обнаружены ошибки:");
        for error in &errors3 {
            println!("     • {}", error);
        }
    }
    println!("   Сканер восстановился и продолжил работу.");
    println!();
}

fn demo_preprocessor() {
    println!("3. ПРЕПРОЦЕССОР (растягивающая цель):");

    let source1 = r#"#define MAX 100
// Однострочный комментарий
int x = MAX; /* Встроенный комментарий */
/*
 * Многострочный
 * комментарий
 */
int y = 50;"#;

    println!("   Пример 1: Удаление комментариев и макросы");
    println!("   Исходный код:");
    println!("   ```");
    println!("{}", source1);
    println!("   ```");

    let mut preprocessor = Preprocessor::new(source1);
    match preprocessor.process() {
        Ok(result) => {
            println!("   Результат после препроцессора:");
            println!("   ```");
            println!("{}", result);
            println!("   ```");
        }
        Err(err) => {
            println!("   Ошибка препроцессора: {}", err);
        }
    }
    println!();

    let source2 = r#"#ifdef DEBUG
    log("Debug mode enabled");
    int debug_level = 3;
#else
    int debug_level = 0;
#endif

    int main() {
        return debug_level;
    }"#;

    println!("   Пример 2: Условная компиляция");
    println!("   С DEBUG:");
    let mut pp1 = Preprocessor::new(source2);
    pp1.define("DEBUG", "1").unwrap();
    println!(
        "   Результат: {}",
        pp1.process().unwrap().lines().nth(0).unwrap()
    );

    println!("   Без DEBUG:");
    let mut pp2 = Preprocessor::new(source2);
    println!(
        "   Результат: {}",
        pp2.process().unwrap().lines().nth(0).unwrap()
    );
    println!();

    let source3 = r#"#define A B
#define B A
int x = A;"#;

    println!("   Пример 3: Обнаружение рекурсии макросов");
    let mut pp3 = Preprocessor::new(source3);
    match pp3.process() {
        Ok(_) => println!("   Ошибка: рекурсия не обнаружена!"),
        Err(err) => println!("   ✓ Рекурсия обнаружена: {}", err),
    }
    println!();
}
