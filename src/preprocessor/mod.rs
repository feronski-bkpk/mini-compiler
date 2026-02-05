//! Препроцессор для языка MiniC.
//!
//! Обрабатывает директивы препроцессора и очищает исходный код:
//! 1. Удаляет комментарии
//! 2. Обрабатывает макросы #define
//! 3. Поддерживает #ifdef/#ifndef/#endif
//!
//! # Пример использования
//!
//! ```
//! use minic::preprocessor::Preprocessor;
//!
//! let source = r#"
//! #define MAX 100
//! int x = MAX;
//! // комментарий
//! /* многострочный
//!    комментарий */
//! "#;
//!
//! let mut preprocessor = Preprocessor::new(source);
//! preprocessor.define("DEBUG", "1");
//!
//! match preprocessor.process() {
//!     Ok(result) => println!("Очищенный код:\n{}", result),
//!     Err(err) => eprintln!("Ошибка препроцессора: {}", err),
//! }
//! ```

mod error;
mod macros;

pub use error::PreprocessorError;
pub use macros::{MacroDefinition, MacroTable};

use crate::common::position::Position;

/// Препроцессор для обработки исходного кода MiniC.
#[derive(Debug)]
pub struct Preprocessor<'a> {
    /// Исходный код
    source: &'a str,
    /// Таблица макросов
    macros: MacroTable,
    /// Сохранение нумерации строк (заменять комментарии на пробелы или новые строки)
    preserve_line_numbers: bool,
    /// Поддержка вложенных директив #if
    support_conditionals: bool,
}

impl<'a> Preprocessor<'a> {
    /// Создает новый препроцессор для указанного исходного кода.
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            macros: MacroTable::new(),
            preserve_line_numbers: true,
            support_conditionals: true,
        }
    }

    /// Определяет макрос.
    pub fn define(&mut self, name: &str, value: &str) -> Result<(), PreprocessorError> {
        self.macros.define(name, value)
    }

    /// Удаляет определение макроса.
    pub fn undefine(&mut self, name: &str) {
        self.macros.undefine(name);
    }

    /// Устанавливает, нужно ли сохранять нумерацию строк.
    /// Если true, комментарии заменяются на пробелы/новые строки.
    pub fn preserve_line_numbers(&mut self, preserve: bool) {
        self.preserve_line_numbers = preserve;
    }

    /// Включает или выключает поддержку условных директив (#ifdef, #ifndef, #endif).
    pub fn enable_conditionals(&mut self, enable: bool) {
        self.support_conditionals = enable;
    }

    /// Обрабатывает исходный код и возвращает очищенную версию.
    pub fn process(&mut self) -> Result<String, PreprocessorError> {
        let mut result = String::with_capacity(self.source.len());
        let mut condition_stack = Vec::new();

        let processed_source =
            self.remove_comments_from_whole_source(self.source, Position::new(1, 1))?;

        let lines = processed_source.lines().enumerate();

        for (line_num, line) in lines {
            let line_position = Position::new(line_num + 1, 1);

            if let Some(directive) = Self::parse_directive(line) {
                match self.process_directive(directive, &mut condition_stack, line_position)? {
                    DirectiveResult::SkipLine => continue,
                    DirectiveResult::ProcessLine(processed) => {
                        result.push_str(&processed);
                        result.push('\n');
                        continue;
                    }
                    DirectiveResult::Continue => {
                    }
                }
            }

            if !self.is_section_active(&condition_stack) {
                if self.preserve_line_numbers {
                    result.push('\n');
                }
                continue;
            }

            let expanded_line = self.macros.expand(line)?;

            result.push_str(&expanded_line);
            result.push('\n');
        }

        if !condition_stack.is_empty() {
            return Err(PreprocessorError::UnterminatedConditional {
                position: Position::new(1, 1),
            });
        }

        Ok(result)
    }

    /// Определяет, является ли строка директивой препроцессора.
    fn parse_directive(line: &str) -> Option<&str> {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            Some(trimmed)
        } else {
            None
        }
    }

    /// Обрабатывает директиву препроцессора.
    fn process_directive(
        &mut self,
        directive: &str,
        condition_stack: &mut Vec<ConditionState>,
        position: Position,
    ) -> Result<DirectiveResult, PreprocessorError> {
        let parts: Vec<&str> = directive.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(DirectiveResult::SkipLine);
        }

        match parts[0] {
            "#define" => {
                if parts.len() < 2 {
                    return Err(PreprocessorError::InvalidDirective {
                        position,
                        directive: directive.to_string(),
                        reason: "Missing macro name".to_string(),
                    });
                }

                let name = parts[1];
                let value = if parts.len() > 2 {
                    parts[2..].join(" ")
                } else {
                    String::new()
                };

                self.macros.define(name, &value)?;
                Ok(DirectiveResult::SkipLine)
            }
            "#undef" => {
                if parts.len() < 2 {
                    return Err(PreprocessorError::InvalidDirective {
                        position,
                        directive: directive.to_string(),
                        reason: "Missing macro name".to_string(),
                    });
                }

                self.macros.undefine(parts[1]);
                Ok(DirectiveResult::SkipLine)
            }
            "#ifdef" => {
                if !self.support_conditionals {
                    return Ok(DirectiveResult::SkipLine);
                }

                if parts.len() < 2 {
                    return Err(PreprocessorError::InvalidDirective {
                        position,
                        directive: directive.to_string(),
                        reason: "Missing condition".to_string(),
                    });
                }

                let name = parts[1];
                let is_defined = self.macros.is_defined(name);
                condition_stack.push(ConditionState::IfDef { active: is_defined });

                Ok(DirectiveResult::SkipLine)
            }
            "#ifndef" => {
                if !self.support_conditionals {
                    return Ok(DirectiveResult::SkipLine);
                }

                if parts.len() < 2 {
                    return Err(PreprocessorError::InvalidDirective {
                        position,
                        directive: directive.to_string(),
                        reason: "Missing condition".to_string(),
                    });
                }

                let name = parts[1];
                let is_defined = self.macros.is_defined(name);
                condition_stack.push(ConditionState::IfNDef {
                    active: !is_defined,
                });

                Ok(DirectiveResult::SkipLine)
            }
            "#endif" => {
                if !self.support_conditionals {
                    return Ok(DirectiveResult::SkipLine);
                }

                if condition_stack.pop().is_none() {
                    return Err(PreprocessorError::UnmatchedEndif { position });
                }

                Ok(DirectiveResult::SkipLine)
            }
            "#" => {
                Ok(DirectiveResult::SkipLine)
            }
            "#else" => {
                if !self.support_conditionals {
                    return Ok(DirectiveResult::SkipLine);
                }

                if let Some(state) = condition_stack.last_mut() {
                    match state {
                        ConditionState::IfDef { active, .. } => {
                            *active = !*active;
                        }
                        ConditionState::IfNDef { active, .. } => {
                            *active = !*active;
                        }
                    }
                } else {
                    return Err(PreprocessorError::UnmatchedElse { position });
                }

                Ok(DirectiveResult::SkipLine)
            }
            _ => {
                Ok(DirectiveResult::ProcessLine(directive.to_string()))
            }
        }
    }

    /// Проверяет, активна ли текущая секция кода.
    fn is_section_active(&self, condition_stack: &[ConditionState]) -> bool {
        let mut active = true;

        for state in condition_stack {
            match state {
                ConditionState::IfDef { active: a, .. } => active = active && *a,
                ConditionState::IfNDef { active: a, .. } => active = active && *a,
            }
        }

        active
    }

    fn remove_comments_from_whole_source(
        &self,
        source: &str,
        start_position: Position,
    ) -> Result<String, PreprocessorError> {
        let mut result = String::with_capacity(source.len());
        let mut chars = source.chars().peekable();
        let mut position = start_position;
        let mut in_string = false;
        let mut in_char = false;
        let mut escape_next = false;
        let mut in_comment = false;
        let mut comment_start: Option<Position> = None;

        while let Some(c) = chars.next() {
            if c == '\n' {
                position.line += 1;
                position.column = 1;
            } else {
                position.column += 1;
            }

            if escape_next {
                escape_next = false;
                result.push(c);
                continue;
            }

            match c {
                '"' if !in_char && !in_comment => {
                    in_string = !in_string;
                    result.push(c);
                }
                '\'' if !in_string && !in_comment => {
                    in_char = !in_char;
                    result.push(c);
                }
                '\\' if (in_string || in_char) && !in_comment => {
                    escape_next = true;
                    result.push(c);
                }
                '/' if !in_string && !in_char && !in_comment => {
                    match chars.peek() {
                        Some('/') => {
                            chars.next();
                            if self.preserve_line_numbers {
                                result.push(' ');
                                for ch in chars.by_ref() {
                                    if ch == '\n' {
                                        result.push('\n');
                                        position.line += 1;
                                        position.column = 1;
                                        break;
                                    }
                                    result.push(' ');
                                }
                            } else {
                                for ch in chars.by_ref() {
                                    if ch == '\n' {
                                        result.push('\n');
                                        position.line += 1;
                                        position.column = 1;
                                        break;
                                    }
                                }
                            }
                        }
                        Some('*') => {
                            chars.next();
                            comment_start = Some(position);
                            in_comment = true;

                            if self.preserve_line_numbers {
                                result.push(' ');
                                result.push(' ');
                            }
                        }
                        _ => {
                            result.push(c);
                        }
                    }
                }
                '*' if in_comment => {
                    if let Some('/') = chars.peek() {
                        chars.next();
                        in_comment = false;
                        comment_start = None;

                        if self.preserve_line_numbers {
                            result.push(' ');
                            result.push(' ');
                        }
                    } else if self.preserve_line_numbers {
                        result.push(' ');
                    }
                }
                _ => {
                    if in_comment {
                        if self.preserve_line_numbers {
                            if c == '\n' {
                                result.push('\n');
                            } else {
                                result.push(' ');
                            }
                        }
                    } else {
                        result.push(c);
                    }
                }
            }
        }

        if in_comment {
            return Err(PreprocessorError::UnterminatedComment {
                position: comment_start.unwrap_or(start_position),
            });
        }

        Ok(result)
    }
}

/// Состояние условной директивы.
#[derive(Debug, Clone)]
enum ConditionState {
    IfDef { active: bool },
    IfNDef { active: bool },
}

/// Результат обработки директивы.
#[derive(Debug)]
enum DirectiveResult {
    SkipLine,
    ProcessLine(String),
    #[allow(dead_code)]
    Continue,
}
