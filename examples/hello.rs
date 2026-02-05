//! Пример использования Mini Compiler как библиотеки

use minic::lexer::Scanner;

fn main() {
    let source_code = r#"
fn main() {
    int x = 42;
    float pi = 3.14;
    return x;
}
"#;

    let mut scanner = Scanner::new(source_code);
    let (tokens, errors) = scanner.scan_all();

    if errors.is_empty() {
        println!("Лексический анализ успешен!");
        println!("Найдено {} токенов:", tokens.len());

        for token in tokens.iter().filter(|t| !t.is_eof()) {
            println!("  {}", token);
        }
    } else {
        println!("Найдены ошибки:");
        for error in errors {
            println!("  {}", error);
        }
    }
}
