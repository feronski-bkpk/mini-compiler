//! Модуль семантического анализа для языка MiniC
//!
//! Этот модуль содержит компоненты для семантической проверки программы:
//! - Таблица символов с поддержкой вложенных областей видимости
//! - Система типов с проверкой совместимости
//! - Семантический анализатор, обходящий AST
//! - Обработка ошибок с детальными сообщениями

pub mod analyzer;
pub mod errors;
pub mod pretty_printer;
pub mod symbol_table;
pub mod type_system;

pub use analyzer::SemanticAnalyzer;
pub use errors::{SemanticError, SemanticErrorKind, SemanticErrors};
pub use pretty_printer::DecoratedAstPrinter;
pub use symbol_table::{Symbol, SymbolKind, SymbolTable};
pub use type_system::{Type, TypeChecker, TypeResult};
