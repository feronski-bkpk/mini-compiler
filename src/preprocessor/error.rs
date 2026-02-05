//! Ошибки препроцессора.

use crate::common::position::Position;
use thiserror::Error;

/// Ошибки, возникающие в процессе работы препроцессора.
#[derive(Debug, Error)]
pub enum PreprocessorError {
    #[error("{position}: Незавершенный многострочный комментарий")]
    UnterminatedComment { position: Position },

    #[error("{position}: Некорректная директива препроцессора '{directive}': {reason}")]
    InvalidDirective {
        position: Position,
        directive: String,
        reason: String,
    },

    #[error("{position}: Непарная директива #endif")]
    UnmatchedEndif { position: Position },

    #[error("{position}: Незавершенная условная директива")]
    UnterminatedConditional { position: Position },

    #[error("Некорректное имя макроса: '{name}'")]
    InvalidMacroName { name: String },

    #[error("Рекурсивное определение макроса: '{name}'")]
    MacroRecursion { name: String },

    #[error("Ошибка подстановки макроса: {message}")]
    MacroExpansion { message: String },

    #[error("{position}: Непарная директива #else")]
    UnmatchedElse { position: Position },

    #[error("{position}: Неожиданная директива #endif")]
    UnexpectedEndif { position: Position },

    #[error("Циклическая зависимость макросов: {cycle:?}")]
    MacroCycle { cycle: Vec<String> },

    #[error("{position}: Макрос '{name}' не определен")]
    UndefinedMacro { position: Position, name: String },

    #[error("{position}: Некорректный синтаксис директивы")]
    InvalidSyntax { position: Position, details: String },
}

/// Результат работы препроцессора с возможной ошибкой.
#[allow(dead_code)]
pub type PreprocessorResult<T> = Result<T, PreprocessorError>;
