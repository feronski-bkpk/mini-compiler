//! Типы токенов для языка MiniC.
//!
//! Определяет структуры данных для представления токенов, полученных
//! в результате лексического анализа исходного кода.
//!
//! # Основные компоненты
//!
//! 1. `TokenKind` - перечисление всех возможных типов токенов
//! 2. `Token` - структура, объединяющая тип, лексему и позицию
//! 3. Вспомогательные функции для работы с токенами
//!
//! # Пример использования
//!
//! ```
//! use minic::common::token::{Token, TokenKind};
//! use minic::common::position::Position;
//!
//! // Создание токена ключевого слова
//! let token = Token::new(
//!     TokenKind::KwIf,
//!     "if".to_string(),
//!     Position::new(1, 1)
//! );
//!
//! println!("Токен: {}", token);
//! ```

use super::position::Position;

/// Тип токена языка MiniC.
///
/// Определяет категорию токена без привязки к конкретной лексеме.
///
/// # Категории токенов
///
/// 1. **Ключевые слова** - зарезервированные слова языка
/// 2. **Идентификаторы** - имена переменных, функций и т.д.
/// 3. **Литералы** - константные значения
/// 4. **Операторы** - арифметические, логические, сравнения, присваивания
/// 5. **Разделители** - скобки, запятые, точки с запятой и т.д.
/// 6. **Специальные** - маркер конца файла
///
/// # Примечание
///
/// Для литералов значения хранятся внутри enum, что позволяет
/// избежать повторного парсинга при семантическом анализе.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// `if` - условный оператор
    KwIf,
    /// `else` - альтернативная ветка условия
    KwElse,
    /// `while` - цикл с предусловием
    KwWhile,
    /// `for` - цикл for
    KwFor,
    /// `int` - целочисленный тип
    KwInt,
    /// `float` - тип с плавающей точкой
    KwFloat,
    /// `bool` - логический тип
    KwBool,
    /// `return` - возврат из функции
    KwReturn,
    /// `true` - логическая истина
    KwTrue,
    /// `false` - логическая ложь
    KwFalse,
    /// `void` - тип без значения
    KwVoid,
    /// `struct` - определение структуры
    KwStruct,
    /// `fn` - определение функции
    KwFn,

    /// Идентификатор (имя переменной, функции и т.д.)
    ///
    /// Хранит строковое значение идентификатора.
    Identifier(String),

    /// Целочисленный литерал
    ///
    /// Диапазон: `[-2³¹, 2³¹-1]`
    IntLiteral(i32),

    /// Литерал с плавающей точкой
    ///
    /// 64-битное число двойной точности
    FloatLiteral(f64),

    /// Строковый литерал
    ///
    /// Хранит содержимое строки без кавычек
    StringLiteral(String),

    /// Логический литерал
    ///
    /// `true` или `false`
    BoolLiteral(bool),

    /// Сложение: `+`
    Plus,
    /// Вычитание: `-`
    Minus,
    /// Умножение: `*`
    Asterisk,
    /// Деление: `/`
    Slash,
    /// Остаток от деления: `%`
    Percent,

    /// Равенство: `==`
    EqEq,
    /// Неравенство: `!=`
    BangEq,
    /// Меньше: `<`
    Lt,
    /// Меньше или равно: `<=`
    LtEq,
    /// Больше: `>`
    Gt,
    /// Больше или равно: `>=`
    GtEq,

    /// Логическое И: `&&`
    AmpAmp,
    /// Логическое ИЛИ: `||`
    PipePipe,
    /// Логическое НЕ: `!`
    Bang,

    /// Присваивание: `=`
    Eq,
    /// Присваивание со сложением: `+=`
    PlusEq,
    /// Присваивание с вычитанием: `-=`
    MinusEq,
    /// Присваивание с умножением: `*=`
    AsteriskEq,
    /// Присваивание с делением: `/=`
    SlashEq,

    /// Левая круглая скобка: `(`
    LParen,
    /// Правая круглая скобка: `)`
    RParen,
    /// Левая фигурная скобка: `{`
    LBrace,
    /// Правая фигурная скобка: `}`
    RBrace,
    /// Левая квадратная скобка: `[`
    LBracket,
    /// Правая квадратная скобка: `]`
    RBracket,
    /// Точка с запятой: `;`
    Semicolon,
    /// Запятая: `,`
    Comma,
    /// Двоеточие: `:`
    Colon,

    /// Маркер конца файла
    ///
    /// Генерируется сканером при достижении конца входных данных
    EndOfFile,
}

/// Токен с полной информацией о лексеме.
///
/// Объединяет тип токена, исходную строку (лексему) и позицию
/// в исходном коде.
///
/// # Поля
///
/// * `kind` - тип токена
/// * `lexeme` - исходная строка из кода
/// * `position` - позиция начала токена
///
/// # Инварианты
///
/// 1. `lexeme` не должна быть пустой (кроме EndOfFile)
/// 2. `position` должна быть валидной позицией
/// 3. Для литералов `lexeme` должна соответствовать парсируемому значению
#[derive(Debug, Clone)]
pub struct Token {
    /// Тип токена
    pub kind: TokenKind,

    /// Исходная лексема (строка из исходного кода)
    pub lexeme: String,

    /// Позиция начала токена в исходном коде
    pub position: Position,
}

impl Token {
    /// Создает новый токен.
    ///
    /// # Аргументы
    ///
    /// * `kind` - тип токена
    /// * `lexeme` - исходная лексема
    /// * `position` - позиция в исходном коде
    ///
    /// # Пример
    ///
    /// ```
    /// use minic::common::token::{Token, TokenKind};
    /// use minic::common::position::Position;
    ///
    /// let token = Token::new(
    ///     TokenKind::IntLiteral(42),
    ///     "42".to_string(),
    ///     Position::new(1, 10)
    /// );
    /// ```
    pub fn new(kind: TokenKind, lexeme: String, position: Position) -> Self {
        debug_assert!(
            !lexeme.is_empty() || matches!(kind, TokenKind::EndOfFile),
            "Лексема не может быть пустой (кроме EndOfFile)"
        );

        debug_assert!(position.is_valid(), "Позиция должна быть валидной");

        Self {
            kind,
            lexeme,
            position,
        }
    }

    /// Создает токен конца файла.
    ///
    /// # Аргументы
    ///
    /// * `position` - позиция после последнего символа файла
    ///
    /// # Пример
    ///
    /// ```
    /// use minic::common::token::Token;
    /// use minic::common::position::Position;
    ///
    /// let eof = Token::eof(Position::new(10, 1));
    /// assert!(eof.is_eof());
    /// ```
    pub fn eof(position: Position) -> Self {
        Self::new(TokenKind::EndOfFile, String::new(), position)
    }

    /// Проверяет, является ли токен ключевым словом.
    ///
    /// # Возвращает
    ///
    /// `true` если токен - ключевое слово, `false` в противном случае
    pub fn is_keyword(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::KwIf
                | TokenKind::KwElse
                | TokenKind::KwWhile
                | TokenKind::KwFor
                | TokenKind::KwInt
                | TokenKind::KwFloat
                | TokenKind::KwBool
                | TokenKind::KwReturn
                | TokenKind::KwTrue
                | TokenKind::KwFalse
                | TokenKind::KwVoid
                | TokenKind::KwStruct
                | TokenKind::KwFn
        )
    }

    /// Проверяет, является ли токен литералом.
    ///
    /// # Возвращает
    ///
    /// `true` если токен - литерал, `false` в противном случае
    pub fn is_literal(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::IntLiteral(_)
                | TokenKind::FloatLiteral(_)
                | TokenKind::StringLiteral(_)
                | TokenKind::BoolLiteral(_)
        )
    }

    /// Проверяет, является ли токен оператором.
    ///
    /// Включает арифметические, логические операторы и операторы сравнения.
    ///
    /// # Возвращает
    ///
    /// `true` если токен - оператор, `false` в противном случае
    pub fn is_operator(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Asterisk
                | TokenKind::Slash
                | TokenKind::Percent
                | TokenKind::EqEq
                | TokenKind::BangEq
                | TokenKind::Lt
                | TokenKind::LtEq
                | TokenKind::Gt
                | TokenKind::GtEq
                | TokenKind::AmpAmp
                | TokenKind::PipePipe
                | TokenKind::Bang
                | TokenKind::Eq
                | TokenKind::PlusEq
                | TokenKind::MinusEq
                | TokenKind::AsteriskEq
                | TokenKind::SlashEq
        )
    }

    /// Проверяет, является ли токен разделителем.
    ///
    /// # Возвращает
    ///
    /// `true` если токен - разделитель, `false` в противном случае
    pub fn is_delimiter(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::LParen
                | TokenKind::RParen
                | TokenKind::LBrace
                | TokenKind::RBrace
                | TokenKind::LBracket
                | TokenKind::RBracket
                | TokenKind::Semicolon
                | TokenKind::Comma
                | TokenKind::Colon
        )
    }

    /// Проверяет, является ли токен концом файла.
    ///
    /// # Возвращает
    ///
    /// `true` если токен - EndOfFile, `false` в противном случае
    pub fn is_eof(&self) -> bool {
        matches!(self.kind, TokenKind::EndOfFile)
    }

    /// Возвращает строковое представление типа токена.
    ///
    /// Используется для вывода и логирования.
    ///
    /// # Возвращает
    ///
    /// Имя типа токена в верхнем регистре с подчеркиваниями
    pub fn type_name(&self) -> &'static str {
        token_type_name(&self.kind)
    }

    /// Извлекает целочисленное значение из токена-литерала.
    ///
    /// # Возвращает
    ///
    /// * `Some(i32)` - если токен является `IntLiteral`
    /// * `None` - в противном случае
    pub fn as_int(&self) -> Option<i32> {
        if let TokenKind::IntLiteral(value) = self.kind {
            Some(value)
        } else {
            None
        }
    }

    /// Извлекает значение с плавающей точкой из токена-литерала.
    ///
    /// # Возвращает
    ///
    /// * `Some(f64)` - если токен является `FloatLiteral`
    /// * `None` - в противном случае
    pub fn as_float(&self) -> Option<f64> {
        if let TokenKind::FloatLiteral(value) = self.kind {
            Some(value)
        } else {
            None
        }
    }

    /// Извлекает строковое значение из токена-литерала.
    ///
    /// # Возвращает
    ///
    /// * `Some(&str)` - если токен является `StringLiteral`
    /// * `None` - в противном случае
    pub fn as_string(&self) -> Option<&str> {
        if let TokenKind::StringLiteral(value) = &self.kind {
            Some(value)
        } else {
            None
        }
    }

    /// Извлекает логическое значение из токена-литерала.
    ///
    /// # Возвращает
    ///
    /// * `Some(bool)` - если токен является `BoolLiteral`
    /// * `None` - в противном случае
    pub fn as_bool(&self) -> Option<bool> {
        if let TokenKind::BoolLiteral(value) = self.kind {
            Some(value)
        } else {
            None
        }
    }
}

impl std::fmt::Display for Token {
    /// Форматирует токен для вывода.
    ///
    /// Формат: `строка:колонка ТИП_ТОКЕНА "лексема" [значение]`
    ///
    /// # Примеры
    ///
    /// ```
    /// use minic::common::token::{Token, TokenKind};
    /// use minic::common::position::Position;
    ///
    /// let token = Token::new(
    ///     TokenKind::IntLiteral(42),
    ///     "42".to_string(),
    ///     Position::new(1, 10)
    /// );
    ///
    /// assert_eq!(token.to_string(), "1:10 INT_LITERAL \"42\" 42");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_name = self.type_name();

        let literal_value = match &self.kind {
            TokenKind::IntLiteral(n) => format!(" {}", n),
            TokenKind::FloatLiteral(n) => format!(" {}", n),
            TokenKind::BoolLiteral(b) => format!(" {}", b),
            TokenKind::StringLiteral(s) => format!(" {}", s),
            _ => String::new(),
        };

        write!(
            f,
            "{} {} \"{}\"{}",
            self.position, type_name, self.lexeme, literal_value
        )
    }
}

/// Возвращает строковое представление типа токена.
///
/// # Аргументы
///
/// * `kind` - тип токена
///
/// # Возвращает
///
/// Имя типа в верхнем регистре с подчеркиваниями
pub fn token_type_name(kind: &TokenKind) -> &'static str {
    match kind {
        TokenKind::KwIf => "KW_IF",
        TokenKind::KwElse => "KW_ELSE",
        TokenKind::KwWhile => "KW_WHILE",
        TokenKind::KwFor => "KW_FOR",
        TokenKind::KwInt => "KW_INT",
        TokenKind::KwFloat => "KW_FLOAT",
        TokenKind::KwBool => "KW_BOOL",
        TokenKind::KwReturn => "KW_RETURN",
        TokenKind::KwTrue => "KW_TRUE",
        TokenKind::KwFalse => "KW_FALSE",
        TokenKind::KwVoid => "KW_VOID",
        TokenKind::KwStruct => "KW_STRUCT",
        TokenKind::KwFn => "KW_FN",
        TokenKind::Identifier(_) => "IDENTIFIER",
        TokenKind::IntLiteral(_) => "INT_LITERAL",
        TokenKind::FloatLiteral(_) => "FLOAT_LITERAL",
        TokenKind::StringLiteral(_) => "STRING_LITERAL",
        TokenKind::BoolLiteral(_) => "BOOL_LITERAL",
        TokenKind::Plus => "PLUS",
        TokenKind::Minus => "MINUS",
        TokenKind::Asterisk => "ASTERISK",
        TokenKind::Slash => "SLASH",
        TokenKind::Percent => "PERCENT",
        TokenKind::EqEq => "EQ_EQ",
        TokenKind::BangEq => "BANG_EQ",
        TokenKind::Lt => "LT",
        TokenKind::LtEq => "LT_EQ",
        TokenKind::Gt => "GT",
        TokenKind::GtEq => "GT_EQ",
        TokenKind::AmpAmp => "AMP_AMP",
        TokenKind::PipePipe => "PIPE_PIPE",
        TokenKind::Bang => "BANG",
        TokenKind::Eq => "ASSIGN",
        TokenKind::PlusEq => "PLUS_EQ",
        TokenKind::MinusEq => "MINUS_EQ",
        TokenKind::AsteriskEq => "ASTERISK_EQ",
        TokenKind::SlashEq => "SLASH_EQ",
        TokenKind::LParen => "LPAREN",
        TokenKind::RParen => "RPAREN",
        TokenKind::LBrace => "LBRACE",
        TokenKind::RBrace => "RBRACE",
        TokenKind::LBracket => "LBRACKET",
        TokenKind::RBracket => "RBRACKET",
        TokenKind::Semicolon => "SEMICOLON",
        TokenKind::Comma => "COMMA",
        TokenKind::Colon => "COLON",
        TokenKind::EndOfFile => "END_OF_FILE",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let pos = Position::new(1, 1);
        let token = Token::new(TokenKind::KwIf, "if".to_string(), pos);

        assert!(token.is_keyword());
        assert!(!token.is_literal());
        assert!(!token.is_operator());
        assert!(!token.is_delimiter());
        assert!(!token.is_eof());
    }

    #[test]
    fn test_token_type_name() {
        assert_eq!(token_type_name(&TokenKind::KwIf), "KW_IF");
        assert_eq!(token_type_name(&TokenKind::IntLiteral(42)), "INT_LITERAL");
        assert_eq!(token_type_name(&TokenKind::Plus), "PLUS");
        assert_eq!(token_type_name(&TokenKind::LParen), "LPAREN");
    }

    #[test]
    fn test_token_display() {
        let pos = Position::new(1, 1);

        let token1 = Token::new(TokenKind::KwIf, "if".to_string(), pos);
        assert_eq!(token1.to_string(), "1:1 KW_IF \"if\"");

        let token2 = Token::new(TokenKind::IntLiteral(42), "42".to_string(), pos);
        assert_eq!(token2.to_string(), "1:1 INT_LITERAL \"42\" 42");

        let token3 = Token::new(
            TokenKind::StringLiteral("hello".to_string()),
            "\"hello\"".to_string(),
            pos,
        );
        assert_eq!(token3.to_string(), "1:1 STRING_LITERAL \"\"hello\"\" hello");
    }

    #[test]
    fn test_token_value_extraction() {
        let pos = Position::new(1, 1);

        let int_token = Token::new(TokenKind::IntLiteral(42), "42".to_string(), pos);
        assert_eq!(int_token.as_int(), Some(42));
        assert_eq!(int_token.as_float(), None);

        let float_token = Token::new(TokenKind::FloatLiteral(3.14), "3.14".to_string(), pos);
        assert_eq!(float_token.as_float(), Some(3.14));
        assert_eq!(float_token.as_int(), None);

        let string_token = Token::new(
            TokenKind::StringLiteral("test".to_string()),
            "\"test\"".to_string(),
            pos,
        );
        assert_eq!(string_token.as_string(), Some("test"));

        let bool_token = Token::new(TokenKind::BoolLiteral(true), "true".to_string(), pos);
        assert_eq!(bool_token.as_bool(), Some(true));
    }

    #[test]
    fn test_token_categories() {
        let pos = Position::new(1, 1);

        let kw_token = Token::new(TokenKind::KwIf, "if".to_string(), pos);
        assert!(kw_token.is_keyword());

        let lit_token = Token::new(TokenKind::IntLiteral(42), "42".to_string(), pos);
        assert!(lit_token.is_literal());

        let op_token = Token::new(TokenKind::Plus, "+".to_string(), pos);
        assert!(op_token.is_operator());

        let delim_token = Token::new(TokenKind::LParen, "(".to_string(), pos);
        assert!(delim_token.is_delimiter());

        let eof_token = Token::eof(pos);
        assert!(eof_token.is_eof());
    }

    #[test]
    fn test_eof_token() {
        let pos = Position::new(10, 1);
        let eof = Token::eof(pos);

        assert!(eof.is_eof());
        assert!(eof.lexeme.is_empty());
        assert_eq!(eof.position, pos);
    }
}
