//! Общие типы данных и структуры для компилятора MiniC.
//!
//! Этот модуль содержит фундаментальные типы данных, которые используются
//! во всех этапах компиляции: от лексического анализа до генерации кода.
//!
//! # Основные компоненты
//!
//! 1. **Position** - позиция в исходном коде (строка, колонка)
//! 2. **Token** - токен с типом, лексемой и позицией
//! 3. **TokenKind** - перечисление всех возможных типов токенов
//!
//! # Принципы проектирования
//!
//! - **Неизменяемость**: Большинство структур являются неизменяемыми
//! - **Прозрачность**: Поля структур публичны для простоты использования
//! - **Безопасность**: Проверки инвариантов через debug_assert
//! - **Производительность**: Использование Copy для небольших структур
//!
//! # Пример использования
//!
//! ```
//! use minic::common::{Position, Token, TokenKind};
//!
//! let pos = Position::new(10, 25);
//! println!("Позиция: {}", pos);
//!
//! let token = Token::new(
//!     TokenKind::IntLiteral(42),
//!     "42".to_string(),
//!     Position::new(1, 10)
//! );
//!
//! println!("Токен: {}", token);
//!
//! if token.is_literal() {
//!     println!("Это литерал!");
//! }
//!
//! if let Some(value) = token.as_int() {
//!     println!("Числовое значение: {}", value);
//! }
//! ```
//!
//! # Сериализация
//!
//! Все типы реализуют `Debug` и `Display` для удобства отладки и вывода.
//! Для производственного использования можно добавить сериализацию
//! через serde при необходимости.
//!
//! # Тестирование
//!
//! Каждый подмодуль содержит исчерпывающие unit-тесты, проверяющие
//! как базовую функциональность, так и граничные случаи.

pub mod position;
pub mod token;

pub use position::Position;
pub use token::{Token, TokenKind, token_type_name};

/// Вспомогательные функции для работы с общими типами.
pub mod utils {
    use super::*;

    /// Создает токен ключевого слова.
    ///
    /// # Аргументы
    ///
    /// * `keyword` - текстовое представление ключевого слова
    /// * `position` - позиция в исходном коде
    ///
    /// # Возвращает
    ///
    /// Токен соответствующего типа или `None`, если строка не является
    /// ключевым словом.
    ///
    /// # Пример
    ///
    /// ```
    /// use minic::common::{Position, utils::create_keyword_token};
    ///
    /// let pos = Position::new(1, 1);
    /// let token = create_keyword_token("if", pos).unwrap();
    /// assert!(token.is_keyword());
    /// ```
    pub fn create_keyword_token(keyword: &str, position: Position) -> Option<Token> {
        let kind = match keyword {
            "if" => TokenKind::KwIf,
            "else" => TokenKind::KwElse,
            "while" => TokenKind::KwWhile,
            "for" => TokenKind::KwFor,
            "int" => TokenKind::KwInt,
            "float" => TokenKind::KwFloat,
            "bool" => TokenKind::KwBool,
            "return" => TokenKind::KwReturn,
            "true" => TokenKind::KwTrue,
            "false" => TokenKind::KwFalse,
            "void" => TokenKind::KwVoid,
            "struct" => TokenKind::KwStruct,
            "fn" => TokenKind::KwFn,
            _ => return None,
        };

        Some(Token::new(kind, keyword.to_string(), position))
    }

    /// Проверяет, является ли строка ключевым словом.
    ///
    /// # Аргументы
    ///
    /// * `s` - проверяемая строка
    ///
    /// # Возвращает
    ///
    /// `true`, если строка является ключевым словом языка MiniC.
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

    /// Форматирует список токенов для отладки.
    ///
    /// # Аргументы
    ///
    /// * `tokens` - список токенов
    ///
    /// # Возвращает
    ///
    /// Отформатированную строку с информацией о токенах.
    pub fn format_tokens_debug(tokens: &[Token]) -> String {
        let mut result = String::new();

        for (i, token) in tokens.iter().enumerate() {
            result.push_str(&format!("{:3}: {}\n", i, token));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utils_is_keyword() {
        assert!(utils::is_keyword("if"));
        assert!(utils::is_keyword("fn"));
        assert!(utils::is_keyword("return"));
        assert!(!utils::is_keyword("x"));
        assert!(!utils::is_keyword("IF"));
    }

    #[test]
    fn test_utils_is_valid_identifier() {
        assert!(utils::is_valid_identifier("x"));
        assert!(utils::is_valid_identifier("_x"));
        assert!(utils::is_valid_identifier("x1"));
        assert!(utils::is_valid_identifier("my_var"));
        assert!(utils::is_valid_identifier("MAX_VALUE"));

        assert!(!utils::is_valid_identifier(""));
        assert!(!utils::is_valid_identifier("1x"));
        assert!(!utils::is_valid_identifier("x-y"));
        assert!(!utils::is_valid_identifier("x.y"));
        assert!(!utils::is_valid_identifier("if"));

        let long_identifier = "a".repeat(256);
        assert!(!utils::is_valid_identifier(&long_identifier));

        let max_length_identifier = "a".repeat(255);
        assert!(utils::is_valid_identifier(&max_length_identifier));
    }

    #[test]
    fn test_create_keyword_token() {
        let pos = Position::new(1, 1);

        let token = utils::create_keyword_token("if", pos).unwrap();
        assert_eq!(token.kind, TokenKind::KwIf);
        assert_eq!(token.lexeme, "if");

        let token = utils::create_keyword_token("x", pos);
        assert!(token.is_none());
    }
}
