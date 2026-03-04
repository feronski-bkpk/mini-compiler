//! Ошибки парсера для языка MiniC

use crate::common::position::Position;
use std::fmt;

/// Типы ошибок парсера
#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
    /// Неожиданный токен
    UnexpectedToken,
    /// Ожидался другой токен
    ExpectedToken,
    /// Неожиданный конец файла
    UnexpectedEOF,
    /// Неизвестный тип
    UnknownType,
    /// Неизвестный идентификатор
    UnknownIdentifier,
    /// Неправильное выражение
    InvalidExpression,
    /// Неправильная инструкция
    InvalidStatement,
    /// Отсутствует точка с запятой
    MissingSemicolon,
    /// Отсутствует открывающая скобка
    MissingOpenParen,
    /// Отсутствует закрывающая скобка
    MissingCloseParen,
    /// Отсутствует открывающая фигурная скобка
    MissingOpenBrace,
    /// Отсутствует закрывающая фигурная скобка
    MissingCloseBrace,
    /// Ошибка в объявлении функции
    InvalidFunctionDecl,
    /// Ошибка в объявлении структуры
    InvalidStructDecl,
    /// Ошибка в объявлении переменной
    InvalidVarDecl,
    /// Ошибка в параметрах функции
    InvalidParamList,
    /// Ошибка в аргументах функции
    InvalidArgList,
    /// Ошибка приоритета операторов
    PrecedenceError,
    /// Общая синтаксическая ошибка
    SyntaxError,
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseErrorKind::UnexpectedToken => write!(f, "неожиданный токен"),
            ParseErrorKind::ExpectedToken => write!(f, "ожидался другой токен"),
            ParseErrorKind::UnexpectedEOF => write!(f, "неожиданный конец файла"),
            ParseErrorKind::UnknownType => write!(f, "неизвестный тип"),
            ParseErrorKind::UnknownIdentifier => write!(f, "неизвестный идентификатор"),
            ParseErrorKind::InvalidExpression => write!(f, "неправильное выражение"),
            ParseErrorKind::InvalidStatement => write!(f, "неправильная инструкция"),
            ParseErrorKind::MissingSemicolon => write!(f, "отсутствует точка с запятой"),
            ParseErrorKind::MissingOpenParen => write!(f, "отсутствует '('"),
            ParseErrorKind::MissingCloseParen => write!(f, "отсутствует ')'"),
            ParseErrorKind::MissingOpenBrace => write!(f, "отсутствует '{{'"),
            ParseErrorKind::MissingCloseBrace => write!(f, "отсутствует '}}'"),
            ParseErrorKind::InvalidFunctionDecl => write!(f, "неправильное объявление функции"),
            ParseErrorKind::InvalidStructDecl => write!(f, "неправильное объявление структуры"),
            ParseErrorKind::InvalidVarDecl => write!(f, "неправильное объявление переменной"),
            ParseErrorKind::InvalidParamList => write!(f, "неправильный список параметров"),
            ParseErrorKind::InvalidArgList => write!(f, "неправильный список аргументов"),
            ParseErrorKind::PrecedenceError => write!(f, "ошибка приоритета операторов"),
            ParseErrorKind::SyntaxError => write!(f, "синтаксическая ошибка"),
        }
    }
}

/// Ошибка парсера с позицией и дополнительной информацией
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    /// Позиция ошибки
    pub position: Position,
    /// Тип ошибки
    pub kind: ParseErrorKind,
    /// Ожидаемые токены (для ExpectedToken)
    pub expected: Option<Vec<String>>,
    /// Найденный токен
    pub found: Option<String>,
    /// Дополнительное сообщение
    pub message: Option<String>,
}

impl ParseError {
    /// Создает новую ошибку
    pub fn new(position: Position, kind: ParseErrorKind) -> Self {
        Self {
            position,
            kind,
            expected: None,
            found: None,
            message: None,
        }
    }

    /// Добавляет информацию об ожидаемых токенах
    pub fn with_expected(mut self, expected: Vec<String>) -> Self {
        self.expected = Some(expected);
        self
    }

    /// Добавляет информацию о найденном токене
    pub fn with_found(mut self, found: String) -> Self {
        self.found = Some(found);
        self
    }

    /// Добавляет дополнительное сообщение
    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.position, self.kind)?;

        if let Some(expected) = &self.expected {
            write!(f, ", ожидалось: {}", expected.join(", "))?;
        }

        if let Some(found) = &self.found {
            write!(f, ", найдено: '{}'", found)?;
        }

        if let Some(message) = &self.message {
            write!(f, " ({})", message)?;
        }

        Ok(())
    }
}

impl std::error::Error for ParseError {}

/// Результат парсинга с возможными ошибками
pub type ParseResult<T> = Result<T, ParseError>;

/// Коллекция ошибок парсера (для множественных ошибок)
#[derive(Debug, Default, Clone)]
pub struct ParseErrors {
    pub errors: Vec<ParseError>,
}

impl ParseErrors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }

    pub fn has_fatal(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl fmt::Display for ParseErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in &self.errors {
            writeln!(f, "{}", error)?;
        }
        Ok(())
    }
}
