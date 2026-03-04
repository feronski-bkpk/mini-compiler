//! Реализация парсера с рекурсивным спуском для языка MiniC

use crate::common::position::Position;
use crate::common::token::{Token, TokenKind};
use crate::parser::ast::*;
use crate::parser::error::{ParseError, ParseErrorKind, ParseErrors, ParseResult};

/// Парсер с рекурсивным спуском для языка MiniC
pub struct Parser {
    /// Входной поток токенов
    tokens: Vec<Token>,
    /// Текущая позиция в потоке токенов
    current: usize,
    /// Собранные ошибки
    errors: ParseErrors,
    /// Флаг для восстановления после ошибок
    in_error_recovery: bool,
}

impl Parser {
    /// Создает новый парсер из списка токенов
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            errors: ParseErrors::new(),
            in_error_recovery: false,
        }
    }

    /// Возвращает собранные ошибки
    pub fn errors(&self) -> &ParseErrors {
        &self.errors
    }

    /// Основной метод разбора - парсит всю программу
    pub fn parse(&mut self) -> Option<Program> {
        let start_pos = self.current_position();
        let mut declarations = Vec::new();

        while !self.is_at_end() {
            if matches!(self.peek().kind, TokenKind::RBrace) {
                let pos = self.current_position();
                self.errors.add(
                    ParseError::new(pos, ParseErrorKind::UnexpectedToken)
                        .with_found(self.peek().lexeme.clone())
                        .with_message(
                            "неожиданная закрывающая скобка на верхнем уровне".to_string(),
                        ),
                );
                self.advance();
                continue;
            }

            match self.parse_declaration() {
                Ok(decl) => {
                    declarations.push(decl);
                }
                Err(e) => {
                    self.errors.add(e);
                    if !self.synchronize() {
                        break;
                    }
                }
            }
        }

        if declarations.is_empty() && self.errors.has_fatal() {
            None
        } else {
            Some(Program::new(declarations, start_pos.line, start_pos.column))
        }
    }

    // === Вспомогательные методы ===

    /// Возвращает текущий токен без потребления
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    /// Возвращает предыдущий токен
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    /// Проверяет, достигнут ли конец файла
    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::EndOfFile
    }

    /// Возвращает текущую позицию
    fn current_position(&self) -> Position {
        self.peek().position.clone()
    }

    /// Проверяет, совпадает ли текущий токен с ожидаемым типом
    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
        }
    }

    /// Проверяет, совпадает ли текущий токен с любым из ожидаемых типов
    #[allow(dead_code)]
    fn check_any(&self, kinds: &[TokenKind]) -> bool {
        kinds.iter().any(|kind| self.check(kind))
    }

    /// Потребляет текущий токен, если он совпадает с ожидаемым типом
    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Потребляет текущий токен, если он совпадает с любым из ожидаемых типов
    fn match_any(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    /// Переходит к следующему токену
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    /// Потребляет текущий токен или сообщает об ошибке
    fn consume(
        &mut self,
        kind: &TokenKind,
        error_kind: ParseErrorKind,
        message: &str,
    ) -> ParseResult<&Token> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            let pos = self.current_position();
            let found = self.peek().lexeme.clone();
            Err(ParseError::new(pos, error_kind)
                .with_found(found)
                .with_message(message.to_string()))
        }
    }

    /// Синхронизация после ошибки - пропускает токены до следующей точки синхронизации
    fn synchronize(&mut self) -> bool {
        self.in_error_recovery = true;

        if !self.is_at_end() {
            self.advance();
        }

        while !self.is_at_end() {
            if self.previous().kind == TokenKind::Semicolon {
                break;
            }

            match &self.peek().kind {
                TokenKind::KwFn
                | TokenKind::KwStruct
                | TokenKind::KwIf
                | TokenKind::KwWhile
                | TokenKind::KwFor
                | TokenKind::KwReturn
                | TokenKind::RBrace
                | TokenKind::EndOfFile => {
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }

        self.in_error_recovery = false;
        let result = !self.is_at_end();
        result
    }

    // === Парсинг объявлений ===

    /// Парсит объявление верхнего уровня
    fn parse_declaration(&mut self) -> ParseResult<Declaration> {
        match &self.peek().kind {
            TokenKind::KwFn => Ok(Declaration::Function(self.parse_function_decl()?)),
            TokenKind::KwStruct => Ok(Declaration::Struct(self.parse_struct_decl()?)),
            _ => {
                if self.is_type_start() {
                    Ok(Declaration::Variable(self.parse_var_decl()?))
                } else {
                    let pos = self.current_position();
                    Err(ParseError::new(pos, ParseErrorKind::SyntaxError)
                        .with_found(self.peek().lexeme.clone())
                        .with_message(
                            "ожидалось объявление функции, структуры или переменной".to_string(),
                        ))
                }
            }
        }
    }

    /// Проверяет, начинается ли токен с типа
    fn is_type_start(&self) -> bool {
        matches!(
            &self.peek().kind,
            TokenKind::KwInt
                | TokenKind::KwFloat
                | TokenKind::KwBool
                | TokenKind::KwVoid
                | TokenKind::KwString
                | TokenKind::KwStruct
        )
    }

    /// Парсит объявление функции: fn name(params) -> Type { ... }
    fn parse_function_decl(&mut self) -> ParseResult<FunctionDecl> {
        let start_pos = self.current_position();

        self.consume(
            &TokenKind::KwFn,
            ParseErrorKind::ExpectedToken,
            "ожидалось 'fn'",
        )?;

        let name = match self.advance() {
            token if matches!(token.kind, TokenKind::Identifier(_)) => token.lexeme.clone(),
            token => {
                return Err(
                    ParseError::new(token.position.clone(), ParseErrorKind::ExpectedToken)
                        .with_found(token.lexeme.clone()),
                );
            }
        };

        self.consume(
            &TokenKind::LParen,
            ParseErrorKind::MissingOpenParen,
            "ожидалось '(' после имени функции",
        )?;

        let parameters = self.parse_param_list()?;

        if name == "main" && !parameters.is_empty() {
            return Err(
                ParseError::new(start_pos, ParseErrorKind::InvalidFunctionDecl)
                    .with_message("Функция main не может иметь параметров".to_string()),
            );
        }

        self.consume(
            &TokenKind::RParen,
            ParseErrorKind::MissingCloseParen,
            "ожидалось ')' после параметров",
        )?;

        let return_type = if self.match_token(&TokenKind::Arrow) {
            self.parse_type()?
        } else {
            Type::Void
        };

        let body = self.parse_block()?;

        Ok(FunctionDecl::new(
            name,
            return_type,
            parameters,
            body,
            start_pos.line,
            start_pos.column,
        ))
    }

    /// Парсит список параметров функции
    fn parse_param_list(&mut self) -> ParseResult<Vec<Param>> {
        let mut params = Vec::new();

        if self.check(&TokenKind::RParen) {
            return Ok(params);
        }

        params.push(self.parse_param()?);

        while self.match_token(&TokenKind::Comma) {
            if self.check(&TokenKind::RParen) {
                break;
            }
            params.push(self.parse_param()?);
        }

        Ok(params)
    }

    /// Парсит один параметр: Type name
    fn parse_param(&mut self) -> ParseResult<Param> {
        let start_pos = self.current_position();
        let param_type = self.parse_type()?;

        let name = match self.advance() {
            token if matches!(token.kind, TokenKind::Identifier(_)) => token.lexeme.clone(),
            token => {
                return Err(
                    ParseError::new(token.position.clone(), ParseErrorKind::ExpectedToken)
                        .with_found(token.lexeme.clone()),
                );
            }
        };

        Ok(Param::new(
            param_type,
            name,
            start_pos.line,
            start_pos.column,
        ))
    }

    /// Парсит объявление структуры: struct Name { fields }
    fn parse_struct_decl(&mut self) -> ParseResult<StructDecl> {
        let start_pos = self.current_position();

        self.consume(
            &TokenKind::KwStruct,
            ParseErrorKind::ExpectedToken,
            "ожидалось 'struct'",
        )?;

        let name = match self.advance() {
            token if matches!(token.kind, TokenKind::Identifier(_)) => token.lexeme.clone(),
            token => {
                return Err(
                    ParseError::new(token.position.clone(), ParseErrorKind::ExpectedToken)
                        .with_found(token.lexeme.clone()),
                );
            }
        };

        self.consume(
            &TokenKind::LBrace,
            ParseErrorKind::MissingOpenBrace,
            "ожидалось '{{' после имени структуры",
        )?;

        let mut fields = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            fields.push(self.parse_var_decl()?);
        }

        self.consume(
            &TokenKind::RBrace,
            ParseErrorKind::MissingCloseBrace,
            "ожидалось '}}' в конце структуры",
        )?;

        Ok(StructDecl::new(
            name,
            fields,
            start_pos.line,
            start_pos.column,
        ))
    }

    /// Парсит объявление переменной: Type name [= Expression];
    fn parse_var_decl(&mut self) -> ParseResult<VarDecl> {
        let start_pos = self.current_position();

        let var_type = self.parse_type()?;

        let name = match self.advance() {
            token if matches!(token.kind, TokenKind::Identifier(_)) => token.lexeme.clone(),
            token => {
                return Err(
                    ParseError::new(token.position.clone(), ParseErrorKind::ExpectedToken)
                        .with_found(token.lexeme.clone()),
                );
            }
        };

        let initializer = if self.match_token(&TokenKind::Eq) {
            let expr = self.parse_expression()?;
            Some(expr)
        } else {
            None
        };

        self.consume(
            &TokenKind::Semicolon,
            ParseErrorKind::MissingSemicolon,
            "ожидалось ';' после объявления переменной",
        )?;

        Ok(VarDecl::new(
            var_type,
            name,
            initializer,
            start_pos.line,
            start_pos.column,
        ))
    }

    /// Парсит тип
    fn parse_type(&mut self) -> ParseResult<Type> {
        let token = self.peek().clone();
        let pos = token.position.clone();

        let typ = match &token.kind {
            TokenKind::KwInt => {
                self.advance();
                Type::Int
            }
            TokenKind::KwFloat => {
                self.advance();
                Type::Float
            }
            TokenKind::KwBool => {
                self.advance();
                Type::Bool
            }
            TokenKind::KwVoid => {
                self.advance();
                Type::Void
            }
            TokenKind::KwString => {
                self.advance();
                Type::String
            }
            TokenKind::KwStruct => {
                self.advance();

                match self.peek() {
                    token if matches!(token.kind, TokenKind::Identifier(_)) => {
                        let name = token.lexeme.clone();
                        self.advance();
                        Type::Struct(name)
                    }
                    token => {
                        return Err(ParseError::new(
                            token.position.clone(),
                            ParseErrorKind::ExpectedToken,
                        )
                        .with_found(token.lexeme.clone()));
                    }
                }
            }
            _ => {
                return Err(
                    ParseError::new(pos, ParseErrorKind::UnknownType).with_found(token.lexeme)
                );
            }
        };

        Ok(typ)
    }

    // === Парсинг инструкций ===

    /// Парсит инструкцию
    fn parse_statement(&mut self) -> ParseResult<Statement> {
        match &self.peek().kind {
            TokenKind::KwIf => Ok(Statement::If(self.parse_if_stmt()?)),
            TokenKind::KwWhile => Ok(Statement::While(self.parse_while_stmt()?)),
            TokenKind::KwFor => Ok(Statement::For(self.parse_for_stmt()?)),
            TokenKind::KwReturn => Ok(Statement::Return(self.parse_return_stmt()?)),
            TokenKind::LBrace => Ok(Statement::Block(self.parse_block()?)),
            TokenKind::Semicolon => {
                self.advance();
                Ok(Statement::Empty(EmptyStmt::new(
                    self.current_position().line,
                    self.current_position().column,
                )))
            }
            _ => {
                if self.is_type_start() {
                    Ok(Statement::VariableDecl(self.parse_var_decl()?))
                } else {
                    let expr_stmt = self.parse_expr_stmt()?;
                    Ok(Statement::Expression(expr_stmt))
                }
            }
        }
    }

    /// Парсит инструкцию-выражение: Expression;
    fn parse_expr_stmt(&mut self) -> ParseResult<ExprStmt> {
        let start_pos = self.current_position();
        let expr = self.parse_expression()?;
        self.consume(
            &TokenKind::Semicolon,
            ParseErrorKind::MissingSemicolon,
            "ожидалось ';' после выражения",
        )?;
        Ok(ExprStmt::new(expr, start_pos.line, start_pos.column))
    }

    /// Парсит блок: { statements }
    fn parse_block(&mut self) -> ParseResult<BlockStmt> {
        let start_pos = self.current_position();

        self.consume(
            &TokenKind::LBrace,
            ParseErrorKind::MissingOpenBrace,
            "ожидалось '{{' в начале блока",
        )?;

        let mut statements = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        if self.check(&TokenKind::RBrace) {
            self.advance();
        } else {
            return Err(ParseError::new(
                self.current_position(),
                ParseErrorKind::MissingCloseBrace,
            )
            .with_found(self.peek().lexeme.clone()));
        }

        Ok(BlockStmt::new(statements, start_pos.line, start_pos.column))
    }

    /// Парсит if-else: if (condition) statement [else statement]
    fn parse_if_stmt(&mut self) -> ParseResult<IfStmt> {
        let start_pos = self.current_position();

        self.consume(
            &TokenKind::KwIf,
            ParseErrorKind::ExpectedToken,
            "ожидалось 'if'",
        )?;
        self.consume(
            &TokenKind::LParen,
            ParseErrorKind::MissingOpenParen,
            "ожидалось '(' после 'if'",
        )?;

        let condition = self.parse_expression()?;

        self.consume(
            &TokenKind::RParen,
            ParseErrorKind::MissingCloseParen,
            "ожидалось ')' после условия",
        )?;

        let then_branch = self.parse_statement()?;

        let else_branch = if self.match_token(&TokenKind::KwElse) {
            Some(self.parse_statement()?)
        } else {
            None
        };

        Ok(IfStmt::new(
            condition,
            then_branch,
            else_branch,
            start_pos.line,
            start_pos.column,
        ))
    }

    /// Парсит while: while (condition) statement
    fn parse_while_stmt(&mut self) -> ParseResult<WhileStmt> {
        let start_pos = self.current_position();

        self.consume(
            &TokenKind::KwWhile,
            ParseErrorKind::ExpectedToken,
            "ожидалось 'while'",
        )?;
        self.consume(
            &TokenKind::LParen,
            ParseErrorKind::MissingOpenParen,
            "ожидалось '(' после 'while'",
        )?;

        let condition = self.parse_expression()?;

        self.consume(
            &TokenKind::RParen,
            ParseErrorKind::MissingCloseParen,
            "ожидалось ')' после условия",
        )?;

        let body = self.parse_statement()?;

        Ok(WhileStmt::new(
            condition,
            body,
            start_pos.line,
            start_pos.column,
        ))
    }

    /// Парсит for: for (init; condition; update) statement
    fn parse_for_stmt(&mut self) -> ParseResult<ForStmt> {
        let start_pos = self.current_position();

        self.consume(
            &TokenKind::KwFor,
            ParseErrorKind::ExpectedToken,
            "ожидалось 'for'",
        )?;
        self.consume(
            &TokenKind::LParen,
            ParseErrorKind::MissingOpenParen,
            "ожидалось '(' после 'for'",
        )?;

        let init = if self.match_token(&TokenKind::Semicolon) {
            None
        } else if self.is_type_start() {
            Some(Statement::VariableDecl(self.parse_var_decl()?))
        } else {
            Some(Statement::Expression(self.parse_expr_stmt()?))
        };

        let condition = if !self.check(&TokenKind::Semicolon) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.consume(
            &TokenKind::Semicolon,
            ParseErrorKind::MissingSemicolon,
            "ожидалось ';' после условия",
        )?;

        let update = if !self.check(&TokenKind::RParen) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume(
            &TokenKind::RParen,
            ParseErrorKind::MissingCloseParen,
            "ожидалось ')' после заголовка for",
        )?;

        let body = self.parse_statement()?;

        Ok(ForStmt::new(
            init,
            condition,
            update,
            body,
            start_pos.line,
            start_pos.column,
        ))
    }

    /// Парсит return: return [expression];
    fn parse_return_stmt(&mut self) -> ParseResult<ReturnStmt> {
        let start_pos = self.current_position();

        self.consume(
            &TokenKind::KwReturn,
            ParseErrorKind::ExpectedToken,
            "ожидалось 'return'",
        )?;

        let value = if !self.check(&TokenKind::Semicolon) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume(
            &TokenKind::Semicolon,
            ParseErrorKind::MissingSemicolon,
            "ожидалось ';' после return",
        )?;

        Ok(ReturnStmt::new(value, start_pos.line, start_pos.column))
    }

    // === Парсинг выражений с приоритетами ===

    /// Парсит выражение (начинаем с самого низкого приоритета)
    pub fn parse_expression(&mut self) -> ParseResult<Expression> {
        self.parse_assignment()
    }

    /// Уровень 9: Присваивание (правоассоциативное)
    fn parse_assignment(&mut self) -> ParseResult<Expression> {
        let expr = self.parse_logical_or()?;

        if self.match_any(&[
            TokenKind::Eq,
            TokenKind::PlusEq,
            TokenKind::MinusEq,
            TokenKind::AsteriskEq,
            TokenKind::SlashEq,
        ]) {
            let operator = match self.previous().kind {
                TokenKind::Eq => AssignmentOp::Assign,
                TokenKind::PlusEq => AssignmentOp::AddAssign,
                TokenKind::MinusEq => AssignmentOp::SubAssign,
                TokenKind::AsteriskEq => AssignmentOp::MulAssign,
                TokenKind::SlashEq => AssignmentOp::DivAssign,
                _ => unreachable!(),
            };

            match &expr {
                Expression::Identifier(_) | Expression::StructAccess(_) => {}
                _ => {
                    return Err(ParseError::new(
                        self.current_position(),
                        ParseErrorKind::InvalidExpression,
                    )
                    .with_message("Недопустимая цель присваивания".to_string()));
                }
            }

            let value = self.parse_assignment()?;
            let pos = self.previous().position.clone();

            Ok(Expression::Assignment(AssignmentExpr::new(
                expr, operator, value, pos.line, pos.column,
            )))
        } else {
            Ok(expr)
        }
    }

    /// Уровень 8: Логическое ИЛИ (левоассоциативное)
    fn parse_logical_or(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_logical_and()?;

        while self.match_token(&TokenKind::PipePipe) {
            let operator = BinaryOp::Or;
            let right = self.parse_logical_and()?;
            let pos = self.previous().position.clone();

            expr = Expression::Binary(BinaryExpr::new(expr, operator, right, pos.line, pos.column));
        }

        Ok(expr)
    }

    /// Уровень 7: Логическое И (левоассоциативное)
    fn parse_logical_and(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_equality()?;

        while self.match_token(&TokenKind::AmpAmp) {
            let operator = BinaryOp::And;
            let right = self.parse_equality()?;
            let pos = self.previous().position.clone();

            expr = Expression::Binary(BinaryExpr::new(expr, operator, right, pos.line, pos.column));
        }

        Ok(expr)
    }

    /// Уровень 6: Равенство (неассоциативное)
    fn parse_equality(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_comparison()?;

        while self.match_any(&[TokenKind::EqEq, TokenKind::BangEq]) {
            let operator = match self.previous().kind {
                TokenKind::EqEq => BinaryOp::Eq,
                TokenKind::BangEq => BinaryOp::Ne,
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?;
            let pos = self.previous().position.clone();

            expr = Expression::Binary(BinaryExpr::new(expr, operator, right, pos.line, pos.column));
        }

        Ok(expr)
    }

    /// Уровень 5: Сравнение (неассоциативное)
    fn parse_comparison(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_additive()?;

        while self.match_any(&[
            TokenKind::Lt,
            TokenKind::LtEq,
            TokenKind::Gt,
            TokenKind::GtEq,
        ]) {
            let operator = match self.previous().kind {
                TokenKind::Lt => BinaryOp::Lt,
                TokenKind::LtEq => BinaryOp::Le,
                TokenKind::Gt => BinaryOp::Gt,
                TokenKind::GtEq => BinaryOp::Ge,
                _ => unreachable!(),
            };
            let right = self.parse_additive()?;
            let pos = self.previous().position.clone();

            expr = Expression::Binary(BinaryExpr::new(expr, operator, right, pos.line, pos.column));
        }

        Ok(expr)
    }

    /// Уровень 4: Сложение/вычитание (левоассоциативное)
    fn parse_additive(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_multiplicative()?;

        while self.match_any(&[TokenKind::Plus, TokenKind::Minus]) {
            let operator = match self.previous().kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };
            let right = self.parse_multiplicative()?;
            let pos = self.previous().position.clone();

            expr = Expression::Binary(BinaryExpr::new(expr, operator, right, pos.line, pos.column));
        }

        Ok(expr)
    }

    /// Уровень 3: Умножение/деление/остаток (левоассоциативное)
    fn parse_multiplicative(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_unary()?;

        while self.match_any(&[TokenKind::Asterisk, TokenKind::Slash, TokenKind::Percent]) {
            let operator = match self.previous().kind {
                TokenKind::Asterisk => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Percent => BinaryOp::Mod,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            let pos = self.previous().position.clone();

            expr = Expression::Binary(BinaryExpr::new(expr, operator, right, pos.line, pos.column));
        }

        Ok(expr)
    }

    /// Уровень 2: Унарные операторы (правоассоциативные)
    fn parse_unary(&mut self) -> ParseResult<Expression> {
        if self.match_any(&[TokenKind::Minus, TokenKind::Bang, TokenKind::Plus]) {
            let operator = match self.previous().kind {
                TokenKind::Minus => UnaryOp::Neg,
                TokenKind::Bang => UnaryOp::Not,
                TokenKind::Plus => UnaryOp::Plus,
                _ => unreachable!(),
            };
            let pos = self.previous().position.clone();
            let expr = self.parse_unary()?;

            Ok(Expression::Unary(UnaryExpr::new(
                operator, expr, pos.line, pos.column,
            )))
        } else {
            let expr = match self.parse_primary() {
                Ok(expr) => expr,
                Err(e) => {
                    return Err(e);
                }
            };
            Ok(expr)
        }
    }

    /// Уровень 1: Первичные выражения (высший приоритет)
    fn parse_primary(&mut self) -> ParseResult<Expression> {
        if self.is_at_end() {
            return Err(ParseError::new(
                self.current_position(),
                ParseErrorKind::UnexpectedEOF,
            ));
        }

        let token = self.peek().clone();
        let pos = token.position.clone();

        match &token.kind {
            TokenKind::IntLiteral(value) => {
                self.advance();
                Ok(Expression::Literal(Literal::new(
                    LiteralValue::Int(*value),
                    pos.line,
                    pos.column,
                )))
            }

            TokenKind::FloatLiteral(value) => {
                self.advance();
                Ok(Expression::Literal(Literal::new(
                    LiteralValue::Float(*value),
                    pos.line,
                    pos.column,
                )))
            }

            TokenKind::StringLiteral(value) => {
                self.advance();
                Ok(Expression::Literal(Literal::new(
                    LiteralValue::String(value.clone()),
                    pos.line,
                    pos.column,
                )))
            }

            TokenKind::KwTrue => {
                self.advance();
                let expr = Expression::Literal(Literal::new(
                    LiteralValue::Bool(true),
                    pos.line,
                    pos.column,
                ));
                Ok(expr)
            }

            TokenKind::KwFalse => {
                self.advance();
                let expr = Expression::Literal(Literal::new(
                    LiteralValue::Bool(false),
                    pos.line,
                    pos.column,
                ));
                Ok(expr)
            }

            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();

                if self.match_token(&TokenKind::LParen) {
                    self.parse_call(
                        Expression::Identifier(IdentifierExpr::new(name, pos.line, pos.column)),
                        pos,
                    )
                } else if self.match_token(&TokenKind::Dot) {
                    self.parse_struct_access(
                        Expression::Identifier(IdentifierExpr::new(name, pos.line, pos.column)),
                        pos,
                    )
                } else {
                    Ok(Expression::Identifier(IdentifierExpr::new(
                        name, pos.line, pos.column,
                    )))
                }
            }

            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                let pos = self.current_position();
                self.consume(
                    &TokenKind::RParen,
                    ParseErrorKind::MissingCloseParen,
                    "ожидалось ')' после выражения",
                )?;
                Ok(Expression::Grouped(GroupedExpr::new(
                    expr, pos.line, pos.column,
                )))
            }

            _ => Err(
                ParseError::new(token.position, ParseErrorKind::UnexpectedToken)
                    .with_found(token.lexeme),
            ),
        }
    }

    /// Парсит доступ к полю структуры: object.field [.field...]
    fn parse_struct_access(
        &mut self,
        object: Expression,
        pos: Position,
    ) -> ParseResult<Expression> {
        let mut current_obj = object;

        loop {
            // Потребляем поле
            let field = match self.advance() {
                token if matches!(token.kind, TokenKind::Identifier(_)) => token.lexeme.clone(),
                token => {
                    return Err(ParseError::new(
                        token.position.clone(),
                        ParseErrorKind::ExpectedToken,
                    )
                    .with_found(token.lexeme.clone()));
                }
            };
            let field_pos = self.previous().position.clone();

            current_obj = Expression::StructAccess(StructAccessExpr::new(
                current_obj,
                field,
                field_pos.line,
                field_pos.column,
            ));

            if !self.match_token(&TokenKind::Dot) {
                break;
            }
        }

        if self.match_token(&TokenKind::LParen) {
            self.parse_call(current_obj, pos)
        } else {
            Ok(current_obj)
        }
    }

    /// Парсит вызов функции: callee(arguments)
    fn parse_call(&mut self, callee: Expression, pos: Position) -> ParseResult<Expression> {
        let mut arguments = Vec::new();

        if !self.check(&TokenKind::RParen) {
            loop {
                arguments.push(self.parse_expression()?);

                if !self.match_token(&TokenKind::Comma) {
                    break;
                }

                if self.check(&TokenKind::RParen) {
                    break;
                }
            }
        }

        self.consume(
            &TokenKind::RParen,
            ParseErrorKind::MissingCloseParen,
            "ожидалось ')' после аргументов",
        )?;

        let call_expr = Expression::Call(CallExpr::new(callee, arguments, pos.line, pos.column));

        if self.match_token(&TokenKind::Dot) {
            self.parse_struct_access(call_expr, pos)
        } else {
            Ok(call_expr)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::position::Position;
    use crate::common::token::{Token, TokenKind};

    #[allow(dead_code)]
    fn create_token(kind: TokenKind, lexeme: &str, line: usize, column: usize) -> Token {
        Token::new(kind, lexeme.to_string(), Position::new(line, column))
    }

    #[test]
    fn test_parse_empty_program() {
        let tokens = vec![Token::eof(Position::new(1, 1))];
        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        assert!(ast.is_some());
        assert!(parser.errors().is_empty());
    }

    #[test]
    fn test_parse_simple_expression() {
        let tokens = vec![
            Token::new(
                TokenKind::IntLiteral(42),
                "42".to_string(),
                Position::new(1, 1),
            ),
            Token::eof(Position::new(1, 3)),
        ];
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expression();
        assert!(expr.is_ok());
    }

    #[test]
    fn test_parse_simple_function() {
        let tokens = vec![
            Token::new(TokenKind::KwFn, "fn".to_string(), Position::new(1, 1)),
            Token::new(
                TokenKind::Identifier("main".to_string()),
                "main".to_string(),
                Position::new(1, 4),
            ),
            Token::new(TokenKind::LParen, "(".to_string(), Position::new(1, 8)),
            Token::new(TokenKind::RParen, ")".to_string(), Position::new(1, 9)),
            Token::new(TokenKind::LBrace, "{".to_string(), Position::new(1, 11)),
            Token::new(
                TokenKind::KwReturn,
                "return".to_string(),
                Position::new(2, 5),
            ),
            Token::new(
                TokenKind::IntLiteral(42),
                "42".to_string(),
                Position::new(2, 12),
            ),
            Token::new(TokenKind::Semicolon, ";".to_string(), Position::new(2, 14)),
            Token::new(TokenKind::RBrace, "}".to_string(), Position::new(3, 1)),
            Token::eof(Position::new(3, 2)),
        ];

        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        assert!(ast.is_some());
        assert!(parser.errors().is_empty());
    }
}
