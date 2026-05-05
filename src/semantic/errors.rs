//! Ошибки семантического анализа

use crate::common::position::Position;
use crate::semantic::type_system::Type;
use std::fmt;

/// Тип семантической ошибки
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticErrorKind {
    /// Необъявленный идентификатор
    UndeclaredIdentifier,
    /// Повторное объявление
    DuplicateDeclaration,
    /// Несоответствие типов
    TypeMismatch,
    /// Несоответствие количества аргументов
    ArgumentCountMismatch,
    /// Несоответствие типа аргумента
    ArgumentTypeMismatch,
    /// Недопустимый возвращаемый тип
    InvalidReturnType,
    /// Недопустимый тип условия
    InvalidConditionType,
    /// Использование до объявления
    UseBeforeDeclaration,
    /// Недопустимая цель присваивания
    InvalidAssignmentTarget,
    /// Необъявленное поле структуры
    UndeclaredField,
    /// Несоответствие типа в присваивании
    AssignmentTypeMismatch,
    /// Ошибка в выражении
    InvalidExpression,
    /// Выход за пределы области видимости
    ScopeError,
    InvalidBreak,
    InvalidContinue,
}

/// Семантическая ошибка
#[derive(Debug, Clone)]
pub struct SemanticError {
    /// Тип ошибки
    pub kind: SemanticErrorKind,
    /// Позиция ошибки
    pub position: Position,
    /// Сообщение об ошибке
    pub message: String,
    /// Предложение по исправлению (опционально)
    pub suggestion: Option<String>,
    /// Контекст (например, имя функции)
    pub context: Option<String>,
    /// Ожидаемый тип (для ошибок типов)
    pub expected_type: Option<Type>,
    /// Фактический тип (для ошибок типов)
    pub found_type: Option<Type>,
}

impl SemanticError {
    /// Создает новую семантическую ошибку
    pub fn new(kind: SemanticErrorKind, position: Position, message: String) -> Self {
        Self {
            kind,
            position,
            message,
            suggestion: None,
            context: None,
            expected_type: None,
            found_type: None,
        }
    }

    /// Добавляет предложение по исправлению
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    /// Добавляет контекст
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    /// Добавляет типы для ошибки несоответствия
    pub fn with_types(mut self, expected: Type, found: Type) -> Self {
        self.expected_type = Some(expected);
        self.found_type = Some(found);
        self
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error_type = match self.kind {
            SemanticErrorKind::UndeclaredIdentifier => "необъявленный идентификатор",
            SemanticErrorKind::DuplicateDeclaration => "повторное объявление",
            SemanticErrorKind::TypeMismatch => "несоответствие типов",
            SemanticErrorKind::ArgumentCountMismatch => "несоответствие количества аргументов",
            SemanticErrorKind::ArgumentTypeMismatch => "несоответствие типа аргумента",
            SemanticErrorKind::InvalidReturnType => "недопустимый возвращаемый тип",
            SemanticErrorKind::InvalidConditionType => "недопустимый тип условия",
            SemanticErrorKind::UseBeforeDeclaration => "использование до объявления",
            SemanticErrorKind::InvalidAssignmentTarget => "недопустимая цель присваивания",
            SemanticErrorKind::UndeclaredField => "необъявленное поле",
            SemanticErrorKind::AssignmentTypeMismatch => "несоответствие типов при присваивании",
            SemanticErrorKind::InvalidExpression => "недопустимое выражение",
            SemanticErrorKind::ScopeError => "ошибка области видимости",
            SemanticErrorKind::InvalidBreak => "Некорректный break",
            SemanticErrorKind::InvalidContinue => "Некорректный continue",
        };

        writeln!(f, "семантическая ошибка: {}", error_type)?;
        writeln!(
            f,
            "  --> строка {}, столбец {}",
            self.position.line, self.position.column
        )?;

        if let Some(context) = &self.context {
            writeln!(f, "  |")?;
            writeln!(f, "  | в {}", context)?;
        }

        writeln!(f, "  |")?;
        writeln!(f, "  | {}", self.message)?;

        if let (Some(expected), Some(found)) = (&self.expected_type, &self.found_type) {
            writeln!(f, "  |")?;
            writeln!(f, "  | ожидалось: {}", expected)?;
            writeln!(f, "  | получено: {}", found)?;
        }

        if let Some(suggestion) = &self.suggestion {
            writeln!(f, "  |")?;
            writeln!(f, "  | совет: {}", suggestion)?;
        }

        Ok(())
    }
}

/// Коллекция семантических ошибок
#[derive(Debug, Clone, Default)]
pub struct SemanticErrors {
    pub errors: Vec<SemanticError>,
    max_errors: usize,
    pub total_errors_detected: usize,
    pub actual_errors: usize,
}

impl SemanticErrors {
    /// Создает новую коллекцию ошибок
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            max_errors: 50,
            total_errors_detected: 0,
            actual_errors: 0,
        }
    }

    /// Устанавливает максимальное количество ошибок
    pub fn with_max_errors(mut self, max: usize) -> Self {
        self.max_errors = max;
        self
    }

    /// Добавляет ошибку
    pub fn add(&mut self, error: SemanticError) {
        self.total_errors_detected += 1;
        self.actual_errors += 1;
        self.errors.push(error);
    }

    /// Добавляет ошибку и обновляет счетчик каскадных ошибок
    pub fn add_with_cascading(&mut self, error: SemanticError) {
        self.total_errors_detected += 1;
        self.errors.push(error);
    }

    /// Проверяет, достигнут ли лимит ошибок
    pub fn reached_limit(&self) -> bool {
        self.errors.len() >= self.max_errors
    }

    /// Проверяет, есть ли ошибки
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Возвращает количество ошибок
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Очищает все ошибки
    pub fn clear(&mut self) {
        self.errors.clear();
        self.total_errors_detected = 0;
        self.actual_errors = 0;
    }

    /// Возвращает количество предотвращенных каскадных ошибок
    pub fn cascading_prevented(&self) -> usize {
        self.total_errors_detected
            .saturating_sub(self.actual_errors)
    }
}

impl fmt::Display for SemanticErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in &self.errors {
            writeln!(f, "{}", error)?;
        }
        writeln!(f, "---")?;
        writeln!(f, "Итог: найдено {} ошибок", self.errors.len())?;
        writeln!(f, "  Всего обнаружено: {}", self.total_errors_detected)?;
        writeln!(f, "  Уникальных ошибок: {}", self.actual_errors)?;
        writeln!(
            f,
            "  Предотвращено каскадных: {}",
            self.cascading_prevented()
        )?;
        Ok(())
    }
}
