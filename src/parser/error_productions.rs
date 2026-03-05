//! Продукции для обработки ошибок в грамматике
//!
//! Этот модуль расширяет грамматику языка продукциями,
//! которые специально предназначены для обработки синтаксических ошибок.

use crate::common::position::Position;
use crate::parser::ast::*;
use crate::parser::error::ParseResult;
use crate::parser::parser::Parser;

/// Маркер для узлов, созданных в процессе восстановления после ошибок
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorNode {
    pub position: Position,
    pub message: String,
    pub original_token: Option<String>,
}

impl ErrorNode {
    pub fn new(position: Position, message: String) -> Self {
        Self {
            position,
            message,
            original_token: None,
        }
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.original_token = Some(token);
        self
    }
}

/// Расширение парсера для поддержки продукции с ошибками
pub trait ErrorProductions {
    /// Пытается распарсить выражение с возможной ошибкой
    fn parse_expression_with_error(&mut self) -> ParseResult<Expression>;

    /// Пытается распарсить инструкцию с возможной ошибкой
    fn parse_statement_with_error(&mut self) -> ParseResult<Statement>;

    /// Создает узел ошибки для вставки в AST
    fn create_error_node(&mut self, _message: String) -> Expression;
}

impl ErrorProductions for Parser {
    fn parse_expression_with_error(&mut self) -> ParseResult<Expression> {
        let pos = self.current_position();

        match self.parse_expression() {
            Ok(expr) => Ok(expr),
            Err(e) => {
                self.add_error(e);

                while !self.is_at_end() && !self.is_sync_point() {
                    self.advance();
                }

                Ok(Expression::Literal(Literal::new(
                    LiteralValue::Int(0),
                    pos.line,
                    pos.column,
                )))
            }
        }
    }

    fn parse_statement_with_error(&mut self) -> ParseResult<Statement> {
        let pos = self.current_position();

        match self.parse_statement() {
            Ok(stmt) => Ok(stmt),
            Err(e) => {
                self.add_error(e);

                while !self.is_at_end() && !self.is_sync_point() {
                    self.advance();
                }

                Ok(Statement::Empty(EmptyStmt::new(pos.line, pos.column)))
            }
        }
    }

    fn create_error_node(&mut self, _message: String) -> Expression {
        let pos = self.current_position();

        Expression::Literal(Literal::new(LiteralValue::Int(0), pos.line, pos.column))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::position::Position;
    use crate::common::token::{Token, TokenKind};
    use crate::parser::parser::Parser;

    #[test]
    fn test_error_production_expression() {
        let tokens = vec![
            Token::new(TokenKind::Percent, "%".to_string(), Position::new(1, 1)),
            Token::new(
                TokenKind::IntLiteral(42),
                "42".to_string(),
                Position::new(1, 2),
            ),
            Token::eof(Position::new(1, 4)),
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse_expression_with_error();

        assert!(result.is_ok());
        assert!(!parser.errors().is_empty());
    }
}
