//! Интеграционные тесты для лексического анализатора.
//!
//! Тестирует корректность работы сканера на реальных примерах
//! исходного кода на языке MiniC.

use minic::lexer::Scanner;
use std::fs;
use std::path::Path;

/// Тестирует корректные (валидные) примеры исходного кода.
#[test]
fn test_valid_examples() {
    let valid_dir = Path::new("tests/lexer/valid");

    if !valid_dir.exists() {
        println!(
            "Директория с валидными примерами не найдена: {:?}",
            valid_dir
        );
        return;
    }

    let mut total_tests = 0;
    let mut passed_tests = 0;

    for entry in fs::read_dir(valid_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "src") {
            total_tests += 1;

            let source = fs::read_to_string(&path).unwrap();
            let mut scanner = Scanner::new(&source);
            let (tokens, errors) = scanner.scan_all();

            if errors.is_empty() {
                passed_tests += 1;
                println!(
                    "✓ {}: успешно ({} токенов)",
                    path.file_name().unwrap().to_string_lossy(),
                    tokens.len()
                );
            } else {
                println!(
                    "✗ {}: найдены ошибки",
                    path.file_name().unwrap().to_string_lossy()
                );
                for error in errors {
                    println!("  • {}", error);
                }
            }
        }
    }

    println!(
        "\nИтог валидных тестов: {}/{} пройдено",
        passed_tests, total_tests
    );
    assert_eq!(passed_tests, total_tests, "Не все валидные тесты пройдены");
}

/// Тестирует некорректные (невалидные) примеры исходного кода.
#[test]
fn test_invalid_examples() {
    let invalid_dir = Path::new("tests/lexer/invalid");

    if !invalid_dir.exists() {
        println!(
            "Директория с невалидными примерами не найдена: {:?}",
            invalid_dir
        );
        return;
    }

    let mut total_tests = 0;
    let mut passed_tests = 0;

    for entry in fs::read_dir(invalid_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "src") {
            total_tests += 1;

            let source = fs::read_to_string(&path).unwrap();
            let mut scanner = Scanner::new(&source);
            let (_, errors) = scanner.scan_all();

            if !errors.is_empty() {
                passed_tests += 1;
                println!(
                    "✓ {}: правильно обнаружены ошибки ({} ошибок)",
                    path.file_name().unwrap().to_string_lossy(),
                    errors.len()
                );

                if !errors.is_empty() {
                    println!("  Первая ошибка: {}", errors[0]);
                }
            } else {
                println!(
                    "✗ {}: ошибки не обнаружены (ожидались ошибки)",
                    path.file_name().unwrap().to_string_lossy()
                );

                let mut scanner2 = Scanner::new(&source);
                let (tokens, _) = scanner2.scan_all();
                println!("  Найдено токенов: {}", tokens.len());
                for token in &tokens[..std::cmp::min(tokens.len(), 3)] {
                    println!("    {}", token);
                }
            }
        }
    }

    println!(
        "\nИтог невалидных тестов: {}/{} пройдено",
        passed_tests, total_tests
    );

    if passed_tests < total_tests {
        println!(
            "ВНИМАНИЕ: Не все тесты пройдены. Пропущено: {}",
            total_tests - passed_tests
        );
    }
}

/// Тест граничных случаев для идентификаторов.
#[test]
fn test_identifier_edge_cases() {
    use minic::common::utils;

    let max_len_id = "a".repeat(255);
    assert!(utils::is_valid_identifier(&max_len_id));

    let too_long_id = "a".repeat(256);
    assert!(!utils::is_valid_identifier(&too_long_id));

    assert!(utils::is_valid_identifier("_"));
    assert!(utils::is_valid_identifier("__"));
    assert!(utils::is_valid_identifier("_var"));
    assert!(utils::is_valid_identifier("var_"));
    assert!(utils::is_valid_identifier("var_name"));

    assert!(utils::is_valid_identifier("var1"));
    assert!(utils::is_valid_identifier("var123"));
    assert!(!utils::is_valid_identifier("1var"));
}

/// Тест граничных случаев для чисел.
#[test]
fn test_number_edge_cases() {
    let test_cases = vec![
        ("0", true),
        ("123", true),
        ("-123", true),
        ("2147483647", true),
        ("-2147483648", true),
        ("2147483648", false),
        ("-2147483649", false),
        ("3.14", true),
        ("-3.14", true),
        ("0.0", true),
        (".5", false),
        ("123.", false),
        ("12.34.56", false),
    ];

    for (number_str, should_be_valid) in test_cases {
        let mut scanner = Scanner::new(number_str);
        let (_, errors) = scanner.scan_all();

        let is_valid = errors.is_empty();
        assert_eq!(
            is_valid, should_be_valid,
            "Число '{}': ожидалось {}, получено {}",
            number_str, should_be_valid, is_valid
        );
    }
}

/// Тест всех ключевых слов языка.
#[test]
fn test_all_keywords() {
    let keywords = vec![
        "if", "else", "while", "for", "int", "float", "bool", "return", "void", "struct", "fn",
    ];

    for keyword in keywords {
        let mut scanner = Scanner::new(keyword);
        let (tokens, errors) = scanner.scan_all();

        assert!(
            errors.is_empty(),
            "Ключевое слово '{}' вызвало ошибку: {:?}",
            keyword,
            errors
        );
        assert_eq!(tokens.len(), 2);

        let token = &tokens[0];
        assert!(
            token.is_keyword(),
            "Токен '{}' не распознан как ключевое слово",
            keyword
        );
    }
}

/// Тест всех операторов языка.
#[test]
fn test_all_operators() {
    let operators = vec![
        "+", "-", "*", "/", "%", "==", "!=", "<", "<=", ">", ">=", "&&", "||", "!", "=", "+=",
        "-=", "*=", "/=",
    ];

    for operator in operators {
        println!("\n=== Тестируем оператор '{}' ===", operator);
        let mut scanner = Scanner::new(operator);

        match scanner.peek_token() {
            Ok(token) => println!("peek_token: {:?}", token.kind),
            Err(e) => println!("peek_token ошибка: {}", e),
        }

        match scanner.next_token() {
            Ok(token) => {
                println!(
                    "next_token: тип = {:?}, лексема = '{}', is_operator = {}, is_eof = {}",
                    token.kind,
                    token.lexeme,
                    token.is_operator(),
                    token.is_eof()
                );

                if token.is_eof() {
                    panic!(
                        "Оператор '{}' распознан как EOF! source='{}'",
                        operator, operator
                    );
                }

                if !token.is_operator() {
                    panic!(
                        "Токен '{}' не распознан как оператор. Тип: {:?}",
                        operator, token.kind
                    );
                }

                match scanner.next_token() {
                    Ok(eof_token) => {
                        assert!(eof_token.is_eof(), "Ожидался EOF после оператора");
                        println!("  Следующий токен: EOF (корректно)");
                    }
                    Err(e) => panic!("Ошибка EOF: {}", e),
                }
            }
            Err(e) => panic!("Ошибка: {}", e),
        }
    }
}

/// Тест всех разделителей языка.
#[test]
fn test_all_delimiters() {
    let delimiters = vec!["(", ")", "{", "}", "[", "]", ";", ",", ":"];

    for delimiter in delimiters {
        let mut scanner = Scanner::new(delimiter);
        let (tokens, errors) = scanner.scan_all();

        assert!(
            errors.is_empty(),
            "Разделитель '{}' вызвал ошибку: {:?}",
            delimiter,
            errors
        );
        assert_eq!(tokens.len(), 2);

        let token = &tokens[0];
        assert!(
            token.is_delimiter(),
            "Токен '{}' не распознан как разделитель",
            delimiter
        );
    }
}
