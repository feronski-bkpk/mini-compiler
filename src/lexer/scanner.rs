//! Лексический анализатор (сканер) для языка MiniC.

use std::iter::Peekable;
use std::str::Chars;

use super::error::{ErrorRecovery, LexerError, LexerResult};
use crate::common::position::Position;
use crate::common::token::{Token, TokenKind};

#[derive(Debug, Clone)]
pub struct Scanner<'a> {
    chars: Peekable<Chars<'a>>,
    current_position: Position,
    start_position: Position,
    current_lexeme: String,
    error_recovery: ErrorRecovery,
}

impl<'a> Scanner<'a> {
    pub fn from_preprocessed(source: &'a str) -> Self {
        Self::new(source)
    }

    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
            current_position: Position::start(),
            start_position: Position::start(),
            current_lexeme: String::new(),
            error_recovery: ErrorRecovery::new(),
        }
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        match c {
            '\n' => self.current_position.new_line(),
            '\r' => self.current_position.advance_column(1),
            _ => self.current_position.advance_column(1),
        }
        if !c.is_whitespace() {
            self.current_lexeme.push(c);
        }
        Some(c)
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    fn matches(&mut self, expected: char) -> bool {
        match self.peek() {
            Some(&ch) if ch == expected => {
                self.advance();
                true
            }
            _ => false,
        }
    }

    pub fn is_at_end(&mut self) -> bool {
        self.peek().is_none()
    }

    pub fn get_line(&self) -> usize {
        self.current_position.line
    }

    pub fn get_column(&self) -> usize {
        self.current_position.column
    }

    fn start_token(&mut self) {
        self.start_position = self.current_position;
        self.current_lexeme.clear();
        self.error_recovery = ErrorRecovery::new();
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        Token::new(kind, self.current_lexeme.clone(), self.start_position)
    }

    fn error(&self, error: LexerError) -> LexerError {
        error
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.peek() {
            if c.is_whitespace() {
                self.advance_whitespace();
            } else {
                break;
            }
        }
        self.current_lexeme.clear();
        self.start_position = self.current_position;
    }

    fn advance_whitespace(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        match c {
            '\n' => self.current_position.new_line(),
            '\r' => {
                if let Some(&next) = self.chars.peek() {
                    if next == '\n' {
                        self.chars.next();
                        self.current_position.new_line();
                    } else {
                        self.current_position.advance_column(1);
                    }
                } else {
                    self.current_position.advance_column(1);
                }
            }
            _ => self.current_position.advance_column(1),
        }
        Some(c)
    }

    fn skip_comments(&mut self) -> LexerResult<()> {
        if self.matches('/') && self.matches('/') {
            while let Some(&c) = self.peek() {
                if c == '\n' {
                    break;
                }
                self.advance();
            }
            return Ok(());
        }

        if self.matches('/') && self.matches('*') {
            let mut depth = 1;
            while depth > 0 && !self.is_at_end() {
                if self.matches('/') && self.matches('*') {
                    depth += 1;
                } else if self.matches('*') && self.matches('/') {
                    depth -= 1;
                } else {
                    self.advance();
                }
            }
            if depth > 0 {
                return Err(self.error(LexerError::UnterminatedComment {
                    position: self.start_position,
                }));
            }
            return Ok(());
        }

        self.current_lexeme.clear();
        self.current_position = self.start_position;
        Ok(())
    }

    fn scan_string(&mut self) -> LexerResult<Token> {
        self.current_lexeme.clear();
        self.current_lexeme.push('"');

        loop {
            match self.chars.next() {
                None => {
                    return Err(self.error(LexerError::UnterminatedString {
                        position: self.start_position,
                    }));
                }
                Some('"') => {
                    self.current_lexeme.push('"');
                    self.current_position.advance_column(1);

                    let content = if self.current_lexeme.len() >= 2 {
                        self.current_lexeme[1..self.current_lexeme.len() - 1].to_string()
                    } else {
                        String::new()
                    };
                    return Ok(self.make_token(TokenKind::StringLiteral(content)));
                }
                Some('\\') => {
                    self.current_lexeme.push('\\');
                    self.current_position.advance_column(1);

                    match self.chars.next() {
                        Some(c) => {
                            self.current_lexeme.push(c);
                            if c == '\n' {
                                self.current_position.new_line();
                            } else {
                                self.current_position.advance_column(1);
                            }
                        }
                        None => {
                            return Err(self.error(LexerError::UnterminatedString {
                                position: self.start_position,
                            }));
                        }
                    }
                }
                Some('\n') => {
                    return Err(self.error(LexerError::UnterminatedString {
                        position: self.start_position,
                    }));
                }
                Some(c) => {
                    self.current_lexeme.push(c);
                    if c == '\r' {
                        if self.chars.peek() == Some(&'\n') {
                            self.chars.next();
                            self.current_lexeme.push('\n');
                            self.current_position.new_line();
                        } else {
                            self.current_position.advance_column(1);
                        }
                    } else {
                        self.current_position.advance_column(1);
                    }
                }
            }
        }
    }

    fn scan_number(&mut self) -> LexerResult<Token> {
        let start_pos = self.start_position;
        let mut is_float = false;
        let mut has_digits = false;
        let mut has_minus = false;

        if self.current_lexeme == "-" {
            has_minus = true;
        } else if !self.current_lexeme.is_empty() {
            if self.current_lexeme.chars().next().unwrap().is_ascii_digit() {
                has_digits = true;
            }
        }

        while let Some(&c) = self.peek() {
            if c.is_ascii_digit() {
                has_digits = true;
                self.advance();
            } else {
                break;
            }
        }

        if let Some(&c) = self.peek() {
            if c == '.' {
                if !has_digits && !has_minus {
                    return Err(self.error(LexerError::InvalidNumber {
                        position: start_pos,
                        lexeme: self.current_lexeme.clone() + ".",
                    }));
                }
                is_float = true;
                self.advance();
                let mut has_digits_after = false;
                while let Some(&c) = self.peek() {
                    if c.is_ascii_digit() {
                        has_digits_after = true;
                        self.advance();
                    } else {
                        break;
                    }
                }
                if !has_digits_after {
                    return Err(self.error(LexerError::InvalidNumber {
                        position: start_pos,
                        lexeme: self.current_lexeme.clone(),
                    }));
                }
            }
        }

        if has_minus && !has_digits && !is_float {
            return Ok(self.make_token(TokenKind::Minus));
        }

        if !has_digits && !is_float {
            return Err(self.error(LexerError::InvalidNumber {
                position: start_pos,
                lexeme: self.current_lexeme.clone(),
            }));
        }

        if is_float {
            match self.current_lexeme.parse::<f64>() {
                Ok(value) => Ok(self.make_token(TokenKind::FloatLiteral(value))),
                Err(_) => Err(self.error(LexerError::InvalidNumber {
                    position: start_pos,
                    lexeme: self.current_lexeme.clone(),
                })),
            }
        } else {
            match self.current_lexeme.parse::<i32>() {
                Ok(value) => Ok(self.make_token(TokenKind::IntLiteral(value))),
                Err(_) => {
                    if let Ok(value) = self.current_lexeme.parse::<i64>() {
                        if value > i32::MAX as i64 || value < i32::MIN as i64 {
                            return Err(self.error(LexerError::InvalidNumber {
                                position: start_pos,
                                lexeme: self.current_lexeme.clone(),
                            }));
                        }
                    }
                    Err(self.error(LexerError::InvalidNumber {
                        position: start_pos,
                        lexeme: self.current_lexeme.clone(),
                    }))
                }
            }
        }
    }

    fn scan_identifier_or_keyword(&mut self) -> LexerResult<Token> {
        while let Some(&c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        if self.current_lexeme.len() > 255 {
            return Err(self.error(LexerError::IdentifierTooLong {
                position: self.start_position,
            }));
        }

        let token = match self.current_lexeme.as_str() {
            "if" => self.make_token(TokenKind::KwIf),
            "else" => self.make_token(TokenKind::KwElse),
            "while" => self.make_token(TokenKind::KwWhile),
            "for" => self.make_token(TokenKind::KwFor),
            "int" => self.make_token(TokenKind::KwInt),
            "float" => self.make_token(TokenKind::KwFloat),
            "bool" => self.make_token(TokenKind::KwBool),
            "return" => self.make_token(TokenKind::KwReturn),
            "true" => self.make_token(TokenKind::KwTrue),
            "false" => self.make_token(TokenKind::KwFalse),
            "void" => self.make_token(TokenKind::KwVoid),
            "struct" => self.make_token(TokenKind::KwStruct),
            "fn" => self.make_token(TokenKind::KwFn),
            "string" => self.make_token(TokenKind::KwString),
            "break" => self.make_token(TokenKind::KwBreak),
            "continue" => self.make_token(TokenKind::KwContinue),
            "switch" => self.make_token(TokenKind::KwSwitch),
            "case" => self.make_token(TokenKind::KwCase),
            "default" => self.make_token(TokenKind::KwDefault),
            "extern" => self.make_token(TokenKind::KwExtern),
            "char" => self.make_token(TokenKind::KwChar),
            _ => self.make_token(TokenKind::Identifier(self.current_lexeme.clone())),
        };

        Ok(token)
    }

    pub fn next_token(&mut self) -> LexerResult<Token> {
        self.start_token();
        self.skip_whitespace();

        while let Some(&c) = self.peek() {
            if c == '/' {
                let mut lookahead = self.chars.clone();
                lookahead.next();
                if let Some(next_char) = lookahead.next() {
                    if next_char == '/' || next_char == '*' {
                        match self.skip_comments() {
                            Ok(_) => {
                                self.skip_whitespace();
                                continue;
                            }
                            Err(e) => {
                                self.error_recovery.mark_recovered(self.start_position);
                                return Err(e);
                            }
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if self.is_at_end() {
            return Ok(Token::eof(self.current_position));
        }

        self.start_position = self.current_position;
        self.current_lexeme.clear();

        if let Some(&'"') = self.peek() {
            self.chars.next();
            self.current_position.advance_column(1);
            return self.scan_string();
        }

        let c = self.advance().unwrap();

        match c {
            '(' => Ok(self.make_token(TokenKind::LParen)),
            ')' => Ok(self.make_token(TokenKind::RParen)),
            '{' => Ok(self.make_token(TokenKind::LBrace)),
            '}' => Ok(self.make_token(TokenKind::RBrace)),
            '[' => Ok(self.make_token(TokenKind::LBracket)),
            ']' => Ok(self.make_token(TokenKind::RBracket)),
            ';' => Ok(self.make_token(TokenKind::Semicolon)),
            ',' => Ok(self.make_token(TokenKind::Comma)),
            ':' => Ok(self.make_token(TokenKind::Colon)),
            '.' => {
                if self.matches('.') {
                    if self.matches('.') {
                        Ok(self.make_token(TokenKind::Ellipsis))
                    } else {
                        Err(self.error(LexerError::UnexpectedCharacter {
                            position: self.start_position,
                            character: '.',
                        }))
                    }
                } else if let Some(&next_char) = self.peek() {
                    if next_char.is_ascii_digit() {
                        Err(self.error(LexerError::InvalidNumber {
                            position: self.start_position,
                            lexeme: ".".to_string() + &next_char.to_string(),
                        }))
                    } else {
                        Ok(self.make_token(TokenKind::Dot))
                    }
                } else {
                    Ok(self.make_token(TokenKind::Dot))
                }
            }
            '+' => {
                if self.matches('+') {
                    Ok(self.make_token(TokenKind::PlusPlus))
                } else if self.matches('=') {
                    Ok(self.make_token(TokenKind::PlusEq))
                } else {
                    Ok(self.make_token(TokenKind::Plus))
                }
            }
            '-' => {
                if self.matches('-') {
                    Ok(self.make_token(TokenKind::MinusMinus))
                } else if let Some(&next_char) = self.peek() {
                    if next_char == '>' {
                        self.advance();
                        Ok(self.make_token(TokenKind::Arrow))
                    } else if next_char.is_ascii_digit() {
                        self.scan_number()
                    } else if next_char == '.' {
                        self.scan_number()
                    } else if self.matches('=') {
                        Ok(self.make_token(TokenKind::MinusEq))
                    } else {
                        Ok(self.make_token(TokenKind::Minus))
                    }
                } else {
                    Ok(self.make_token(TokenKind::Minus))
                }
            }
            '*' => {
                if self.matches('=') {
                    Ok(self.make_token(TokenKind::AsteriskEq))
                } else {
                    Ok(self.make_token(TokenKind::Asterisk))
                }
            }
            '/' => {
                if self.matches('=') {
                    Ok(self.make_token(TokenKind::SlashEq))
                } else {
                    Ok(self.make_token(TokenKind::Slash))
                }
            }
            '%' => Ok(self.make_token(TokenKind::Percent)),
            '!' => {
                if self.matches('=') {
                    Ok(self.make_token(TokenKind::BangEq))
                } else {
                    Ok(self.make_token(TokenKind::Bang))
                }
            }
            '=' => {
                if self.matches('=') {
                    Ok(self.make_token(TokenKind::EqEq))
                } else {
                    Ok(self.make_token(TokenKind::Eq))
                }
            }
            '<' => {
                if self.matches('=') {
                    Ok(self.make_token(TokenKind::LtEq))
                } else {
                    Ok(self.make_token(TokenKind::Lt))
                }
            }
            '>' => {
                if self.matches('=') {
                    Ok(self.make_token(TokenKind::GtEq))
                } else {
                    Ok(self.make_token(TokenKind::Gt))
                }
            }
            '&' => {
                if self.matches('&') {
                    Ok(self.make_token(TokenKind::AmpAmp))
                } else {
                    Ok(self.make_token(TokenKind::Amp))
                }
            }
            '|' => {
                if self.matches('|') {
                    Ok(self.make_token(TokenKind::PipePipe))
                } else {
                    Err(self.error(LexerError::UnexpectedCharacter {
                        position: self.start_position,
                        character: '|',
                    }))
                }
            }
            c if c.is_ascii_digit() => self.scan_number(),
            c if c.is_ascii_alphabetic() || c == '_' => self.scan_identifier_or_keyword(),
            _ => Err(self.error(LexerError::UnexpectedCharacter {
                position: self.start_position,
                character: c,
            })),
        }
    }

    pub fn peek_token(&mut self) -> LexerResult<Token> {
        let saved_position = self.current_position;
        let saved_start = self.start_position;
        let saved_lexeme = self.current_lexeme.clone();
        let saved_chars = self.chars.clone();
        let saved_recovery = self.error_recovery.clone();
        let token = self.next_token();
        self.current_position = saved_position;
        self.start_position = saved_start;
        self.current_lexeme = saved_lexeme;
        self.chars = saved_chars;
        self.error_recovery = saved_recovery;
        token
    }

    pub fn scan_all(&mut self) -> (Vec<Token>, Vec<LexerError>) {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();
        loop {
            match self.next_token() {
                Ok(token) => {
                    let is_eof = token.is_eof();
                    tokens.push(token);
                    if is_eof {
                        break;
                    }
                }
                Err(error) => {
                    errors.push(error);
                    self.error_recovery.mark_recovered(self.start_position);
                    if let Some(_c) = self.advance() {
                        self.error_recovery.skip_char();
                    }
                }
            }
        }
        (tokens, errors)
    }
}
