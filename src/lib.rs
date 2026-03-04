//! Mini Compiler - компилятор упрощенного C-подобного языка.
//!
//! # Обзор
//!
//! Mini Compiler - это учебный проект, реализующий полный pipeline
//! компиляции для упрощенного C-подобного языка (MiniC).

pub mod common;
pub mod lexer;
pub mod parser;
pub mod preprocessor;
pub mod utils;

pub use common::{Position, Token, TokenKind};
pub use lexer::{LexerError, LexerErrorExt, LexerResult, Scanner};
pub use parser::dot_generator::DotGenerator;
pub use parser::json_generator::JsonGenerator;
pub use parser::pretty_printer::PrettyPrinter;
pub use parser::visitor::{DefaultVisitor, Visitor, VisitorMut};
pub use parser::{ParseError, ParseErrorKind, ParseErrors, ParseOutput, Parser, ast::*};
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
    pub fn lexical_analysis(source: &str) -> (Vec<Token>, Vec<LexerError>) {
        let mut scanner = Scanner::new(source);
        scanner.scan_all()
    }

    /// Выполняет синтаксический анализ исходного кода.
    pub fn syntactic_analysis(source: &str) -> ParseOutput {
        let (tokens, lex_errors) = lexical_analysis(source);

        if !lex_errors.is_empty() {
            let mut errors = ParseErrors::new();
            for lex_error in lex_errors {
                errors.add(ParseError::from_lexer_error(lex_error));
            }
            return ParseOutput::new(None, errors);
        }

        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        ParseOutput::new(ast, parser.errors().clone())
    }

    /// Выполняет полный пайплайн компиляции.
    pub fn compile(source: &str, defines: Vec<(&str, &str)>) -> ParseOutput {
        let mut preprocessor = Preprocessor::new(source);
        for (name, value) in defines {
            if let Err(e) = preprocessor.define(name, value) {
                let mut errors = ParseErrors::new();
                errors.add(ParseError::from_preprocessor_error(e));
                return ParseOutput::new(None, errors);
            }
        }

        let processed = match preprocessor.process() {
            Ok(s) => s,
            Err(e) => {
                let mut errors = ParseErrors::new();
                errors.add(ParseError::from_preprocessor_error(e));
                return ParseOutput::new(None, errors);
            }
        };

        let (tokens, lex_errors) = lexical_analysis(&processed);

        if !lex_errors.is_empty() {
            let mut errors = ParseErrors::new();
            for lex_error in lex_errors {
                errors.add(ParseError::from_lexer_error(lex_error));
            }
            return ParseOutput::new(None, errors);
        }

        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        ParseOutput::new(ast, parser.errors().clone())
    }

    /// Проверяет, является ли исходный код синтаксически корректным.
    pub fn is_syntactically_valid(source: &str) -> bool {
        let output = syntactic_analysis(source);
        output.is_valid()
    }

    /// Форматирует результат лексического анализа для вывода.
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

impl ParseError {
    /// Создает ошибку парсера из ошибки лексического анализатора
    pub fn from_lexer_error(error: LexerError) -> Self {
        Self::new(error.position().clone(), ParseErrorKind::SyntaxError)
            .with_message(error.to_string())
    }

    /// Создает ошибку парсера из ошибки препроцессора
    pub fn from_preprocessor_error(error: PreprocessorError) -> Self {
        Self::new(Position::new(1, 1), ParseErrorKind::SyntaxError).with_message(error.to_string())
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
    fn test_syntactic_analysis_valid() {
        let source = "fn main() { return 42; }";
        let output = compiler::syntactic_analysis(source);
        assert!(output.is_valid());
    }

    #[test]
    fn test_syntactic_analysis_invalid() {
        let source = "fn main() { return 42 }";
        let output = compiler::syntactic_analysis(source);
        assert!(!output.is_valid());
        assert!(output.has_errors());

        assert!(output.errors.len() > 0);
    }

    #[test]
    fn test_compile_with_preprocessor() {
        let source = r#"
            #define ANSWER 42
            fn main() { return ANSWER; }
        "#;
        let defines = vec![];
        let output = compiler::compile(source, defines);
        assert!(output.is_valid());
    }
}
