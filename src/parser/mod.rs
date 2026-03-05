//! Модуль парсера для языка MiniC
//!
//! Этот модуль содержит парсер с рекурсивным спуском,
//! который преобразует поток токенов в абстрактное синтаксическое дерево (AST).

pub mod ast;
pub mod dot_generator;
pub mod error;
pub mod error_productions;
pub mod json_generator;
pub mod ll1;
pub mod parser;
pub mod pretty_printer;
pub mod visitor;

pub use ast::*;
pub use dot_generator::DotGenerator;
pub use error::{ErrorMetrics, ParseError, ParseErrorKind, ParseErrors, ParseResult};
pub use error_productions::{ErrorNode, ErrorProductions};
pub use json_generator::JsonGenerator;
pub use ll1::{FirstFollowCalculator, GrammarSymbol, Production};
pub use parser::Parser;
pub use pretty_printer::PrettyPrinter;
pub use visitor::{DefaultVisitor, Visitor, VisitorMut};

/// Результат парсинга с AST и возможными ошибками
#[derive(Debug)]
pub struct ParseOutput {
    /// Корневой узел AST (программа)
    pub ast: Option<Program>,
    /// Ошибки, обнаруженные во время парсинга
    pub errors: ParseErrors,
}

impl ParseOutput {
    pub fn new(ast: Option<Program>, errors: ParseErrors) -> Self {
        Self { ast, errors }
    }

    /// Была ли программа успешно разобрана (без фатальных ошибок)
    pub fn is_valid(&self) -> bool {
        self.ast.is_some() && self.errors.is_empty()
    }

    /// Есть ли ошибки
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Возвращает метрики ошибок
    pub fn error_metrics(&self) -> &ErrorMetrics {
        &self.errors.metrics
    }
}
