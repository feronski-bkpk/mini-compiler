//! Менеджер меток для кодогенератора
//!
//! Обеспечивает генерацию уникальных меток для базовых блоков,
//! условных переходов, циклов и других управляющих конструкций.
//!
//! Основная функциональность встроена в `ir_generator.rs`.
//! Этот модуль предоставляет независимый интерфейс для
//! будущего расширения.

/// Менеджер меток с поддержкой вложенных контекстов
pub struct LabelManager {
    counter: usize,
    prefix: String,
    context_stack: Vec<String>,
}

impl LabelManager {
    pub fn new(prefix: &str) -> Self {
        Self {
            counter: 0,
            prefix: prefix.to_string(),
            context_stack: Vec::new(),
        }
    }

    /// Входит в новый контекст (например, цикл)
    pub fn push_context(&mut self, name: &str) {
        self.context_stack.push(name.to_string());
    }

    /// Выходит из текущего контекста
    pub fn pop_context(&mut self) {
        self.context_stack.pop();
    }

    /// Генерирует метку с учетом контекста
    pub fn generate_label(&mut self, suffix: &str) -> String {
        self.counter += 1;
        let context = self.context_stack.join("_");
        if context.is_empty() {
            format!("{}_{:03}_{}", self.prefix, self.counter, suffix)
        } else {
            format!("{}_{}_{:03}_{}", self.prefix, context, self.counter, suffix)
        }
    }
}

impl Default for LabelManager {
    fn default() -> Self {
        Self::new("L")
    }
}
