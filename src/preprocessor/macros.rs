//! Таблица макросов для препроцессора.

use std::collections::{HashMap, HashSet};

use super::PreprocessorError;

/// Определение макроса.
#[derive(Debug, Clone)]
pub struct MacroDefinition {
    name: String,
    value: String,
}

impl MacroDefinition {
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

/// Таблица макросов для хранения и подстановки.
#[derive(Debug, Default)]
pub struct MacroTable {
    macros: HashMap<String, MacroDefinition>,
    expansion_stack: HashSet<String>,
}

impl MacroTable {
    pub fn new() -> Self {
        Self {
            macros: HashMap::new(),
            expansion_stack: HashSet::new(),
        }
    }

    /// Определяет новый макрос.
    pub fn define(&mut self, name: &str, value: &str) -> Result<(), PreprocessorError> {
        if !Self::is_valid_macro_name(name) {
            return Err(PreprocessorError::InvalidMacroName {
                name: name.to_string(),
            });
        }

        let definition = MacroDefinition::new(name, value);
        self.macros.insert(name.to_string(), definition);
        Ok(())
    }

    /// Удаляет определение макроса.
    pub fn undefine(&mut self, name: &str) {
        self.macros.remove(name);
    }

    /// Проверяет, определен ли макрос.
    pub fn is_defined(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    /// Расширяет макросы в строке.
    pub fn expand(&mut self, input: &str) -> Result<String, PreprocessorError> {
        let mut result = String::with_capacity(input.len());
        let mut current_word = String::new();
        let mut in_identifier = false;

        for c in input.chars() {
            if c.is_alphanumeric() || c == '_' {
                if !in_identifier {
                    in_identifier = true;
                    current_word.clear();
                }
                current_word.push(c);
                result.push(c);
            } else {
                if in_identifier {
                    in_identifier = false;
                    let macro_value = self
                        .macros
                        .get(&current_word)
                        .map(|m| m.value().to_string());

                    if let Some(value) = macro_value {
                        let word_len = current_word.len();
                        result.truncate(result.len() - word_len);

                        if self.expansion_stack.contains(&current_word) {
                            return Err(PreprocessorError::MacroRecursion {
                                name: current_word.clone(),
                            });
                        }

                        self.expansion_stack.insert(current_word.clone());

                        let expanded = self.expand(&value)?;
                        result.push_str(&expanded);

                        self.expansion_stack.remove(&current_word);
                    }
                }
                result.push(c);
            }
        }

        if in_identifier && !current_word.is_empty() {
            let macro_value = self
                .macros
                .get(&current_word)
                .map(|m| m.value().to_string());

            if let Some(value) = macro_value {
                let word_len = current_word.len();
                result.truncate(result.len() - word_len);

                if self.expansion_stack.contains(&current_word) {
                    return Err(PreprocessorError::MacroRecursion {
                        name: current_word.clone(),
                    });
                }

                self.expansion_stack.insert(current_word.clone());
                let expanded = self.expand(&value)?;
                result.push_str(&expanded);
                self.expansion_stack.remove(&current_word);
            }
        }

        Ok(result)
    }

    /// Проверяет валидность имени макроса.
    fn is_valid_macro_name(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        let first_char = name.chars().next().unwrap();
        if !first_char.is_alphabetic() && first_char != '_' {
            return false;
        }

        name.chars().all(|c| c.is_alphanumeric() || c == '_')
    }

    /// Возвращает итератор по всем определенным макросам.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &MacroDefinition)> {
        self.macros.iter()
    }
}
