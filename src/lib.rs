//! Mini Compiler - компилятор упрощенного C-подобного языка.
//!
//! # Обзор
//!
//! Mini Compiler - это учебный проект, реализующий полный pipeline
//! компиляции для упрощенного C-подобного языка (MiniC).
//!
//! # Архитектура
//!
//! Проект разделен на следующие основные модули:
//!
//! 1. **common** - общие типы данных (токены, позиции)
//! 2. **lexer** - лексический анализатор (сканер)
//! 3. **utils** - вспомогательные утилиты
//!
//! # Пример использования
//!
//! ```
//! use minic::lexer::Scanner;
//!
//! let source = "fn main() { return 42; }";
//! let mut scanner = Scanner::new(source);
//! let (tokens, errors) = scanner.scan_all();
//!
//! if errors.is_empty() {
//!     for token in tokens {
//!         println!("{}", token);
//!     }
//! } else {
//!     for error in errors {
//!         eprintln!("Ошибка: {}", error);
//!     }
//! }
//! ```
//!
//! # Ограничения
//!
//! - Поддерживается только ASCII для идентификаторов
//! - Целые числа ограничены i32 диапазоном
//! - Максимальный размер файла: 1MB

pub mod common;
pub mod lexer;
pub mod preprocessor;
pub mod utils;

pub use common::{Position, Token, TokenKind};
pub use lexer::{LexerError, LexerResult, Scanner};
pub use preprocessor::{Preprocessor, PreprocessorError};

/// Версия компилятора.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Название компилятора.
pub const NAME: &str = "Mini Compiler";

/// Автор компилятора.
pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

/// Описание компилятора.
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Вспомогательные функции для работы с компилятором.
pub mod compiler {
    use super::*;

    /// Выполняет лексический анализ исходного кода.
    ///
    /// # Аргументы
    ///
    /// * `source` - исходный код на языке MiniC
    ///
    /// # Возвращает
    ///
    /// Кортеж из вектора токенов и вектора ошибок.
    ///
    /// # Пример
    ///
    /// ```
    /// use minic::compiler::lexical_analysis;
    ///
    /// let source = "fn main() { return 42; }";
    /// let (tokens, errors) = lexical_analysis(source);
    ///
    /// if errors.is_empty() {
    ///     println!("Успешно проанализировано {} токенов", tokens.len());
    /// }
    /// ```
    pub fn lexical_analysis(source: &str) -> (Vec<Token>, Vec<LexerError>) {
        let mut scanner = Scanner::new(source);
        scanner.scan_all()
    }

    /// Проверяет, является ли исходный код синтаксически корректным
    /// на уровне лексического анализа.
    ///
    /// # Аргументы
    ///
    /// * `source` - исходный код для проверки
    ///
    /// # Возвращает
    ///
    /// `true`, если не было ошибок лексического анализа.
    pub fn is_lexically_valid(source: &str) -> bool {
        let mut scanner = Scanner::new(source);
        let (_, errors) = scanner.scan_all();
        errors.is_empty()
    }

    /// Форматирует результат лексического анализа для вывода.
    ///
    /// # Аргументы
    ///
    /// * `tokens` - список токенов
    /// * `errors` - список ошибок
    ///
    /// # Возвращает
    ///
    /// Отформатированную строку с результатом анализа.
    pub fn format_lexical_analysis_result(tokens: &[Token], errors: &[LexerError]) -> String {
        use crate::utils;

        let mut result = String::new();

        if !errors.is_empty() {
            result.push_str(&utils::format_errors(errors));
            result.push('\n');
        }

        result.push_str(&utils::format_tokens(tokens));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexical_analysis() {
        let source = "x = 42;";
        let (tokens, errors) = compiler::lexical_analysis(source);

        assert!(errors.is_empty());
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_is_lexically_valid() {
        assert!(compiler::is_lexically_valid("x = 42;"));
        assert!(!compiler::is_lexically_valid("x = @;"));
    }
}
