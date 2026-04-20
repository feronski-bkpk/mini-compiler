//! Управление стековым фреймом для x86-64
//!
//! Реализует выделение стековых слотов для локальных переменных
//! и временных значений в соответствии с System V AMD64 ABI.

use std::collections::HashMap;

/// Менеджер стекового фрейма
#[derive(Debug, Clone)]
pub struct StackFrameManager {
    /// Текущая функция
    current_function: Option<String>,
    /// Размер фрейма в байтах (выровнен по 16)
    frame_size: usize,
    /// Смещения для локальных переменных (имя -> смещение от rbp)
    var_offsets: HashMap<String, i32>,
    /// Смещения для временных переменных (имя -> смещение от rbp)
    temp_offsets: HashMap<String, i32>,
    /// Следующее доступное смещение для переменных
    next_var_offset: i32,
    /// Следующее доступное смещение для временных переменных
    next_temp_offset: i32,
    /// Сохраняемые регистры (callee-saved)
    saved_registers: Vec<String>,
    /// Красная зона используется?
    use_red_zone: bool,
    /// Функция является листовой (не вызывает другие функции)
    is_leaf: bool,
}

impl StackFrameManager {
    /// Создает новый менеджер стекового фрейма
    pub fn new() -> Self {
        Self {
            current_function: None,
            frame_size: 0,
            var_offsets: HashMap::new(),
            temp_offsets: HashMap::new(),
            next_var_offset: 0,
            next_temp_offset: 0,
            saved_registers: Vec::new(),
            use_red_zone: false,
            is_leaf: true,
        }
    }

    /// Начинает новую функцию
    pub fn begin_function(&mut self, name: &str) {
        self.current_function = Some(name.to_string());
        self.frame_size = 0;
        self.var_offsets.clear();
        self.temp_offsets.clear();
        self.next_var_offset = 0;
        self.next_temp_offset = 0;
        self.saved_registers.clear();
        self.use_red_zone = false;
        self.is_leaf = true;
    }

    /// Завершает текущую функцию
    pub fn end_function(&mut self) {
        self.current_function = None;
    }

    /// Устанавливает, является ли функция листовой
    pub fn set_leaf(&mut self, is_leaf: bool) {
        self.is_leaf = is_leaf;
        self.use_red_zone = is_leaf;
    }

    /// Выделяет слот для локальной переменной
    pub fn allocate_var(&mut self, name: &str, size: usize) -> i32 {
        let alignment = self.alignment_for_size(size);
        self.next_var_offset =
            (self.next_var_offset + alignment as i32 - 1) & !(alignment as i32 - 1);

        let offset = -self.next_var_offset - size as i32;
        self.var_offsets.insert(name.to_string(), offset);

        self.next_var_offset += size as i32;
        offset
    }

    /// Выделяет слот для временной переменной
    pub fn allocate_temp(&mut self, name: &str, size: usize) -> i32 {
        let alignment = self.alignment_for_size(size);
        self.next_temp_offset =
            (self.next_temp_offset + alignment as i32 - 1) & !(alignment as i32 - 1);

        let offset = -self.next_var_offset - self.next_temp_offset - size as i32;
        self.temp_offsets.insert(name.to_string(), offset);

        self.next_temp_offset += size as i32;
        offset
    }

    /// Возвращает смещение переменной
    pub fn get_var_offset(&self, name: &str) -> Option<i32> {
        self.var_offsets.get(name).copied()
    }

    /// Возвращает смещение временной переменной
    pub fn get_temp_offset(&self, name: &str) -> Option<i32> {
        self.temp_offsets.get(name).copied()
    }

    /// Устанавливает размер фрейма
    pub fn set_frame_size(&mut self, size: usize) {
        self.frame_size = size;
    }

    /// Возвращает текущий размер фрейма
    pub fn current_frame_size(&self) -> usize {
        self.frame_size
    }

    /// Вычисляет общий размер фрейма
    pub fn compute_frame_size(&self) -> usize {
        let var_size = self.next_var_offset as usize;
        let temp_size = self.next_temp_offset as usize;
        let saved_regs_size = self.saved_registers.len() * 8;

        let total = var_size + temp_size + saved_regs_size;

        (total + 15) & !15
    }

    /// Добавляет сохраняемый регистр
    pub fn save_register(&mut self, reg: &str) {
        if !self.saved_registers.contains(&reg.to_string()) {
            self.saved_registers.push(reg.to_string());
        }
    }

    /// Возвращает список сохраняемых регистров
    pub fn get_saved_registers(&self) -> &[String] {
        &self.saved_registers
    }

    /// Генерирует пролог функции
    pub fn generate_prologue(&self) -> String {
        let mut output = String::new();

        output.push_str("    push rbp\n");
        output.push_str("    mov rbp, rsp\n");

        for reg in &self.saved_registers {
            output.push_str(&format!("    push {}\n", reg));
        }

        if self.frame_size > 0 {
            output.push_str(&format!("    sub rsp, {}\n", self.frame_size));
        }

        output
    }

    /// Генерирует эпилог функции
    pub fn generate_epilogue(&self, has_return_value: bool) -> String {
        let mut output = String::new();

        if !has_return_value {
            output.push_str("    mov rsp, rbp\n");
            output.push_str("    pop rbp\n");
            output.push_str("    ret\n");
        } else {
            for reg in self.saved_registers.iter().rev() {
                output.push_str(&format!("    pop {}\n", reg));
            }
            output.push_str("    mov rsp, rbp\n");
            output.push_str("    pop rbp\n");
            output.push_str("    ret\n");
        }

        output
    }

    /// Проверяет, можно ли использовать красную зону
    pub fn can_use_red_zone(&self) -> bool {
        self.use_red_zone && self.is_leaf && self.frame_size <= 128
    }

    /// Генерирует пролог с учетом красной зоны
    pub fn generate_prologue_with_red_zone(&self, is_leaf: bool, frame_size: usize) -> String {
        let mut output = String::new();

        output.push_str("    push rbp\n");
        output.push_str("    mov rbp, rsp\n");

        if is_leaf && frame_size <= 128 {
            output.push_str("    ; Using red zone (128 bytes below rsp)\n");
        } else if frame_size > 0 {
            output.push_str(&format!("    sub rsp, {}\n", frame_size));
        }

        output
    }

    /// Генерирует эпилог с учетом красной зоны
    pub fn generate_epilogue_with_red_zone(
        &self,
        is_leaf: bool,
        frame_size: usize,
        has_return_value: bool,
    ) -> String {
        let mut output = String::new();

        if !is_leaf || frame_size > 128 {
            if frame_size > 0 {
                output.push_str(&format!("    add rsp, {}\n", frame_size));
            }
        }

        if !has_return_value {
            output.push_str("    pop rbp\n");
            output.push_str("    ret\n");
        } else {
            output.push_str("    mov rsp, rbp\n");
            output.push_str("    pop rbp\n");
            output.push_str("    ret\n");
        }

        output
    }

    /// Возвращает выравнивание для заданного размера
    fn alignment_for_size(&self, size: usize) -> usize {
        match size {
            1 => 1,
            2 => 2,
            4 => 4,
            8 => 8,
            _ => 8,
        }
    }

    /// Дамп информации о стековом фрейме
    pub fn dump(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("Функция: {:?}\n", self.current_function));
        output.push_str(&format!("Размер фрейма: {} байт\n", self.frame_size));
        output.push_str(&format!("Листовая функция: {}\n", self.is_leaf));
        output.push_str(&format!("Использует красную зону: {}\n", self.use_red_zone));
        output.push_str("\nЛокальные переменные:\n");

        let mut vars: Vec<(&String, &i32)> = self.var_offsets.iter().collect();
        vars.sort_by_key(|(_, offset)| *offset);

        for (name, offset) in vars {
            output.push_str(&format!(
                "  {}: [rbp{}]\n",
                name,
                self.format_offset(*offset)
            ));
        }

        output.push_str("\nВременные переменные:\n");
        let mut temps: Vec<(&String, &i32)> = self.temp_offsets.iter().collect();
        temps.sort_by_key(|(_, offset)| *offset);

        for (name, offset) in temps {
            output.push_str(&format!(
                "  {}: [rbp{}]\n",
                name,
                self.format_offset(*offset)
            ));
        }

        if !self.saved_registers.is_empty() {
            output.push_str("\nСохраненные регистры:\n");
            for reg in &self.saved_registers {
                output.push_str(&format!("  {}\n", reg));
            }
        }

        output
    }

    /// Форматирует смещение для вывода
    fn format_offset(&self, offset: i32) -> String {
        if offset < 0 {
            format!("-{}", -offset)
        } else {
            format!("+{}", offset)
        }
    }
}

impl Default for StackFrameManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_allocation() {
        let mut manager = StackFrameManager::new();
        manager.begin_function("test");

        let offset1 = manager.allocate_var("x", 4);
        let offset2 = manager.allocate_var("y", 8);
        let offset3 = manager.allocate_var("z", 4);

        assert!(offset1 < 0);
        assert!(offset2 < 0);
        assert!(offset3 < 0);

        assert!(offset1 > offset2);
        assert!(offset2 > offset3);

        assert_eq!(manager.get_var_offset("x"), Some(offset1));
        assert_eq!(manager.get_var_offset("y"), Some(offset2));
        assert_eq!(manager.get_var_offset("z"), Some(offset3));
    }

    #[test]
    fn test_frame_size_computation() {
        let mut manager = StackFrameManager::new();
        manager.begin_function("test");

        manager.allocate_var("x", 4);
        manager.allocate_var("y", 8);

        let size = manager.compute_frame_size();
        assert_eq!(size, 16);
    }

    #[test]
    fn test_frame_size_with_alignment() {
        let mut manager = StackFrameManager::new();
        manager.begin_function("test");

        manager.allocate_var("a", 1);
        manager.allocate_var("b", 8);
        manager.allocate_var("c", 4);

        let size = manager.compute_frame_size();
        assert_eq!(size, 32);
    }

    #[test]
    fn test_temp_allocation() {
        let mut manager = StackFrameManager::new();
        manager.begin_function("test");

        manager.allocate_var("x", 4);
        let temp_offset = manager.allocate_temp("t1", 4);

        assert!(temp_offset < 0);
        let var_offset = manager.get_var_offset("x").unwrap();
        assert!(temp_offset < var_offset);
    }

    #[test]
    fn test_saved_registers() {
        let mut manager = StackFrameManager::new();
        manager.begin_function("test");

        manager.save_register("rbx");
        manager.save_register("r12");
        manager.save_register("rbx");

        assert_eq!(manager.get_saved_registers().len(), 2);
        assert_eq!(manager.get_saved_registers()[0], "rbx");
        assert_eq!(manager.get_saved_registers()[1], "r12");

        let size = manager.compute_frame_size();
        assert_eq!(size, 16);
    }
}
