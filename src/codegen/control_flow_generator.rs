//! Генератор управляющих конструкций (if/else, while, for, switch)
//!
//! Этот модуль является частью кодогенератора и отвечает за
//! трансляцию управляющих конструкций из IR в x86-64 ассемблер.
//!
//! Основная логика реализована в `x86_generator.rs` и `ir_generator.rs`.
//! В будущем этот модуль может быть расширен для:
//! - Генерации jump tables для switch
//! - Оптимизации условных переходов
//! - Анализа достижимости кода

/// Тип управляющей конструкции
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControlFlowType {
    If,
    IfElse,
    While,
    For,
    Switch,
    Break,
    Continue,
}

/// Генератор меток для управляющих конструкций
pub struct LabelManager {
    counter: usize,
    function_id: usize,
}

impl LabelManager {
    pub fn new(function_id: usize) -> Self {
        Self {
            counter: 0,
            function_id,
        }
    }

    /// Генерирует уникальную метку с префиксом функции
    pub fn next_label(&mut self) -> String {
        self.counter += 1;
        format!("L{}_{:03}", self.function_id, self.counter)
    }
}

impl Default for LabelManager {
    fn default() -> Self {
        Self::new(0)
    }
}
