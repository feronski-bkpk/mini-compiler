//! Ошибки парсера для языка MiniC

use crate::common::position::Position;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

/// Типы ошибок парсера
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    /// Ошибка восстановления после ошибки
    RecoveryError,
    /// Каскадная ошибка (предотвращена)
    CascadingErrorPrevented,
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
            ParseErrorKind::RecoveryError => write!(f, "ошибка восстановления"),
            ParseErrorKind::CascadingErrorPrevented => write!(f, "предотвращена каскадная ошибка"),
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
    /// Предложение по исправлению
    pub suggestion: Option<String>,
    /// Является ли ошибка каскадной (производной от другой)
    pub is_cascading: bool,
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
            suggestion: None,
            is_cascading: false,
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

    /// Добавляет предложение по исправлению
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    /// Помечает ошибку как каскадную
    pub fn as_cascading(mut self) -> Self {
        self.is_cascading = true;
        self
    }

    /// Генерирует предложение по исправлению на основе типа ошибки
    pub fn generate_suggestion(&self) -> Option<String> {
        if self.suggestion.is_some() {
            return self.suggestion.clone();
        }

        match self.kind {
            ParseErrorKind::MissingSemicolon => {
                Some("Попробуйте добавить ';' в конце инструкции".to_string())
            }
            ParseErrorKind::MissingOpenParen => Some("Попробуйте добавить '('".to_string()),
            ParseErrorKind::MissingCloseParen => Some("Попробуйте добавить ')'".to_string()),
            ParseErrorKind::MissingOpenBrace => {
                Some("Попробуйте добавить '{{' в начале блока".to_string())
            }
            ParseErrorKind::MissingCloseBrace => {
                Some("Попробуйте добавить '}}' в конце блока".to_string())
            }
            ParseErrorKind::UnexpectedToken => {
                if let Some(found) = &self.found {
                    if found == "}" {
                        Some(
                            "Возможно, лишняя закрывающая скобка или пропущена открывающая"
                                .to_string(),
                        )
                    } else if found == ")" {
                        Some("Возможно, лишняя закрывающая скобка".to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            ParseErrorKind::InvalidExpression => Some("Проверьте синтаксис выражения".to_string()),
            _ => None,
        }
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

        if let Some(suggestion) = &self.suggestion {
            write!(f, "\n  Совет: {}", suggestion)?;
        } else if let Some(suggestion) = self.generate_suggestion() {
            write!(f, "\n  Совет: {}", suggestion)?;
        }

        if self.is_cascading {
            write!(f, " [каскадная ошибка]")?;
        }

        Ok(())
    }
}

impl std::error::Error for ParseError {}

/// Результат парсинга с возможными ошибками
pub type ParseResult<T> = Result<T, ParseError>;

/// Метрики ошибок для оценки качества восстановления
#[derive(Debug, Default, Clone)]
pub struct ErrorMetrics {
    /// Общее количество обнаруженных ошибок
    pub total_errors_detected: usize,
    /// Количество фактических ошибок (после дедупликации)
    pub actual_errors: usize,
    /// Количество предотвращенных каскадных ошибок
    pub cascading_prevented: usize,
    /// Количество успешно восстановленных ошибок
    pub recovered_errors: usize,
    /// Карта ошибок по типам
    pub errors_by_type: HashMap<ParseErrorKind, usize>,
    /// Карта ошибок по позициям (для обнаружения каскадных)
    pub errors_by_position: HashMap<String, usize>,
}

impl ErrorMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Добавляет ошибку в метрики
    pub fn add_error(&mut self, error: &ParseError) {
        self.total_errors_detected += 1;

        let pos_key = format!("{}:{}", error.position.line, error.position.column);
        let entry = self.errors_by_position.entry(pos_key).or_insert(0);

        if *entry == 0 {
            self.actual_errors += 1;
        }
        *entry += 1;

        *self.errors_by_type.entry(error.kind.clone()).or_insert(0) += 1;

        if error.is_cascading {
            self.cascading_prevented += 1;
        }
    }

    /// Отмечает успешно восстановленную ошибку
    pub fn mark_recovered(&mut self) {
        self.recovered_errors += 1;
    }

    /// Возвращает качество восстановления (0.0 - 1.0)
    pub fn recovery_quality(&self) -> f64 {
        if self.total_errors_detected == 0 {
            1.0
        } else {
            self.recovered_errors as f64 / self.total_errors_detected as f64
        }
    }

    /// Возвращает эффективность предотвращения каскадных ошибок
    pub fn cascade_prevention_efficiency(&self) -> f64 {
        if self.total_errors_detected == 0 {
            1.0
        } else {
            self.cascading_prevented as f64 / self.total_errors_detected as f64
        }
    }

    /// Возвращает точность обнаружения (actual / total)
    pub fn detection_accuracy(&self) -> f64 {
        if self.total_errors_detected == 0 {
            1.0
        } else {
            self.actual_errors as f64 / self.total_errors_detected as f64
        }
    }
}

impl fmt::Display for ErrorMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Метрики ошибок:")?;
        writeln!(f, "  Обнаружено ошибок: {}", self.total_errors_detected)?;
        writeln!(f, "  Фактических ошибок: {}", self.actual_errors)?;
        writeln!(f, "  Предотвращено каскадных: {}", self.cascading_prevented)?;
        writeln!(f, "  Успешно восстановлено: {}", self.recovered_errors)?;
        writeln!(
            f,
            "  Качество восстановления: {:.1}%",
            self.recovery_quality() * 100.0
        )?;
        writeln!(
            f,
            "  Точность обнаружения: {:.1}%",
            self.detection_accuracy() * 100.0
        )?;

        if !self.errors_by_type.is_empty() {
            writeln!(f, "  Ошибки по типам:")?;
            let mut sorted: Vec<_> = self.errors_by_type.iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(a.1));
            for (kind, count) in sorted.iter().take(5) {
                writeln!(f, "    {}: {}", kind, count)?;
            }
        }

        Ok(())
    }
}

/// Коллекция ошибок парсера (для множественных ошибок)
#[derive(Debug, Default, Clone)]
pub struct ParseErrors {
    pub errors: Vec<ParseError>,
    /// Метрики ошибок
    pub metrics: ErrorMetrics,
    /// Максимальное количество ошибок перед остановкой
    pub max_errors: Option<usize>,
}

impl ParseErrors {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            metrics: ErrorMetrics::new(),
            max_errors: Some(50), // По умолчанию останавливаемся после 50 ошибок
        }
    }

    pub fn with_max_errors(mut self, max: usize) -> Self {
        self.max_errors = Some(max);
        self
    }

    pub fn without_limit(mut self) -> Self {
        self.max_errors = None;
        self
    }

    pub fn add(&mut self, error: ParseError) {
        // Проверяем лимит ошибок
        if let Some(max) = self.max_errors {
            if self.errors.len() >= max {
                return;
            }
        }

        let is_cascading = self.is_cascading_error(&error);
        let error = if is_cascading {
            error.as_cascading()
        } else {
            error
        };

        self.metrics.add_error(&error);
        self.errors.push(error);
    }

    /// Проверяет, является ли ошибка каскадной (произошла в том же месте)
    fn is_cascading_error(&self, error: &ParseError) -> bool {
        if self.errors.is_empty() {
            return false;
        }

        // Проверяем, была ли уже ошибка на этой или предыдущей позиции
        let current_pos = (error.position.line, error.position.column);

        for prev_error in &self.errors {
            let prev_pos = (prev_error.position.line, prev_error.position.column);

            // Если ошибка на той же строке и близко по колонке (в пределах 5 символов)
            if prev_pos.0 == current_pos.0 && (current_pos.1 as i32 - prev_pos.1 as i32).abs() <= 5
            {
                return true;
            }

            // Если ошибка на предыдущей строке
            if prev_pos.0 + 1 == current_pos.0 {
                return true;
            }
        }

        false
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

    /// Возвращает true, если достигнут лимит ошибок
    pub fn reached_limit(&self) -> bool {
        if let Some(max) = self.max_errors {
            self.errors.len() >= max
        } else {
            false
        }
    }

    /// Очищает все ошибки и сбрасывает метрики
    pub fn clear(&mut self) {
        self.errors.clear();
        self.metrics = ErrorMetrics::new();
    }
}

impl fmt::Display for ParseErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in &self.errors {
            writeln!(f, "{}", error)?;
        }

        if self.reached_limit() {
            writeln!(
                f,
                "\nДостигнут лимит ошибок ({}). Остановка.",
                self.max_errors.unwrap()
            )?;
        }

        if !self.errors.is_empty() {
            writeln!(f, "\n{}", self.metrics)?;
        }

        Ok(())
    }
}
