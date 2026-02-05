//! Ручное тестирование всех компонентов Mini Compiler
//!
//! Этот скрипт проверяет работу каждого модуля системы.

use minic::{
    common::{Position, Token, TokenKind},
    lexer::Scanner,
    utils,
};

fn main() {
    println!("====================================\n");

    test_positions();

    test_tokens();

    test_scanner_basic();

    test_scanner_advanced();

    test_utils();

    test_full_pipeline();
}

fn test_positions() {
    println!("1. ТЕСТ ПОЗИЦИЙ");
    println!("----------------");

    let mut pos = Position::start();
    println!("  Стартовая позиция: {}", pos);

    pos.advance_column(5);
    println!("  После advance_column(5): {}", pos);

    pos.new_line();
    println!("  После new_line(): {}", pos);

    let pos2 = pos.with_column_offset(10);
    println!("  with_column_offset(10): {}", pos2);

    println!("  Форматирование Display: {}", pos);
    println!("  Форматирование debug: {}", pos.debug());

    println!();
}

fn test_tokens() {
    println!("2. ТЕСТ ТОКЕНОВ");
    println!("---------------");

    let pos = Position::new(10, 20);

    let kw_token = Token::new(TokenKind::KwIf, "if".to_string(), pos);
    println!("  Ключевое слово: {}", kw_token);
    println!(
        "    is_keyword: {}, is_literal: {}, is_operator: {}",
        kw_token.is_keyword(),
        kw_token.is_literal(),
        kw_token.is_operator()
    );

    let int_token = Token::new(TokenKind::IntLiteral(42), "42".to_string(), pos);
    println!("  Числовой литерал: {}", int_token);
    println!("    as_int: {:?}", int_token.as_int());

    let str_token = Token::new(
        TokenKind::StringLiteral("hello".to_string()),
        "\"hello\"".to_string(),
        pos,
    );
    println!("  Строковый литерал: {}", str_token);
    println!("    as_string: {:?}", str_token.as_string());

    let op_token = Token::new(TokenKind::Plus, "+".to_string(), pos);
    println!("  Оператор: {}", op_token);
    println!("    is_operator: {}", op_token.is_operator());

    let eof_token = Token::eof(pos);
    println!("  EOF: {}", eof_token);
    println!("    is_eof: {}", eof_token.is_eof());

    println!();
}

fn test_scanner_basic() {
    println!("3. ТЕСТ СКАНЕРА (БАЗОВЫЙ)");
    println!("-------------------------");

    let source = "x = 42;";
    println!("  Тест: '{}'", source);

    let mut scanner = Scanner::new(source);
    let (tokens, errors) = scanner.scan_all();

    if !errors.is_empty() {
        println!("Ошибки: {:?}", errors);
    } else {
        println!("Успешно! Токенов: {}", tokens.len());
        for (i, token) in tokens.iter().enumerate() {
            if !token.is_eof() {
                println!("    {:2}: {}", i + 1, token);
            }
        }
    }

    println!("\n  Проверка позиционирования:");
    let mut scanner2 = Scanner::new("a\n  b\n   c");
    while let Ok(token) = scanner2.next_token() {
        if token.is_eof() {
            break;
        }
        println!("    {}", token);
    }

    println!();
}

fn test_scanner_advanced() {
    println!("4. ТЕСТ СКАНЕРА (СЛОЖНЫЕ СЛУЧАИ)");
    println!("--------------------------------");

    let sources = vec![
        ("// комментарий\nx = 1;", "Однострочный комментарий"),
        (
            "/* многострочный\nкомментарий */ x = 2;",
            "Многострочный комментарий",
        ),
        ("x = 3; // комментарий после кода", "Комментарий после кода"),
        ("x = /* встроенный */ 4;", "Встроенный комментарий"),
    ];

    for (source, description) in sources {
        println!("  {}:", description);
        println!("    Исходник: {:?}", source);

        let mut scanner = Scanner::new(source);
        let (tokens, errors) = scanner.scan_all();

        if !errors.is_empty() {
            println!("Ошибки: {}", errors.len());
        } else {
            let valid_tokens = tokens.iter().filter(|t| !t.is_eof()).count();
            println!("Токенов: {} (без EOF)", valid_tokens);
        }
    }

    println!("\n  Строки и escape-последовательности:");
    let string_tests = vec![
        r#""простая строка""#,
        r#""строка с \"кавычками\"""#,
        r#""строка с \n переводом строки""#,
        r#""строка с \\ обратным слешем""#,
    ];

    for test in string_tests {
        let mut scanner = Scanner::new(test);
        let (_tokens, errors) = scanner.scan_all();

        println!(
            "    {:?} -> {}",
            test,
            if errors.is_empty() { "ok" } else { "bad" }
        );
    }

    println!("\n  Восстановление после ошибок:");
    let error_source = "x = @; y = 42; z = #;";
    println!("    Исходник с ошибками: {:?}", error_source);

    let mut scanner = Scanner::new(error_source);
    let (tokens, errors) = scanner.scan_all();

    println!("    Найдено ошибок: {}", errors.len());
    println!("    Найдено токенов: {}", tokens.len());

    let has_y = tokens.iter().any(|t| t.lexeme == "y");
    let has_42 = tokens
        .iter()
        .any(|t| matches!(t.kind, TokenKind::IntLiteral(42)));

    println!(
        "    Восстановление: {}",
        if has_y && has_42 {
            "ok - (найдены y и 42)"
        } else {
            "bad"
        }
    );

    println!();
}

fn test_utils() {
    println!("5. ТЕСТ УТИЛИТ");
    println!("-------------");

    println!("  Проверка is_keyword:");
    let words = vec!["if", "fn", "variable", "IF"];
    for word in words {
        println!("    '{}' -> {}", word, utils::is_keyword(word));
    }

    println!("\n  Проверка is_valid_identifier:");
    let ids = vec!["x", "_x", "x1", "1x", "x_y", "very_long_identifier"];
    for id in ids {
        println!("    '{}' -> {}", id, utils::is_valid_identifier(id));
    }

    println!("\n  Проверка escape_string:");
    let strings = vec!["hello\nworld", "tab\there", "\"quoted\"", "back\\slash"];
    for s in strings {
        println!("    {:?} -> {:?}", s, utils::escape_string(s));
    }

    println!();
}

fn test_full_pipeline() {
    println!("6. ПОЛНЫЙ ПАЙПЛАЙН");
    println!("------------------");

    let program = r#"
// Простая программа на MiniC
fn main() {
    int counter = 0;
    float pi = 3.14159;
    bool running = true;

    while (counter < 10 && running) {
        counter = counter + 1;
        pi = pi * 1.01;

        if (counter >= 5) {
            running = false;
        }
    }

    return counter;
}
"#;

    println!("  Тестируем полную программу:");
    println!("  Длина программы: {} символов", program.len());

    let mut scanner = Scanner::new(program);
    let (tokens, errors) = scanner.scan_all();

    println!("  Результат:");
    println!("    - Ошибок: {}", errors.len());
    println!("    - Токенов: {} (включая EOF)", tokens.len());

    if !errors.is_empty() {
        println!("Первая ошибка: {}", errors[0]);
    } else {
        println!("Лексический анализ успешен!");

        let mut stats = std::collections::HashMap::new();
        for token in &tokens {
            if !token.is_eof() {
                let type_name = token.type_name();
                *stats.entry(type_name).or_insert(0) += 1;
            }
        }

        println!("\n    Статистика токенов:");
        let mut stats_vec: Vec<_> = stats.iter().collect();
        stats_vec.sort_by_key(|&(name, _)| *name);

        for (type_name, count) in stats_vec {
            println!("      {:20}: {}", type_name, count);
        }
    }

    println!();
}
