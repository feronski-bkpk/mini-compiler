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
    /// Глубина рекурсии (для предотвращения переполнения стека)
    recursion_depth: usize,
    /// Максимальная глубина рекурсии
    max_recursion_depth: usize,
    /// Счетчик попыток восстановления
    recovery_attempts: usize,
}

impl Parser {
    /// Создает новый парсер из списка токенов
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            errors: ParseErrors::new(),
            in_error_recovery: false,
            recursion_depth: 0,
            max_recursion_depth: 1000,
            recovery_attempts: 0,
        }
    }

    /// Устанавливает максимальное количество ошибок
    pub fn with_max_errors(mut self, max: usize) -> Self {
        self.errors = self.errors.with_max_errors(max);
        self
    }

    /// Устанавливает максимальную глубину рекурсии
    pub fn with_max_recursion_depth(mut self, depth: usize) -> Self {
        self.max_recursion_depth = depth;
        self
    }

    /// Возвращает собранные ошибки
    pub fn errors(&self) -> &ParseErrors {
        &self.errors
    }

    /// Возвращает метрики ошибок
    pub fn error_metrics(&self) -> &crate::parser::error::ErrorMetrics {
        &self.errors.metrics
    }

    /// Добавляет ошибку
    pub fn add_error(&mut self, error: ParseError) {
        self.errors.add(error);
    }

    /// Основной метод разбора - парсит всю программу
    pub fn parse(&mut self) -> Option<Program> {
        let start_pos = self.current_position();
        let mut declarations = Vec::new();

        while !self.is_at_end() && !self.errors.reached_limit() {
            if matches!(self.peek().kind, TokenKind::RBrace) {
                let pos = self.current_position();
                let suggestion =
                    "Проверьте, нет ли лишней закрывающей скобки или не хватает открывающей";

                self.errors.add(
                    ParseError::new(pos, ParseErrorKind::UnexpectedToken)
                        .with_found(self.peek().lexeme.clone())
                        .with_message(
                            "неожиданная закрывающая скобка на верхнем уровне".to_string(),
                        )
                        .with_suggestion(suggestion.to_string()),
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

                    if !self.advanced_synchronize() {
                        break;
                    }
                }
            }
        }

        if declarations.is_empty()
            && self.errors.has_fatal()
            && self.errors.metrics.actual_errors > 0
        {
            None
        } else {
            Some(Program::new(declarations, start_pos.line, start_pos.column))
        }
    }

    /// Возвращает текущий токен без потребления
    pub fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    /// Возвращает предыдущий токен
    pub fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    /// Проверяет, достигнут ли конец файла
    pub fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::EndOfFile
    }

    /// Возвращает текущую позицию
    pub fn current_position(&self) -> Position {
        self.peek().position.clone()
    }

    /// Проверяет, совпадает ли текущий токен с ожидаемым типом
    pub fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
        }
    }

    /// Проверяет, совпадает ли текущий токен с любым из ожидаемых типов
    pub fn check_any(&self, kinds: &[TokenKind]) -> bool {
        kinds.iter().any(|kind| self.check(kind))
    }

    /// Потребляет текущий токен, если он совпадает с ожидаемым типом
    pub fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Потребляет текущий токен, если он совпадает с любым из ожидаемых типов
    pub fn match_any(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    /// Переходит к следующему токену
    pub fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    /// Потребляет текущий токен или сообщает об ошибке
    pub fn consume(
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

            if self.try_phrase_level_recovery(kind, &found) {
                Ok(self.previous())
            } else {
                let suggestion = match error_kind {
                    ParseErrorKind::MissingSemicolon => {
                        Some("Попробуйте добавить ';' в конце инструкции".to_string())
                    }
                    ParseErrorKind::MissingOpenParen => Some("Попробуйте добавить '('".to_string()),
                    ParseErrorKind::MissingCloseParen => {
                        Some("Попробуйте добавить ')'".to_string())
                    }
                    ParseErrorKind::MissingOpenBrace => {
                        Some("Попробуйте добавить '{{' в начале блока".to_string())
                    }
                    ParseErrorKind::MissingCloseBrace => {
                        Some("Попробуйте добавить '}}' в конце блока".to_string())
                    }
                    _ => None,
                };

                Err(ParseError::new(pos, error_kind)
                    .with_found(found)
                    .with_message(message.to_string())
                    .with_suggestion(suggestion.unwrap_or_default()))
            }
        }
    }

    /// Пытается восстановиться на уровне фразы (вставка/удаление токенов)
    fn try_phrase_level_recovery(&mut self, expected_kind: &TokenKind, found: &str) -> bool {
        self.recovery_attempts += 1;

        if self.recovery_attempts > 10 {
            return false;
        }

        match expected_kind {
            TokenKind::Semicolon => {
                if !self.is_at_end() {
                    let next = self.peek();
                    match &next.kind {
                        TokenKind::KwIf
                        | TokenKind::KwWhile
                        | TokenKind::KwFor
                        | TokenKind::KwReturn
                        | TokenKind::KwFn
                        | TokenKind::KwStruct
                        | TokenKind::RBrace => {
                            self.errors.metrics.mark_recovered();
                            return true;
                        }
                        _ => {}
                    }
                }
                false
            }

            TokenKind::RParen => {
                if found == ")" {
                    self.advance();
                    self.errors.metrics.mark_recovered();
                    true
                } else if matches!(self.peek().kind, TokenKind::LBrace | TokenKind::Semicolon) {
                    self.errors.metrics.mark_recovered();
                    true
                } else {
                    false
                }
            }

            TokenKind::RBrace => {
                if matches!(self.peek().kind, TokenKind::EndOfFile) {
                    self.errors.metrics.mark_recovered();
                    true
                } else {
                    false
                }
            }

            _ => false,
        }
    }

    /// Продвинутая синхронизация с несколькими стратегиями
    fn advanced_synchronize(&mut self) -> bool {
        self.in_error_recovery = true;
        self.recovery_attempts = 0;

        let mut synchronized = false;
        let mut tokens_skipped = 0;
        let max_skip = 20;

        while !self.is_at_end() && tokens_skipped < max_skip {
            if self.is_sync_point() {
                synchronized = true;
                break;
            }
            self.advance();
            tokens_skipped += 1;
        }

        if !synchronized && !self.is_at_end() {
            synchronized = self.try_insert_missing_tokens();
        }

        self.in_error_recovery = false;

        if synchronized {
            self.errors.metrics.mark_recovered();
        }

        synchronized && !self.is_at_end()
    }

    /// Проверяет, является ли текущая позиция точкой синхронизации
    pub fn is_sync_point(&self) -> bool {
        if self.is_at_end() {
            return true;
        }

        match &self.peek().kind {
            TokenKind::Semicolon => true,

            TokenKind::KwFn | TokenKind::KwStruct => true,

            TokenKind::KwIf
            | TokenKind::KwWhile
            | TokenKind::KwFor
            | TokenKind::KwReturn
            | TokenKind::LBrace
            | TokenKind::RBrace => true,

            TokenKind::RParen | TokenKind::Comma => true,

            _ => false,
        }
    }

    /// Пытается вставить недостающие токены для восстановления
    fn try_insert_missing_tokens(&mut self) -> bool {
        let current = self.peek().clone();

        match &current.kind {
            TokenKind::RBrace => {
                self.advance();
                true
            }

            TokenKind::RParen => {
                if self.current + 1 < self.tokens.len() {
                    let next = &self.tokens[self.current + 1];
                    match &next.kind {
                        TokenKind::Semicolon | TokenKind::RParen | TokenKind::Comma => {
                            self.advance();
                            return true;
                        }
                        _ => {}
                    }
                }
                false
            }

            kind if Self::is_operator(kind) => true,

            _ => false,
        }
    }

    /// Проверяет, является ли токен оператором
    fn is_operator(kind: &TokenKind) -> bool {
        matches!(
            kind,
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
                | TokenKind::Eq
                | TokenKind::PlusEq
                | TokenKind::MinusEq
                | TokenKind::AsteriskEq
                | TokenKind::SlashEq
        )
    }

    /// Парсит объявление верхнего уровня
    pub fn parse_declaration(&mut self) -> ParseResult<Declaration> {
        self.recursion_depth += 1;
        if self.recursion_depth > self.max_recursion_depth {
            self.recursion_depth -= 1;
            return Err(
                ParseError::new(self.current_position(), ParseErrorKind::SyntaxError)
                    .with_message("Превышена максимальная глубина рекурсии".to_string()),
            );
        }

        let result = match &self.peek().kind {
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
        };

        self.recursion_depth -= 1;
        result
    }

    /// Проверяет, начинается ли токен с типа
    pub fn is_type_start(&self) -> bool {
        matches!(
            &self.peek().kind,
            TokenKind::KwInt
                | TokenKind::KwFloat
                | TokenKind::KwBool
                | TokenKind::KwVoid
                | TokenKind::KwString
                | TokenKind::KwStruct
        ) || {
            if let TokenKind::Identifier(name) = &self.peek().kind {
                name == "var"
            } else {
                false
            }
        }
    }

    /// Парсит объявление функции: fn name(params) -> Type { ... }
    pub fn parse_function_decl(&mut self) -> ParseResult<FunctionDecl> {
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
                        .with_found(token.lexeme.clone())
                        .with_suggestion("Имя функции должно быть идентификатором".to_string()),
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
                    .with_message("Функция main не может иметь параметров".to_string())
                    .with_suggestion("Удалите параметры у функции main".to_string()),
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

        let body_result = self.parse_block();

        match body_result {
            Ok(body) => Ok(FunctionDecl::new(
                name,
                return_type,
                parameters,
                body,
                start_pos.line,
                start_pos.column,
            )),
            Err(e) => {
                self.errors.add(e);

                let dummy_body = BlockStmt::new(Vec::new(), start_pos.line, start_pos.column);
                Ok(FunctionDecl::new(
                    name,
                    return_type,
                    parameters,
                    dummy_body,
                    start_pos.line,
                    start_pos.column,
                ))
            }
        }
    }

    /// Парсит список параметров функции
    pub fn parse_param_list(&mut self) -> ParseResult<Vec<Param>> {
        let mut params = Vec::new();

        if self.check(&TokenKind::RParen) {
            return Ok(params);
        }

        match self.parse_param() {
            Ok(param) => params.push(param),
            Err(e) => {
                self.errors.add(e);
                if !self.advanced_synchronize() {
                    return Ok(params);
                }
            }
        }

        while self.match_token(&TokenKind::Comma) {
            if self.check(&TokenKind::RParen) {
                break;
            }

            match self.parse_param() {
                Ok(param) => params.push(param),
                Err(e) => {
                    self.errors.add(e);
                    while !self.is_at_end()
                        && !self.check(&TokenKind::Comma)
                        && !self.check(&TokenKind::RParen)
                    {
                        self.advance();
                    }
                }
            }
        }

        Ok(params)
    }

    /// Парсит один параметр: Type name
    pub fn parse_param(&mut self) -> ParseResult<Param> {
        let start_pos = self.current_position();
        let param_type = self.parse_type()?;

        let name = match self.advance() {
            token if matches!(token.kind, TokenKind::Identifier(_)) => token.lexeme.clone(),
            token => {
                return Err(
                    ParseError::new(token.position.clone(), ParseErrorKind::ExpectedToken)
                        .with_found(token.lexeme.clone())
                        .with_suggestion("Имя параметра должно быть идентификатором".to_string()),
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
    pub fn parse_struct_decl(&mut self) -> ParseResult<StructDecl> {
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
                        .with_found(token.lexeme.clone())
                        .with_suggestion("Имя структуры должно быть идентификатором".to_string()),
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
            match self.parse_var_decl() {
                Ok(field) => fields.push(field),
                Err(e) => {
                    self.errors.add(e);
                    while !self.is_at_end()
                        && !self.check(&TokenKind::RBrace)
                        && !self.check(&TokenKind::KwStruct)
                    {
                        self.advance();
                    }
                }
            }
        }

        self.consume(
            &TokenKind::RBrace,
            ParseErrorKind::MissingCloseBrace,
            "ожидалось '}}' в конце структуры",
        )?;

        if self.check(&TokenKind::Semicolon) {
            self.advance();
        }

        Ok(StructDecl::new(
            name,
            fields,
            start_pos.line,
            start_pos.column,
        ))
    }

    /// Парсит объявление переменной: Type name [= Expression];
    pub fn parse_var_decl(&mut self) -> ParseResult<VarDecl> {
        let start_pos = self.current_position();

        let var_type = self.parse_type()?;

                let name = match self.advance() {
            token if matches!(token.kind, TokenKind::Identifier(_)) => token.lexeme.clone(),
            token => {
                return Err(
                    ParseError::new(token.position.clone(), ParseErrorKind::ExpectedToken)
                        .with_found(token.lexeme.clone())
                        .with_suggestion("Имя переменной должно быть идентификатором".to_string()),
                );
            }
        };

        if self.check(&TokenKind::LBracket) {
            self.advance();
            if let TokenKind::IntLiteral(_) = self.peek().kind {
                self.advance();
            }
            self.consume(
                &TokenKind::RBracket,
                ParseErrorKind::MissingCloseParen,
                "ожидалось ']' после размера массива",
            )?;
        }

        let initializer = if self.match_token(&TokenKind::Eq) {
            let expr = self.parse_expression()?;
            Some(expr)
        } else {
            None
        };

        if self.check(&TokenKind::Semicolon) {
            self.advance();
            Ok(VarDecl::new(
                var_type,
                name,
                initializer,
                start_pos.line,
                start_pos.column,
            ))
        } else if self.check(&TokenKind::RBrace) && initializer.is_none() {
            Ok(VarDecl::new(
                var_type,
                name,
                initializer,
                start_pos.line,
                start_pos.column,
            ))
        } else {
            let pos = self.current_position();
            let error = ParseError::new(pos.clone(), ParseErrorKind::MissingSemicolon)
                .with_found(self.peek().lexeme.clone())
                .with_suggestion("Добавьте ';' в конце объявления переменной".to_string());
            self.errors.add(error);
            self.errors.metrics.mark_recovered();

            Ok(VarDecl::new(
                var_type,
                name,
                initializer,
                start_pos.line,
                start_pos.column,
            ))
        }
    }

    /// Парсит тип
    pub fn parse_type(&mut self) -> ParseResult<Type> {
        let token = self.peek().clone();
        let pos = token.position.clone();

        let mut typ = match &token.kind {
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
            TokenKind::Identifier(name) if name == "var" => {
                self.advance();
                Type::Inferred
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
                        .with_found(token.lexeme.clone())
                        .with_suggestion(
                            "После 'struct' должно следовать имя структуры".to_string(),
                        ));
                    }
                }
            }
            _ => {
                return Err(
                    ParseError::new(pos, ParseErrorKind::UnknownType)
                        .with_found(token.lexeme)
                        .with_suggestion("Используйте один из встроенных типов: int, float, bool, void, string, struct Имя, или var".to_string()),
                );
            }
        };

        while self.match_token(&TokenKind::LBracket) {
            let size = if let TokenKind::IntLiteral(s) = self.peek().kind {
                self.advance();
                Some(s)
            } else {
                None
            };
            self.consume(
                &TokenKind::RBracket,
                ParseErrorKind::MissingCloseParen,
                "ожидалось ']'",
            )?;
            typ = Type::Array(Box::new(typ), size);
        }

        Ok(typ)
    }

    /// Парсит инструкцию
    pub fn parse_statement(&mut self) -> ParseResult<Statement> {
        self.recursion_depth += 1;
        if self.recursion_depth > self.max_recursion_depth {
            self.recursion_depth -= 1;
            return Err(
                ParseError::new(self.current_position(), ParseErrorKind::SyntaxError)
                    .with_message("Превышена максимальная глубина рекурсии".to_string()),
            );
        }

        let result = match &self.peek().kind {
            TokenKind::KwIf => Ok(Statement::If(self.parse_if_stmt()?)),
            TokenKind::KwWhile => Ok(Statement::While(self.parse_while_stmt()?)),
            TokenKind::KwFor => Ok(Statement::For(self.parse_for_stmt()?)),
            TokenKind::KwReturn => Ok(Statement::Return(self.parse_return_stmt()?)),
            TokenKind::LBrace => Ok(Statement::Block(self.parse_block()?)),
            TokenKind::KwBreak => {
                self.advance();
                let pos = self.previous().position.clone();
                if self.check(&TokenKind::Semicolon) {
                    self.advance();
                }
                Ok(Statement::Break(BreakStmt::new(pos.line, pos.column)))
            }
            TokenKind::KwContinue => {
                self.advance();
                let pos = self.previous().position.clone();
                if self.check(&TokenKind::Semicolon) {
                    self.advance();
                }
                Ok(Statement::Continue(ContinueStmt::new(pos.line, pos.column)))
            }
            TokenKind::KwSwitch => Ok(Statement::Switch(self.parse_switch_stmt()?)),
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
        };

        self.recursion_depth -= 1;
        result
    }

    /// Парсит инструкцию-выражение: Expression;
    pub fn parse_expr_stmt(&mut self) -> ParseResult<ExprStmt> {
        let start_pos = self.current_position();
        let expr = self.parse_expression()?;

        if self.check(&TokenKind::Semicolon) {
            self.advance();
            Ok(ExprStmt::new(expr, start_pos.line, start_pos.column))
        } else if self.check(&TokenKind::RBrace) || self.is_at_end() {
            Ok(ExprStmt::new(expr, start_pos.line, start_pos.column))
        } else {
            let pos = self.current_position();
            let error = ParseError::new(pos.clone(), ParseErrorKind::MissingSemicolon)
                .with_found(self.peek().lexeme.clone())
                .with_suggestion("Добавьте ';' в конце инструкции".to_string());
            self.errors.add(error);
            self.errors.metrics.mark_recovered();

            Ok(ExprStmt::new(expr, start_pos.line, start_pos.column))
        }
    }

    /// Парсит блок: { statements }
    pub fn parse_block(&mut self) -> ParseResult<BlockStmt> {
        self.recursion_depth += 1;
        if self.recursion_depth > self.max_recursion_depth {
            self.recursion_depth -= 1;
            return Err(
                ParseError::new(self.current_position(), ParseErrorKind::SyntaxError).with_message(
                    "Превышена максимальная глубина рекурсии при разборе блока".to_string(),
                ),
            );
        }

        let start_pos = self.current_position();

        if !self.check(&TokenKind::LBrace) {
            let pos = self.current_position();
            let found = self.peek().lexeme.clone();
            let error = ParseError::new(pos, ParseErrorKind::MissingOpenBrace)
                .with_found(found)
                .with_suggestion("Добавьте '{{' в начале блока".to_string());

            self.recursion_depth -= 1;
            return Err(error);
        }
        self.advance();

        let mut statements = Vec::new();
        let mut iteration_count = 0;
        let max_iterations = 10000;

        while !self.check(&TokenKind::RBrace)
            && !self.is_at_end()
            && iteration_count < max_iterations
        {
            iteration_count += 1;

            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    self.errors.add(e);
                    if !self.advanced_synchronize() {
                        break;
                    }
                }
            }
        }

        if iteration_count >= max_iterations {
            self.recursion_depth -= 1;
            return Err(
                ParseError::new(self.current_position(), ParseErrorKind::SyntaxError).with_message(
                    "Превышено максимальное количество инструкций в блоке".to_string(),
                ),
            );
        }

        if self.check(&TokenKind::RBrace) {
            self.advance();
            self.recursion_depth -= 1;
            Ok(BlockStmt::new(statements, start_pos.line, start_pos.column))
        } else {
            let pos = self.current_position();
            let found = if self.is_at_end() {
                "конец файла".to_string()
            } else {
                self.peek().lexeme.clone()
            };

            let error = ParseError::new(pos.clone(), ParseErrorKind::MissingCloseBrace)
                .with_found(found)
                .with_suggestion("Добавьте '}}' в конце блока".to_string());

            self.errors.add(error);
            self.errors.metrics.mark_recovered();

            self.recursion_depth -= 1;
            Ok(BlockStmt::new(statements, start_pos.line, start_pos.column))
        }
    }

    /// Парсит if-else: if (condition) statement [else statement]
    pub fn parse_if_stmt(&mut self) -> ParseResult<IfStmt> {
        let start_pos = self.current_position();

        self.consume(
            &TokenKind::KwIf,
            ParseErrorKind::ExpectedToken,
            "ожидалось 'if'",
        )?;

        let has_open_paren = self.check(&TokenKind::LParen);

        if has_open_paren {
            self.advance();
        }

        let condition = self.parse_expression()?;

        if self.check(&TokenKind::RParen) {
            self.advance();
        } else if has_open_paren {
            let pos = self.current_position();
            let error = ParseError::new(pos, ParseErrorKind::MissingCloseParen)
                .with_found(self.peek().lexeme.clone())
                .with_suggestion("Добавьте ')' после условия".to_string());
            self.errors.add(error);
        }

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
    pub fn parse_while_stmt(&mut self) -> ParseResult<WhileStmt> {
        let start_pos = self.current_position();

        self.consume(
            &TokenKind::KwWhile,
            ParseErrorKind::ExpectedToken,
            "ожидалось 'while'",
        )?;

        let has_paren = self.check(&TokenKind::LParen);
        if has_paren {
            self.advance();
        }

        let condition = self.parse_expression()?;

        if has_paren && self.check(&TokenKind::RParen) {
            self.advance();
        }

        let body = self.parse_statement()?;

        Ok(WhileStmt::new(
            condition,
            body,
            start_pos.line,
            start_pos.column,
        ))
    }

    /// Парсит for: for (init; condition; update) statement
    pub fn parse_for_stmt(&mut self) -> ParseResult<ForStmt> {
        let start_pos = self.current_position();

        self.consume(
            &TokenKind::KwFor,
            ParseErrorKind::ExpectedToken,
            "ожидалось 'for'",
        )?;

        if !self.check(&TokenKind::LParen) {
            let pos = self.current_position();
            let error = ParseError::new(pos, ParseErrorKind::MissingOpenParen)
                .with_found(self.peek().lexeme.clone())
                .with_suggestion("Добавьте '(' после 'for'".to_string());
            self.errors.add(error);
        } else {
            self.advance();
        }

        let init = if self.check(&TokenKind::Semicolon) {
            self.advance();
            None
        } else if self.is_type_start() {
            match self.parse_var_decl() {
                Ok(var_decl) => Some(Statement::VariableDecl(var_decl)),
                Err(e) => {
                    self.errors.add(e);
                    while !self.is_at_end() && !self.check(&TokenKind::Semicolon) {
                        self.advance();
                    }
                    if self.check(&TokenKind::Semicolon) {
                        self.advance();
                    }
                    None
                }
            }
        } else {
            match self.parse_expr_stmt() {
                Ok(expr_stmt) => Some(Statement::Expression(expr_stmt)),
                Err(e) => {
                    self.errors.add(e);
                    while !self.is_at_end() && !self.check(&TokenKind::Semicolon) {
                        self.advance();
                    }
                    if self.check(&TokenKind::Semicolon) {
                        self.advance();
                    }
                    None
                }
            }
        };

        let condition = if !self.check(&TokenKind::Semicolon) && !self.check(&TokenKind::RParen) {
            match self.parse_expression() {
                Ok(expr) => {
                    if !self.check(&TokenKind::Semicolon) {
                        let pos = self.current_position();
                        let error = ParseError::new(pos, ParseErrorKind::MissingSemicolon)
                            .with_found(self.peek().lexeme.clone())
                            .with_suggestion("Добавьте ';' после условия".to_string());
                        self.errors.add(error);
                    } else {
                        self.advance();
                    }
                    Some(expr)
                }
                Err(e) => {
                    self.errors.add(e);
                    while !self.is_at_end() && !self.check(&TokenKind::Semicolon) {
                        self.advance();
                    }
                    if self.check(&TokenKind::Semicolon) {
                        self.advance();
                    }
                    None
                }
            }
        } else {
            if self.check(&TokenKind::Semicolon) {
                self.advance();
            }
            None
        };

        let update = if !self.check(&TokenKind::RParen) && !self.is_at_end() {
            match self.parse_expression() {
                Ok(expr) => Some(expr),
                Err(e) => {
                    self.errors.add(e);
                    None
                }
            }
        } else {
            None
        };

        if self.check(&TokenKind::RParen) {
            self.advance();
        } else {
            let pos = self.current_position();
            let error = ParseError::new(pos, ParseErrorKind::MissingCloseParen)
                .with_found(self.peek().lexeme.clone())
                .with_suggestion("Добавьте ')' после заголовка for".to_string());
            self.errors.add(error);
        }

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
    pub fn parse_return_stmt(&mut self) -> ParseResult<ReturnStmt> {
        let start_pos = self.current_position();

        self.consume(
            &TokenKind::KwReturn,
            ParseErrorKind::ExpectedToken,
            "ожидалось 'return'",
        )?;

        let value = if !self.check(&TokenKind::Semicolon) && !self.check(&TokenKind::RBrace) {
            match self.parse_expression() {
                Ok(expr) => Some(expr),
                Err(e) => {
                    self.errors.add(e);
                    None
                }
            }
        } else {
            None
        };

        if self.check(&TokenKind::Semicolon) {
            self.advance();
            Ok(ReturnStmt::new(value, start_pos.line, start_pos.column))
        } else {
            if self.check(&TokenKind::RBrace) && value.is_some() {
                let pos = self.current_position();
                let error = ParseError::new(pos.clone(), ParseErrorKind::MissingSemicolon)
                    .with_found(self.peek().lexeme.clone())
                    .with_suggestion("Добавьте ';' перед закрывающей скобкой".to_string());
                self.errors.add(error);
                self.errors.metrics.mark_recovered();
            }
            Ok(ReturnStmt::new(value, start_pos.line, start_pos.column))
        }
    }

    /// Парсит выражение (начинаем с самого низкого приоритета)
    pub fn parse_expression(&mut self) -> ParseResult<Expression> {
        let result = self.parse_assignment();
        result
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
                    .with_message("Недопустимая цель присваивания".to_string())
                    .with_suggestion(
                        "Целью присваивания должна быть переменная или поле структуры".to_string(),
                    ));
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
            let op_pos = self.previous().position.clone();
            let right = self.parse_additive()?;

            expr = Expression::Binary(BinaryExpr::new(
                expr,
                operator,
                right,
                op_pos.line,
                op_pos.column,
            ));
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
            let op_pos = self.previous().position.clone();
            let right = self.parse_multiplicative()?;

            expr = Expression::Binary(BinaryExpr::new(
                expr,
                operator,
                right,
                op_pos.line,
                op_pos.column,
            ));
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
            let op_pos = self.previous().position.clone();
            let right = self.parse_unary()?;

            expr = Expression::Binary(BinaryExpr::new(
                expr,
                operator,
                right,
                op_pos.line,
                op_pos.column,
            ));
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
        } else if self.match_any(&[TokenKind::PlusPlus, TokenKind::MinusMinus]) {
            let operator = match self.previous().kind {
                TokenKind::PlusPlus => UnaryOp::PreIncrement,
                TokenKind::MinusMinus => UnaryOp::PreDecrement,
                _ => unreachable!(),
            };
            let pos = self.previous().position.clone();
            let expr = self.parse_unary()?;

            Ok(Expression::Unary(UnaryExpr::new(
                operator, expr, pos.line, pos.column,
            )))
        } else {
            let mut expr = self.parse_primary()?;

            loop {
                if self.match_token(&TokenKind::PlusPlus) {
                    let op_pos = self.previous().position.clone();
                    expr = Expression::Unary(UnaryExpr::new(
                        UnaryOp::PostIncrement,
                        expr,
                        op_pos.line,
                        op_pos.column,
                    ));
                } else if self.match_token(&TokenKind::MinusMinus) {
                    let op_pos = self.previous().position.clone();
                    expr = Expression::Unary(UnaryExpr::new(
                        UnaryOp::PostDecrement,
                        expr,
                        op_pos.line,
                        op_pos.column,
                    ));
                } else {
                    break;
                }
            }

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
                Ok(Expression::Literal(Literal::new(
                    LiteralValue::Bool(true),
                    pos.line,
                    pos.column,
                )))
            }

            TokenKind::KwFalse => {
                self.advance();
                Ok(Expression::Literal(Literal::new(
                    LiteralValue::Bool(false),
                    pos.line,
                    pos.column,
                )))
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
                if !self.check(&TokenKind::RParen) {
                    let pos = self.current_position();
                    let error = ParseError::new(pos, ParseErrorKind::MissingCloseParen)
                        .with_found(self.peek().lexeme.clone())
                        .with_suggestion("Добавьте ')' после выражения".to_string());
                    self.errors.add(error);
                } else {
                    self.advance();
                }
                Ok(Expression::Grouped(GroupedExpr::new(
                    expr, pos.line, pos.column,
                )))
            }

            _ => {
                let suggestion = match &token.kind {
                    TokenKind::RBrace => "Неожиданная '}', возможно, лишняя скобка".to_string(),
                    TokenKind::RParen => "Неожиданная ')', проверьте парные скобки".to_string(),
                    _ => format!("Ожидалось выражение, найдено: {}", token.lexeme),
                };

                Err(
                    ParseError::new(token.position, ParseErrorKind::UnexpectedToken)
                        .with_found(token.lexeme)
                        .with_suggestion(suggestion),
                )
            }
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
            let field = match self.advance() {
                token if matches!(token.kind, TokenKind::Identifier(_)) => token.lexeme.clone(),
                token => {
                    return Err(ParseError::new(
                        token.position.clone(),
                        ParseErrorKind::ExpectedToken,
                    )
                    .with_found(token.lexeme.clone())
                    .with_suggestion("После '.' должно следовать имя поля".to_string()));
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
                match self.parse_expression() {
                    Ok(arg) => arguments.push(arg),
                    Err(e) => {
                        self.errors.add(e);
                        while !self.is_at_end()
                            && !self.check(&TokenKind::Comma)
                            && !self.check(&TokenKind::RParen)
                        {
                            self.advance();
                        }
                    }
                }

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

    /// Парсит switch: switch (expression) { case literal: statement; ... [default: statement;] }
    pub fn parse_switch_stmt(&mut self) -> ParseResult<SwitchStmt> {
        let start_pos = self.current_position();

        self.consume(
            &TokenKind::KwSwitch,
            ParseErrorKind::ExpectedToken,
            "ожидалось 'switch'",
        )?;

        let has_paren = self.check(&TokenKind::LParen);
        if has_paren {
            self.advance();
        }

        let expression = self.parse_expression()?;

        if has_paren && self.check(&TokenKind::RParen) {
            self.advance();
        }

        self.consume(
            &TokenKind::LBrace,
            ParseErrorKind::MissingOpenBrace,
            "ожидалось '{' после switch",
        )?;

        let mut cases = Vec::new();
        let mut default = None;

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            if self.match_token(&TokenKind::KwCase) {
                // case literal: statement
                let value = match self.parse_primary() {
                    Ok(Expression::Literal(lit)) => lit,
                    Ok(_) => {
                        return Err(ParseError::new(
                            self.current_position(),
                            ParseErrorKind::InvalidExpression,
                        )
                        .with_message("Ожидался литерал в case".to_string()));
                    }
                    Err(e) => return Err(e),
                };

                self.consume(
                    &TokenKind::Colon,
                    ParseErrorKind::ExpectedToken,
                    "ожидалось ':' после case",
                )?;

                let mut case_body = Vec::new();
                while !self.check(&TokenKind::KwCase)
                    && !self.check(&TokenKind::KwDefault)
                    && !self.check(&TokenKind::RBrace)
                    && !self.is_at_end()
                {
                    case_body.push(self.parse_statement()?);
                }

                let body_statement = if case_body.len() == 1 {
                    case_body.into_iter().next().unwrap()
                } else {
                    Statement::Block(BlockStmt::new(case_body, start_pos.line, start_pos.column))
                };

                cases.push(CaseStmt::new(
                    value,
                    body_statement,
                    start_pos.line,
                    start_pos.column,
                ));
            } else if self.match_token(&TokenKind::KwDefault) {
                self.consume(
                    &TokenKind::Colon,
                    ParseErrorKind::ExpectedToken,
                    "ожидалось ':' после default",
                )?;

                let mut default_body = Vec::new();
                while !self.check(&TokenKind::KwCase)
                    && !self.check(&TokenKind::RBrace)
                    && !self.is_at_end()
                {
                    default_body.push(self.parse_statement()?);
                }

                let body_statement = if default_body.len() == 1 {
                    default_body.into_iter().next().unwrap()
                } else {
                    Statement::Block(BlockStmt::new(
                        default_body,
                        start_pos.line,
                        start_pos.column,
                    ))
                };

                default = Some(body_statement);
            } else {
                self.advance();
            }
        }

        self.consume(
            &TokenKind::RBrace,
            ParseErrorKind::MissingCloseBrace,
            "ожидалось '}' в конце switch",
        )?;

        Ok(SwitchStmt::new(
            expression,
            cases,
            default,
            start_pos.line,
            start_pos.column,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::position::Position;
    use crate::common::token::{Token, TokenKind};

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

    #[test]
    fn test_error_recovery_missing_semicolon() {
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
            Token::new(TokenKind::RBrace, "}".to_string(), Position::new(3, 1)),
            Token::eof(Position::new(3, 2)),
        ];

        let mut parser = Parser::new(tokens);
        let ast = parser.parse();

        println!("AST построен: {:?}", ast.is_some());
        println!("Количество ошибок: {}", parser.errors().len());
        println!("Ошибки: {:?}", parser.errors().errors);
        println!("Метрики: {:?}", parser.errors().metrics);

        assert!(ast.is_some());

        assert_eq!(parser.errors().len(), 1, "Должна быть ровно одна ошибка");

        if parser.errors().len() > 0 {
            let error = &parser.errors().errors[0];
            assert_eq!(error.kind, ParseErrorKind::MissingSemicolon);

            assert_eq!(parser.errors().metrics.total_errors_detected, 1);
            assert_eq!(parser.errors().metrics.actual_errors, 1);
            assert_eq!(parser.errors().metrics.recovered_errors, 1);
        }
    }

    #[test]
    fn test_error_recovery_multiple_errors() {
        let tokens = vec![
            Token::new(TokenKind::KwFn, "fn".to_string(), Position::new(1, 1)),
            Token::new(
                TokenKind::Identifier("main".to_string()),
                "main".to_string(),
                Position::new(1, 4),
            ),
            Token::new(TokenKind::LParen, "(".to_string(), Position::new(1, 8)),
            Token::new(TokenKind::LBrace, "{".to_string(), Position::new(1, 9)),
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
            Token::new(TokenKind::RBrace, "}".to_string(), Position::new(3, 1)),
            Token::new(TokenKind::RBrace, "}".to_string(), Position::new(4, 1)),
            Token::eof(Position::new(4, 2)),
        ];

        let mut parser = Parser::new(tokens);
        let ast = parser.parse();

        assert!(ast.is_some());
        assert!(parser.errors().len() >= 2);
        assert!(parser.errors().metrics.total_errors_detected >= 2);
        assert!(parser.errors().metrics.actual_errors >= 2);
        assert!(parser.errors().metrics.recovery_quality() > 0.0);

        println!("{}", parser.errors().metrics);
    }
}
