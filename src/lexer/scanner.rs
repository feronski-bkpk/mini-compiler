//! Лексический анализатор (сканер) для языка MiniC.
//!
//! # Обзор
//!
//! Сканер преобразует исходный код на языке MiniC в последовательность токенов.
//! Он обрабатывает:
//! - Ключевые слова
//! - Идентификаторы
//! - Числовые и строковые литералы
//! - Операторы и разделители
//! - Комментарии и пробельные символы
//!
//! # Пример использования
//!
//! ```
//! use minic::lexer::Scanner;
//!
//! let source = "fn main() { return 42; }";
//! let mut scanner = Scanner::new(source);
//!
//! while !scanner.is_at_end() {
//!     match scanner.next_token() {
//!         Ok(token) => println!("{}", token),
//!         Err(error) => eprintln!("Ошибка: {}", error),
//!     }
//! }
//! ```

use std::iter::Peekable;
use std::str::Chars;

use super::error::{ErrorRecovery, LexerError, LexerResult};
use crate::common::position::Position;
use crate::common::token::{Token, TokenKind};

/// Лексический анализатор (сканер) для языка MiniC.
///
/// # Алгоритм работы
///
/// 1. Считывает исходный код символ за символом
/// 2. Определяет начало и конец каждого токена
/// 3. Классифицирует токены по типам
/// 4. Сохраняет позиционную информацию
/// 5. Обрабатывает ошибки с восстановлением
///
/// # Обработка ошибок
///
/// Сканер пытается восстановиться после ошибок, пропуская некорректные символы
/// и продолжая анализ со следующего допустимого символа.
#[derive(Debug, Clone)]
pub struct Scanner<'a> {
    /// Итератор по символам исходного кода с возможностью заглядывания вперед
    chars: Peekable<Chars<'a>>,

    /// Текущая позиция в исходном коде
    current_position: Position,

    /// Позиция начала текущего токена
    start_position: Position,

    /// Накопленная лексема текущего токена
    current_lexeme: String,

    /// Контекст восстановления после ошибок
    error_recovery: ErrorRecovery,
}

impl<'a> Scanner<'a> {
    /// Создает сканер из предобработанного исходного кода.
    pub fn from_preprocessed(source: &'a str) -> Self {
        Self::new(source)
    }

    /// Создает новый сканер для указанного исходного кода.
    ///
    /// # Аргументы
    ///
    /// * `source` - исходный код на языке MiniC
    ///
    /// # Пример
    ///
    /// ```
    /// use minic::lexer::Scanner;
    ///
    /// let scanner = Scanner::new("fn main() {}");
    /// ```
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
            current_position: Position::start(),
            start_position: Position::start(),
            current_lexeme: String::new(),
            error_recovery: ErrorRecovery::new(),
        }
    }

    /// Продвигает сканер на один символ вперед и обновляет позицию.
    ///
    /// # Возвращает
    ///
    /// * `Some(char)` - прочитанный символ
    /// * `None` - если достигнут конец файла
    ///
    /// # Обновление позиции
    ///
    /// - Для символа новой строки (`\n`): увеличивает номер строки,
    ///   сбрасывает колонку в 1
    /// - Для возврата каретки (`\r`): увеличивает колонку на 1
    ///   (полная обработка `\r\n` выполняется в `advance_whitespace()`)
    /// - Для других символов: увеличивает колонку на 1
    ///
    /// # Обработка лексемы
    ///
    /// Символ добавляется в `current_lexeme` только если он НЕ является
    /// пробельным символом. Пробельные символы обрабатываются отдельно
    /// в `skip_whitespace()` с помощью `advance_whitespace()`.
    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next()?;

        match c {
            '\n' => {
                self.current_position.new_line();
            }
            '\r' => {
                self.current_position.advance_column(1);
            }
            _ => {
                self.current_position.advance_column(1);
            }
        }

        if !c.is_whitespace() {
            self.current_lexeme.push(c);
        }

        Some(c)
    }

    /// Заглядывает вперед на следующий символ без его чтения.
    ///
    /// # Возвращает
    ///
    /// * `Some(&char)` - ссылка на следующий символ
    /// * `None` - если достигнут конец файла
    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    /// Проверяет, совпадает ли следующий символ с ожидаемым.
    /// Если совпадает - читает его, иначе оставляет на месте.
    ///
    /// # Аргументы
    ///
    /// * `expected` - ожидаемый символ
    ///
    /// # Возвращает
    ///
    /// * `true` - если символ совпал и был прочитан
    /// * `false` - если символ не совпал
    fn matches(&mut self, expected: char) -> bool {
        match self.peek() {
            Some(&ch) if ch == expected => {
                self.advance();
                true
            }
            _ => false,
        }
    }

    /// Проверяет, достигнут ли конец исходного кода.
    ///
    /// # Возвращает
    ///
    /// * `true` - если все символы прочитаны
    /// * `false` - если остались символы для чтения
    pub fn is_at_end(&mut self) -> bool {
        self.peek().is_none()
    }

    /// Возвращает текущий номер строки.
    pub fn get_line(&self) -> usize {
        self.current_position.line
    }

    /// Возвращает текущий номер колонки.
    pub fn get_column(&self) -> usize {
        self.current_position.column
    }

    /// Подготавливает сканер к чтению нового токена.
    ///
    /// Сохраняет текущую позицию как начало токена и очищает накопленную лексему.
    fn start_token(&mut self) {
        self.start_position = self.current_position;
        self.current_lexeme.clear();
        self.error_recovery = ErrorRecovery::new();
    }

    /// Создает токен на основе текущего состояния сканера.
    ///
    /// # Аргументы
    ///
    /// * `kind` - тип создаваемого токена
    ///
    /// # Возвращает
    ///
    /// Токен с накопленной лексемой и позицией начала
    fn make_token(&self, kind: TokenKind) -> Token {
        Token::new(kind, self.current_lexeme.clone(), self.start_position)
    }

    /// Создает объект ошибки с текущей позицией.
    ///
    /// # Аргументы
    ///
    /// * `error` - тип ошибки
    fn error(&self, error: LexerError) -> LexerError {
        error
    }

    /// Пропускает пробельные символы (пробелы и табуляции).
    ///
    /// Не пропускает символы новой строки, так как они важны для позиционирования.
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

    /// Продвигает сканер на один пробельный символ вперед.
    ///
    /// Используется исключительно в `skip_whitespace()` для пропуска
    /// пробелов, табуляций и переводов строк без добавления их в лексему.
    ///
    /// # Возвращает
    ///
    /// * `Some(char)` - прочитанный пробельный символ
    /// * `None` - если достигнут конец файла
    ///
    /// # Особенности обработки
    ///
    /// 1. **Windows-стиль окончаний строк (`\r\n`)**:
    ///    - При чтении `\r` проверяет следующий символ
    ///    - Если следующий символ `\n`, пропускает его тоже
    ///    - Учитывается как один перевод строки
    ///
    /// 2. **UNIX-стиль окончаний строк (`\n`)**:
    ///    - Обрабатывается как обычный перевод строки
    ///
    /// 3. **Позиционирование**:
    ///    - Символы НЕ добавляются в `current_lexeme`
    ///    - Позиция обновляется корректно для всех символов
    fn advance_whitespace(&mut self) -> Option<char> {
        let c = self.chars.next()?;

        match c {
            '\n' => {
                self.current_position.new_line();
            }
            '\r' => {
                if let Some(&next) = self.chars.peek() {
                    if next == '\n' {
                        self.chars.next(); // Пропускаем \n
                        self.current_position.new_line();
                    } else {
                        self.current_position.advance_column(1);
                    }
                } else {
                    self.current_position.advance_column(1);
                }
            }
            _ => {
                self.current_position.advance_column(1);
            }
        }

        Some(c)
    }

    /// Обрабатывает комментарии.
    ///
    /// Поддерживает:
    /// - Однострочные комментарии (`// комментарий`)
    /// - Многострочные комментарии (`/* комментарий */`)
    ///
    /// # Возвращает
    ///
    /// * `Ok(())` - если комментарий успешно обработан
    /// * `Err(LexerError)` - если комментарий не завершен
    ///
    /// # Ошибки
    ///
    /// Возвращает `LexerError::UnterminatedComment` для незавершенных
    /// многострочных комментариев.
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

    /// Сканирует строковый литерал.
    ///
    /// Обрабатывает:
    /// - Обычные символы
    /// - Escape-последовательности (`\n`, `\t`, `\r`, `\\`, `\"`, `\'`)
    /// - Незавершенные строки
    ///
    /// # Возвращает
    ///
    /// * `Ok(Token)` - токен строкового литерала
    /// * `Err(LexerError)` - если строка не завершена
    ///
    /// # Примеры
    ///
    /// - `"hello"` → `StringLiteral("hello")`
    /// - `"line1\nline2"` → `StringLiteral("line1\nline2")`
    fn scan_string(&mut self) -> LexerResult<Token> {
        while let Some(&c) = self.peek() {
            match c {
                '"' => {
                    self.advance();

                    let content = if self.current_lexeme.len() >= 2 {
                        self.current_lexeme[1..self.current_lexeme.len() - 1].to_string()
                    } else {
                        String::new()
                    };

                    return Ok(self.make_token(TokenKind::StringLiteral(content)));
                }
                '\\' => {
                    self.advance();

                    if let Some(&next) = self.peek() {
                        match next {
                            'n' | 't' | 'r' | '\\' | '"' | '\'' => {
                                self.advance();
                            }
                            _ => {
                                self.advance();
                            }
                        }
                    }
                }
                '\n' => {
                    return Err(self.error(LexerError::UnterminatedString {
                        position: self.start_position,
                    }));
                }
                _ => {
                    self.advance();
                }
            }
        }

        Err(self.error(LexerError::UnterminatedString {
            position: self.start_position,
        }))
    }

    /// Сканирует числовой литерал (целое или с плавающей точкой).
    ///
    /// # Форматы чисел
    ///
    /// - Целые: `42`, `-100`, `0`
    /// - С плавающей точкой: `3.14`, `-0.5`, `123.456`
    ///
    /// # Возвращает
    ///
    /// * `Ok(Token)` - токен числового литерала
    /// * `Err(LexerError)` - если формат числа некорректен
    ///
    /// # Ограничения
    ///
    /// - Целые числа: диапазон i32 (`-2³¹` до `2³¹-1`)
    /// - После точки должна быть хотя бы одна цифра
    fn scan_number(&mut self) -> LexerResult<Token> {
        let mut is_float = false;
        let mut has_digit_after_dot = false;

        let has_minus = self.current_lexeme == "-";

        while let Some(&c) = self.peek() {
            if c.is_ascii_digit() {
                self.advance();
                if is_float {
                    has_digit_after_dot = true;
                }
            } else if c == '.' && !is_float {
                is_float = true;
                self.advance();
            } else {
                break;
            }
        }

        if is_float && !has_digit_after_dot {
            return Err(self.error(LexerError::InvalidNumber {
                position: self.start_position,
                lexeme: self.current_lexeme.clone(),
            }));
        }

        if is_float {
            match self.current_lexeme.parse::<f64>() {
                Ok(value) => Ok(self.make_token(TokenKind::FloatLiteral(value))),
                Err(_) => Err(self.error(LexerError::InvalidNumber {
                    position: self.start_position,
                    lexeme: self.current_lexeme.clone(),
                })),
            }
        } else {
            match self.current_lexeme.parse::<i32>() {
                Ok(value) => Ok(self.make_token(TokenKind::IntLiteral(value))),
                Err(_) => {
                    if has_minus {
                        Err(self.error(LexerError::InvalidNumber {
                            position: self.start_position,
                            lexeme: self.current_lexeme.clone(),
                        }))
                    } else {
                        Err(self.error(LexerError::InvalidNumber {
                            position: self.start_position,
                            lexeme: self.current_lexeme.clone(),
                        }))
                    }
                }
            }
        }
    }

    /// Сканирует идентификатор или ключевое слово.
    ///
    /// # Правила идентификаторов
    ///
    /// 1. Начинается с буквы или подчеркивания
    /// 2. Может содержать буквы, цифры и подчеркивания
    /// 3. Максимальная длина: 255 символов
    /// 4. Чувствительность к регистру
    ///
    /// # Возвращает
    ///
    /// Токен идентификатора или ключевого слова
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
            "true" => self.make_token(TokenKind::BoolLiteral(true)),
            "false" => self.make_token(TokenKind::BoolLiteral(false)),
            "void" => self.make_token(TokenKind::KwVoid),
            "struct" => self.make_token(TokenKind::KwStruct),
            "fn" => self.make_token(TokenKind::KwFn),
            _ => self.make_token(TokenKind::Identifier(self.current_lexeme.clone())),
        };

        Ok(token)
    }

    /// Читает следующий токен из исходного кода.
    ///
    /// # Возвращает
    ///
    /// * `Ok(Token)` - следующий токен
    /// * `Err(LexerError)` - ошибка лексического анализа
    ///
    /// # Алгоритм
    ///
    /// 1. Пропускает пробельные символы и комментарии
    /// 2. Определяет тип токена по первому символу
    /// 3. Читает полную лексему токена
    /// 4. Создает токен с позиционной информацией
    ///
    /// # Восстановление после ошибок
    ///
    /// При возникновении ошибки сканер пытается пропустить некорректные символы
    /// и продолжить анализ со следующего допустимого символа.
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

            '+' => {
                if self.matches('=') {
                    Ok(self.make_token(TokenKind::PlusEq))
                } else {
                    Ok(self.make_token(TokenKind::Plus))
                }
            }
            '-' => {
                if let Some(&next_char) = self.peek() {
                    if next_char.is_ascii_digit() {
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
                    Err(self.error(LexerError::UnexpectedCharacter {
                        position: self.start_position,
                        character: '&',
                    }))
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

            '"' => self.scan_string(),

            c if c.is_ascii_digit() => self.scan_number(),

            c if c.is_ascii_alphabetic() || c == '_' => self.scan_identifier_or_keyword(),

            _ => Err(self.error(LexerError::UnexpectedCharacter {
                position: self.start_position,
                character: c,
            })),
        }
    }

    /// Заглядывает вперед на следующий токен без его чтения.
    ///
    /// # Возвращает
    ///
    /// * `Ok(Token)` - следующий токен
    /// * `Err(LexerError)` - ошибка лексического анализа
    ///
    /// # Примечание
    ///
    /// Метод сохраняет и восстанавливает состояние сканера,
    /// поэтому его вызов не влияет на последующий вызов `next_token()`.
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

    /// Сканирует все токены до конца файла.
    ///
    /// # Возвращает
    ///
    /// Вектор токенов и вектор ошибок (если были)
    ///
    /// # Пример
    ///
    /// ```
    /// use minic::lexer::Scanner;
    ///
    /// let source = "fn main() { return 42; }";
    /// let mut scanner = Scanner::new(source);
    /// let (tokens, errors) = scanner.scan_all();
    ///
    /// if errors.is_empty() {
    ///     for token in tokens {
    ///         println!("{}", token);
    ///     }
    /// } else {
    ///     for error in errors {
    ///         eprintln!("Ошибка: {}", error);
    ///     }
    /// }
    /// ```
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_assignment() {
        let source = "x = 42;";
        let mut scanner = Scanner::new(source);
        let mut tokens = Vec::new();

        loop {
            match scanner.next_token() {
                Ok(token) => {
                    let is_eof = token.is_eof();
                    tokens.push(token);
                    if is_eof {
                        break;
                    }
                }
                Err(e) => {
                    panic!("Ошибка при сканировании: {}", e);
                }
            }
        }

        assert_eq!(tokens.len(), 5);

        assert_eq!(tokens[0].lexeme, "x");
        assert_eq!(tokens[1].lexeme, "=");
        assert_eq!(tokens[2].lexeme, "42");
        assert_eq!(tokens[3].lexeme, ";");
    }

    #[test]
    fn test_unterminated_comment() {
        let source = "/* Unterminated comment";
        let mut scanner = Scanner::new(source);

        scanner.skip_whitespace();

        match scanner.skip_comments() {
            Ok(_) => println!("Комментарий успешно пропущен (но он незавершенный!?)"),
            Err(e) => println!("Ошибка незавершенного комментария: {}", e),
        }

        let mut scanner2 = Scanner::new(source);
        match scanner2.next_token() {
            Ok(token) => println!("next_token вернул: {:?}", token.kind),
            Err(e) => println!("next_token ошибка: {}", e),
        }
    }
}
