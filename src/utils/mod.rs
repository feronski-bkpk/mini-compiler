//! Вспомогательные утилиты для компилятора MiniC.
//!
//! Содержит полезные функции для работы с файлами, строками,
//! валидации и других общих задач.

use std::fs;
use std::io;
use std::path::Path;

/// Читает содержимое файла в строку с проверкой размера.
///
/// # Аргументы
///
/// * `path` - путь к файлу
///
/// # Возвращает
///
/// Содержимое файла или ошибку ввода-вывода.
///
/// # Ограничения
///
/// - Максимальный размер файла: 1MB
/// - Кодировка: UTF-8
///
/// # Пример
///
/// ```
/// use minic::utils::read_file_with_limit;
/// use std::path::Path;
///
/// let temp_file = std::env::temp_dir().join("test_file.txt");
/// std::fs::write(&temp_file, "test content").unwrap();
///
/// let content = read_file_with_limit(&temp_file).unwrap();
/// assert_eq!(content, "test content");
///
/// std::fs::remove_file(temp_file).unwrap();
/// ```
pub fn read_file_with_limit(path: &Path) -> io::Result<String> {
    let content = fs::read_to_string(path)?;

    const MAX_SIZE: usize = 1024 * 1024;
    if content.len() > MAX_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Файл слишком большой ({} > {} байт)",
                content.len(),
                MAX_SIZE
            ),
        ));
    }

    Ok(content)
}

/// Записывает строку в файл.
///
/// # Аргументы
///
/// * `path` - путь к файлу
/// * `content` - содержимое для записи
///
/// # Пример
///
/// ```
/// use minic::utils::write_file;
/// use std::path::Path;
///
/// write_file(Path::new("output.txt"), "Hello, world!").unwrap();
/// ```
pub fn write_file(path: &Path, content: &str) -> io::Result<()> {
    fs::write(path, content)
}

/// Читает исходный код из стандартного ввода.
///
/// # Возвращает
///
/// Введенный текст или ошибку ввода-вывода.
///
pub fn read_stdin() -> io::Result<String> {
    use io::Read;

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

/// Проверяет, является ли строка ключевым словом языка MiniC.
///
/// # Аргументы
///
/// * `s` - проверяемая строка
///
/// # Возвращает
///
/// `true`, если строка является ключевым словом.
///
/// # Пример
///
/// ```
/// use minic::utils::is_keyword;
///
/// assert!(is_keyword("if"));
/// assert!(is_keyword("fn"));
/// assert!(!is_keyword("variable"));
/// ```
pub fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "if" | "else"
            | "while"
            | "for"
            | "int"
            | "float"
            | "bool"
            | "return"
            | "true"
            | "false"
            | "void"
            | "struct"
            | "fn"
    )
}

/// Проверяет, является ли строка допустимым идентификатором.
///
/// Согласно спецификации языка MiniC:
/// 1. Начинается с буквы или подчеркивания
/// 2. Содержит только буквы, цифры и подчеркивания
/// 3. Не является ключевым словом
/// 4. Длина ≤ 255 символов
///
/// # Аргументы
///
/// * `s` - проверяемая строка
///
/// # Возвращает
///
/// `true`, если строка является допустимым идентификатором.
///
/// # Пример
///
/// ```
/// use minic::utils::is_valid_identifier;
///
/// assert!(is_valid_identifier("variable"));
/// assert!(is_valid_identifier("_private"));
/// assert!(!is_valid_identifier("123var"));
/// assert!(!is_valid_identifier("if"));
/// ```
pub fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() || s.len() > 255 {
        return false;
    }

    let first_char = s.chars().next().unwrap();
    if !(first_char.is_ascii_alphabetic() || first_char == '_') {
        return false;
    }

    for c in s.chars() {
        if !(c.is_ascii_alphanumeric() || c == '_') {
            return false;
        }
    }

    !is_keyword(s)
}

/// Экранирует специальные символы в строке для безопасного вывода.
///
/// # Аргументы
///
/// * `s` - исходная строка
///
/// # Возвращает
///
/// Экранированную строку.
///
/// # Пример
///
/// ```
/// use minic::utils::escape_string;
///
/// assert_eq!(escape_string("hello\nworld"), "hello\\nworld");
/// assert_eq!(escape_string("\"quoted\""), "\\\"quoted\\\"");
/// ```
pub fn escape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());

    for c in s.chars() {
        match c {
            '\n' => result.push_str("\\n"),
            '\t' => result.push_str("\\t"),
            '\r' => result.push_str("\\r"),
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\'' => result.push_str("\\'"),
            _ => result.push(c),
        }
    }

    result
}

/// Форматирует вектор ошибок для вывода.
///
/// # Аргументы
///
/// * `errors` - вектор ошибок
///
/// # Возвращает
///
/// Отформатированную строку с ошибками.
pub fn format_errors(errors: &[crate::lexer::LexerError]) -> String {
    let mut result = String::new();

    if errors.is_empty() {
        result.push_str("Ошибок не обнаружено.\n");
    } else {
        result.push_str(&format!("Найдено {} ошибок:\n", errors.len()));
        for (i, error) in errors.iter().enumerate() {
            result.push_str(&format!("  {}. {}\n", i + 1, error));
        }
    }

    result
}

/// Форматирует вектор токенов для отладки.
///
/// # Аргументы
///
/// * `tokens` - вектор токенов
///
/// # Возвращает
///
/// Отформатированную строку с токенами.
pub fn format_tokens(tokens: &[crate::common::Token]) -> String {
    let mut result = String::new();

    if tokens.is_empty() {
        result.push_str("Токены не найдены.\n");
    } else {
        result.push_str(&format!("Найдено {} токенов:\n", tokens.len()));
        for (i, token) in tokens.iter().enumerate() {
            if !token.is_eof() {
                result.push_str(&format!("  {:3}: {}\n", i + 1, token));
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_keyword() {
        assert!(is_keyword("if"));
        assert!(is_keyword("fn"));
        assert!(is_keyword("return"));
        assert!(!is_keyword("variable"));
        assert!(!is_keyword("IF"));
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("x"));
        assert!(is_valid_identifier("_x"));
        assert!(is_valid_identifier("x1"));
        assert!(is_valid_identifier("my_var"));
        assert!(is_valid_identifier("MAX_VALUE"));

        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("1x"));
        assert!(!is_valid_identifier("x-y"));
        assert!(!is_valid_identifier("x.y"));
        assert!(!is_valid_identifier("if"));

        let long_identifier = "a".repeat(256);
        assert!(!is_valid_identifier(&long_identifier));

        let max_length_identifier = "a".repeat(255);
        assert!(is_valid_identifier(&max_length_identifier));
    }

    #[test]
    fn test_escape_string() {
        assert_eq!(escape_string("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_string("tab\there"), "tab\\there");
        assert_eq!(escape_string("\"quoted\""), "\\\"quoted\\\"");
        assert_eq!(escape_string("back\\slash"), "back\\\\slash");
        assert_eq!(escape_string("normal"), "normal");
    }

    #[test]
    fn test_format_functions() {
        let errors = vec![];
        assert!(format_errors(&errors).contains("Ошибок не обнаружено"));

        let tokens = vec![];
        assert!(format_tokens(&tokens).contains("Токены не найдены"));
    }
}
