//! Ошибки лексического анализатора.
//!
//! Определяет типы ошибок, которые могут возникнуть в процессе
//! лексического анализа исходного кода на языке MiniC.
//!
//! # Категории ошибок
//!
//! 1. **Синтаксические ошибки** - некорректные символы, незавершенные конструкции
//! 2. **Семантические ошибки** - слишком длинные идентификаторы, некорректные числа
//! 3. **Ошибки восстановления** - контекст для продолжения анализа после ошибок
//!
//! # Обработка ошибок
//!
//! Все ошибки реализуют трейт `std::error::Error` и могут быть
//! легко преобразованы в человекочитаемые сообщения.

use crate::common::position::Position;
use thiserror::Error;

/// Ошибки, возникающие в процессе лексического анализа.
///
/// Каждая ошибка содержит информацию о позиции в исходном коде
/// и детальное описание проблемы.
///
/// # Пример использования
///
/// ```
/// use minic::LexerError;
/// use minic::common::position::Position;
///
/// let error = LexerError::UnexpectedCharacter {
///     position: Position::new(1, 5),
///     character: '@',
/// };
///
/// println!("Ошибка: {}", error);
/// ```
#[derive(Debug, Error, Clone, PartialEq)]
pub enum LexerError {
    /// Встречен недопустимый символ в исходном коде.
    ///
    /// Возникает, когда сканер встречает символ, который не может
    /// быть частью ни одного допустимого токена.
    ///
    /// # Примеры
    ///
    /// - `@` (недопустимый символ)
    /// - `$variable` (символ `$` не поддерживается)
    /// - `x#y` (символ `#` вне комментария)
    #[error("{position}: Неожиданный символ '{character}'")]
    UnexpectedCharacter {
        /// Позиция недопустимого символа
        position: Position,
        /// Сам недопустимый символ
        character: char,
    },

    /// Строковый литерал не был завершен закрывающей кавычкой.
    ///
    /// Возникает, когда сканер достигает конца строки или файла
    /// до закрывающей кавычки строкового литерала.
    ///
    /// # Пример
    ///
    /// ```c
    /// "Это незавершенная строка
    /// ```
    #[error("{position}: Незавершенная строковая константа")]
    UnterminatedString {
        /// Позиция начала строкового литерала
        position: Position,
    },

    /// Некорректный формат числового литерала.
    ///
    /// Возникает в следующих случаях:
    /// 1. Число выходит за допустимый диапазон (для i32)
    /// 2. Неправильный формат числа с плавающей точкой
    /// 3. Пустая дробная часть (например, `123.`)
    ///
    /// # Примеры
    ///
    /// - `9999999999` (слишком большое для i32)
    /// - `123.` (отсутствует дробная часть)
    /// - `12.34.56` (две точки)
    #[error("{position}: Некорректный числовой формат: '{lexeme}'")]
    InvalidNumber {
        /// Позиция начала числа
        position: Position,
        /// Некорректная лексема числа
        lexeme: String,
    },

    /// Идентификатор превысил максимально допустимую длину.
    ///
    /// Согласно спецификации языка, максимальная длина идентификатора
    /// составляет 255 символов.
    ///
    /// # Пример
    ///
    /// ```c
    /// очень_длинное_имя_переменной_которое_превышает_двести_пятьдесят_пять_символов_и_поэтому_не_может_быть_использовано_в_качестве_идентификатора_потому_что_это_нарушает_спецификацию_языка_MiniC_и_требует_обработки_этой_ошибки
    /// ```
    #[error("{position}: Слишком длинный идентификатор (максимум 255 символов)")]
    IdentifierTooLong {
        /// Позиция начала идентификатора
        position: Position,
    },

    /// Многострочный комментарий не был завершен.
    ///
    /// Возникает, когда сканер достигает конца файла до закрывающей
    /// последовательности `*/` многострочного комментария.
    ///
    /// # Пример
    ///
    /// ```c
    /// /* Это незавершенный комментарий
    /// ```
    #[error("{position}: Незавершенный многострочный комментарий")]
    UnterminatedComment {
        /// Позиция начала комментария
        position: Position,
    },

    /// Некорректная escape-последовательность в строковом литерале.
    ///
    /// Возникает, когда после обратного слеша следует символ,
    /// который не является допустимой escape-последовательностью.
    ///
    /// # Пример
    ///
    /// ```c
    /// "Некорректная escape-последовательность: \z"
    /// ```
    #[error("{position}: Некорректная escape-последовательность '{sequence}' в строке")]
    InvalidEscapeSequence {
        /// Позиция escape-последовательности
        position: Position,
        /// Некорректная последовательность
        sequence: String,
    },

    /// Пустой файл или нулевой символ.
    ///
    /// Возникает в редких случаях при обработке специальных символов
    /// или пустого ввода.
    #[error("{position}: Пустой ввод или нулевой символ")]
    EmptyInput {
        /// Позиция ошибки
        position: Position,
    },
}

/// Тип-алиас для результатов лексического анализа.
///
/// Упрощает сигнатуры функций, возвращающих результат с возможной ошибкой.
///
/// # Пример использования
///
/// ```
/// use minic::LexerResult;
/// use minic::common::position::Position;
/// use minic::common::token::{Token, TokenKind};
///
/// fn process_token() -> LexerResult<Token> {
///     Ok(Token::new(
///         TokenKind::IntLiteral(42),
///         "42".to_string(),
///         Position::new(1, 1)
///     ))
/// }
/// ```
pub type LexerResult<T> = Result<T, LexerError>;

/// Контекст для восстановления после ошибок лексического анализа.
///
/// Позволяет сканеру пропускать некорректные символы и продолжать
/// анализ с следующего допустимого токена, минимизируя количество
/// ложных ошибок после первой настоящей ошибки.
///
/// # Принцип работы
///
/// 1. При обнаружении ошибки сканер помечает её как "восстановленную"
/// 2. Пропускает один или несколько некорректных символов
/// 3. Продолжает анализ со следующего допустимого символа
/// 4. Сохраняет информацию о пропущенных символах для отладки
///
/// # Пример
///
/// Для ввода `123abc @ def`:
/// - `123abc` - корректный идентификатор (хотя начинается с цифр)
/// - `@` - ошибка `UnexpectedCharacter`
/// - После восстановления сканер пропускает `@` и пробел
/// - `def` - корректный идентификатор
#[derive(Debug, Clone)]
pub struct ErrorRecovery {
    /// Количество символов, пропущенных после последней ошибки
    skipped_chars: usize,

    /// Флаг, указывающий, что произошла ошибка и было восстановление
    recovered: bool,

    /// Позиция последней ошибки (для отладки)
    last_error_position: Option<Position>,
}

impl ErrorRecovery {
    /// Создает новый контекст восстановления.
    ///
    /// Все счетчики и флаги инициализируются нулевыми значениями.
    pub fn new() -> Self {
        Self {
            skipped_chars: 0,
            recovered: false,
            last_error_position: None,
        }
    }

    /// Отмечает пропуск одного символа.
    ///
    /// Увеличивает счетчик пропущенных символов на 1.
    pub fn skip_char(&mut self) {
        self.skipped_chars += 1;
    }

    /// Отмечает пропуск нескольких символов.
    ///
    /// # Аргументы
    ///
    /// * `count` - количество пропущенных символов
    pub fn skip_chars(&mut self, count: usize) {
        self.skipped_chars += count;
    }

    /// Отмечает, что произошла ошибка и было выполнено восстановление.
    ///
    /// # Аргументы
    ///
    /// * `position` - позиция ошибки
    pub fn mark_recovered(&mut self, position: Position) {
        self.recovered = true;
        self.last_error_position = Some(position);
    }

    /// Сбрасывает состояние восстановления.
    ///
    /// Вызывается после успешного чтения корректного токена.
    pub fn reset(&mut self) {
        self.skipped_chars = 0;
        self.recovered = false;
        self.last_error_position = None;
    }

    /// Проверяет, было ли восстановление после ошибки.
    pub fn is_recovered(&self) -> bool {
        self.recovered
    }

    /// Возвращает количество пропущенных символов.
    pub fn skipped_chars(&self) -> usize {
        self.skipped_chars
    }

    /// Возвращает позицию последней ошибки.
    pub fn last_error_position(&self) -> Option<Position> {
        self.last_error_position
    }

    /// Форматирует информацию о восстановлении для отладки.
    pub fn debug_info(&self) -> String {
        if let Some(pos) = self.last_error_position {
            format!(
                "Восстановление после ошибки на позиции {}, пропущено символов: {}",
                pos, self.skipped_chars
            )
        } else {
            String::from("Ошибок восстановления не было")
        }
    }
}

impl Default for ErrorRecovery {
    /// Создает контекст восстановления с значениями по умолчанию.
    fn default() -> Self {
        Self::new()
    }
}

/// Трейт для расширенной информации об ошибках лексического анализа.
///
/// Предоставляет дополнительные методы для работы с ошибками,
/// такие как извлечение позиции и форматирование для пользователя.
pub trait LexerErrorExt {
    /// Возвращает позицию ошибки.
    fn position(&self) -> Position;

    /// Возвращает краткое описание ошибки для пользователя.
    fn user_message(&self) -> String;

    /// Возвращает предложение по исправлению ошибки.
    fn suggestion(&self) -> Option<String>;
}

impl LexerErrorExt for LexerError {
    fn position(&self) -> Position {
        match self {
            LexerError::UnexpectedCharacter { position, .. } => *position,
            LexerError::UnterminatedString { position } => *position,
            LexerError::InvalidNumber { position, .. } => *position,
            LexerError::IdentifierTooLong { position } => *position,
            LexerError::UnterminatedComment { position } => *position,
            LexerError::InvalidEscapeSequence { position, .. } => *position,
            LexerError::EmptyInput { position } => *position,
        }
    }

    fn user_message(&self) -> String {
        match self {
            LexerError::UnexpectedCharacter { character, .. } => {
                format!("Недопустимый символ '{}'", character)
            }
            LexerError::UnterminatedString { .. } => "Строковая константа не закрыта".to_string(),
            LexerError::InvalidNumber { lexeme, .. } => {
                format!("Некорректное число '{}'", lexeme)
            }
            LexerError::IdentifierTooLong { .. } => "Слишком длинное имя переменной".to_string(),
            LexerError::UnterminatedComment { .. } => "Комментарий не закрыт".to_string(),
            LexerError::InvalidEscapeSequence { sequence, .. } => {
                format!("Некорректная escape-последовательность '{}'", sequence)
            }
            LexerError::EmptyInput { .. } => "Пустой ввод".to_string(),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            LexerError::UnexpectedCharacter { character, .. } => Some(format!(
                "Удалите символ '{}' или замените его на допустимый",
                character
            )),
            LexerError::UnterminatedString { .. } => {
                Some("Добавьте закрывающую кавычку \" в конце строки".to_string())
            }
            LexerError::InvalidNumber { .. } => {
                Some("Исправьте формат числа (например, 123 или 123.45)".to_string())
            }
            LexerError::IdentifierTooLong { .. } => {
                Some("Сократите имя переменной до 255 символов".to_string())
            }
            LexerError::UnterminatedComment { .. } => {
                Some("Добавьте */ в конце комментария".to_string())
            }
            LexerError::InvalidEscapeSequence { sequence, .. } => Some(format!(
                "Используйте допустимые escape-последовательности: \\n, \\t, \\r, \\\\, \\\", \\' (вместо {})",
                sequence
            )),
            LexerError::EmptyInput { .. } => Some("Введите исходный код".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let pos = Position::new(1, 5);

        let error = LexerError::UnexpectedCharacter {
            position: pos,
            character: '@',
        };

        assert_eq!(error.to_string(), "1:5: Неожиданный символ '@'");
    }

    #[test]
    fn test_error_recovery() {
        let mut recovery = ErrorRecovery::new();

        assert!(!recovery.is_recovered());
        assert_eq!(recovery.skipped_chars(), 0);

        recovery.skip_char();
        assert_eq!(recovery.skipped_chars(), 1);

        let pos = Position::new(1, 1);
        recovery.mark_recovered(pos);
        assert!(recovery.is_recovered());
        assert_eq!(recovery.last_error_position(), Some(pos));

        recovery.reset();
        assert!(!recovery.is_recovered());
        assert_eq!(recovery.skipped_chars(), 0);
        assert_eq!(recovery.last_error_position(), None);
    }

    #[test]
    fn test_lexer_error_ext() {
        let pos = Position::new(2, 10);

        let error = LexerError::UnterminatedString { position: pos };

        assert_eq!(error.position(), pos);
        assert_eq!(error.user_message(), "Строковая константа не закрыта");
        assert_eq!(
            error.suggestion(),
            Some("Добавьте закрывающую кавычку \" в конце строки".to_string())
        );
    }

    #[test]
    fn test_error_types() {
        let pos = Position::new(1, 1);

        let errors = vec![
            LexerError::UnexpectedCharacter {
                position: pos,
                character: '#',
            },
            LexerError::UnterminatedString { position: pos },
            LexerError::InvalidNumber {
                position: pos,
                lexeme: "123.".to_string(),
            },
            LexerError::IdentifierTooLong { position: pos },
            LexerError::UnterminatedComment { position: pos },
            LexerError::InvalidEscapeSequence {
                position: pos,
                sequence: "\\z".to_string(),
            },
            LexerError::EmptyInput { position: pos },
        ];

        for error in errors {
            assert!(error.to_string().contains(&pos.to_string()));
        }
    }
}
