//! Базовые блоки и представление функций в IR

use super::ir_instructions::{IRInstruction, Operand};
use std::collections::HashMap;

/// Базовый блок - последовательность инструкций без ветвлений
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Уникальная метка блока
    pub label: String,
    /// Инструкции в блоке
    pub instructions: Vec<IRInstruction>,
    /// Предшественники в CFG
    pub predecessors: Vec<String>,
    /// Последователи в CFG
    pub successors: Vec<String>,
}

impl BasicBlock {
    /// Создает новый базовый блок
    pub fn new(label: String) -> Self {
        Self {
            label,
            instructions: Vec::new(),
            predecessors: Vec::new(),
            successors: Vec::new(),
        }
    }

    /// Добавляет инструкцию в блок
    pub fn add_instruction(&mut self, instr: IRInstruction) {
        self.instructions.push(instr);
    }

    /// Добавляет предшественника
    pub fn add_predecessor(&mut self, pred: String) {
        if !self.predecessors.contains(&pred) {
            self.predecessors.push(pred);
        }
    }

    /// Добавляет последователя
    pub fn add_successor(&mut self, succ: String) {
        if !self.successors.contains(&succ) {
            self.successors.push(succ);
        }
    }

    /// Проверяет, является ли блок терминальным (без выхода)
    pub fn is_terminator(&self) -> bool {
        self.instructions
            .last()
            .map(|i| i.is_terminator())
            .unwrap_or(false)
    }

    /// Возвращает последнюю инструкцию (терминатор)
    pub fn terminator(&self) -> Option<&IRInstruction> {
        self.instructions.last().filter(|i| i.is_terminator())
    }
}

/// Функция в IR
#[derive(Debug, Clone)]
pub struct FunctionIR {
    /// Имя функции
    pub name: String,
    /// Возвращаемый тип
    pub return_type: String,
    /// Параметры (имя, тип)
    pub parameters: Vec<(String, String)>,
    /// Локальные переменные (имя, тип)
    pub locals: Vec<(String, String)>,
    /// Базовые блоки
    pub blocks: HashMap<String, BasicBlock>,
    /// Входной блок
    pub entry_block: String,
    /// Выходные блоки
    pub exit_blocks: Vec<String>,
}

impl FunctionIR {
    /// Создает новую функцию
    pub fn new(name: String, return_type: String) -> Self {
        Self {
            name,
            return_type,
            parameters: Vec::new(),
            locals: Vec::new(),
            blocks: HashMap::new(),
            entry_block: String::new(),
            exit_blocks: Vec::new(),
        }
    }

    /// Добавляет базовый блок
    pub fn add_block(&mut self, block: BasicBlock) {
        self.blocks.insert(block.label.clone(), block);
    }

    /// Получает базовый блок
    pub fn get_block(&self, label: &str) -> Option<&BasicBlock> {
        self.blocks.get(label)
    }

    /// Получает мутабельный базовый блок
    pub fn get_block_mut(&mut self, label: &str) -> Option<&mut BasicBlock> {
        self.blocks.get_mut(label)
    }

    /// Устанавливает входной блок
    pub fn set_entry(&mut self, label: String) {
        self.entry_block = label;
    }

    /// Добавляет выходной блок
    pub fn add_exit(&mut self, label: String) {
        if !self.exit_blocks.contains(&label) {
            self.exit_blocks.push(label);
        }
    }
}

/// Полная IR программа
#[derive(Debug, Clone)]
pub struct ProgramIR {
    /// Функции
    pub functions: HashMap<String, FunctionIR>,
    /// Глобальные переменные
    pub globals: Vec<(String, String)>,
}

impl ProgramIR {
    /// Создает новую программу
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            globals: Vec::new(),
        }
    }

    /// Добавляет функцию
    pub fn add_function(&mut self, func: FunctionIR) {
        self.functions.insert(func.name.clone(), func);
    }

    /// Получает функцию
    pub fn get_function(&self, name: &str) -> Option<&FunctionIR> {
        self.functions.get(name)
    }

    /// Получает мутабельную функцию
    pub fn get_function_mut(&mut self, name: &str) -> Option<&mut FunctionIR> {
        self.functions.get_mut(name)
    }

    /// Добавляет глобальную переменную
    pub fn add_global(&mut self, name: String, typ: String) {
        self.globals.push((name, typ));
    }
}

impl Default for ProgramIR {
    fn default() -> Self {
        Self::new()
    }
}

/// Статистика IR
#[derive(Debug, Default)]
pub struct IRStatistics {
    pub total_instructions: usize,
    pub instruction_counts: std::collections::HashMap<String, usize>,
    pub basic_block_count: usize,
    pub temporary_count: usize,
    pub max_stack_depth: usize,
}

impl IRStatistics {
    /// Вычисляет статистику для программы
    pub fn compute(program: &ProgramIR) -> Self {
        let mut stats = IRStatistics::default();
        let mut temp_set = std::collections::HashSet::new();

        for func in program.functions.values() {
            stats.basic_block_count += func.blocks.len();

            for block in func.blocks.values() {
                for instr in &block.instructions {
                    stats.total_instructions += 1;

                    let instr_name = match instr {
                        IRInstruction::Add(_, _, _) => "ADD",
                        IRInstruction::Sub(_, _, _) => "SUB",
                        IRInstruction::Mul(_, _, _) => "MUL",
                        IRInstruction::Div(_, _, _) => "DIV",
                        IRInstruction::Mod(_, _, _) => "MOD",
                        IRInstruction::Neg(_, _) => "NEG",
                        IRInstruction::And(_, _, _) => "AND",
                        IRInstruction::Or(_, _, _) => "OR",
                        IRInstruction::Not(_, _) => "NOT",
                        IRInstruction::Xor(_, _, _) => "XOR",
                        IRInstruction::CmpEq(_, _, _) => "CMP_EQ",
                        IRInstruction::CmpNe(_, _, _) => "CMP_NE",
                        IRInstruction::CmpLt(_, _, _) => "CMP_LT",
                        IRInstruction::CmpLe(_, _, _) => "CMP_LE",
                        IRInstruction::CmpGt(_, _, _) => "CMP_GT",
                        IRInstruction::CmpGe(_, _, _) => "CMP_GE",
                        IRInstruction::Load(_, _) => "LOAD",
                        IRInstruction::Store(_, _) => "STORE",
                        IRInstruction::Alloca(_, _) => "ALLOCA",
                        IRInstruction::Gep(_, _, _) => "GEP",
                        IRInstruction::Jump(_) => "JUMP",
                        IRInstruction::JumpIf(_, _) => "JUMP_IF",
                        IRInstruction::JumpIfNot(_, _) => "JUMP_IF_NOT",
                        IRInstruction::Label(_) => "LABEL",
                        IRInstruction::Phi(_, _) => "PHI",
                        IRInstruction::Call(_, _, _) => "CALL",
                        IRInstruction::Return(_) => "RETURN",
                        IRInstruction::Param(_, _) => "PARAM",
                        IRInstruction::Move(_, _) => "MOVE",
                    };

                    *stats
                        .instruction_counts
                        .entry(instr_name.to_string())
                        .or_insert(0) += 1;

                    for op in instr.operands() {
                        if let Operand::Temporary(name) = op {
                            temp_set.insert(name.clone());
                        }
                    }
                    match instr {
                        IRInstruction::Add(dest, _, _)
                        | IRInstruction::Sub(dest, _, _)
                        | IRInstruction::Mul(dest, _, _)
                        | IRInstruction::Div(dest, _, _)
                        | IRInstruction::Mod(dest, _, _)
                        | IRInstruction::Neg(dest, _)
                        | IRInstruction::And(dest, _, _)
                        | IRInstruction::Or(dest, _, _)
                        | IRInstruction::Not(dest, _)
                        | IRInstruction::Xor(dest, _, _)
                        | IRInstruction::CmpEq(dest, _, _)
                        | IRInstruction::CmpNe(dest, _, _)
                        | IRInstruction::CmpLt(dest, _, _)
                        | IRInstruction::CmpLe(dest, _, _)
                        | IRInstruction::CmpGt(dest, _, _)
                        | IRInstruction::CmpGe(dest, _, _)
                        | IRInstruction::Load(dest, _)
                        | IRInstruction::Alloca(dest, _)
                        | IRInstruction::Gep(dest, _, _)
                        | IRInstruction::Call(dest, _, _)
                        | IRInstruction::Move(dest, _) => {
                            if let Operand::Temporary(name) = dest {
                                temp_set.insert(name.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        stats.temporary_count = temp_set.len();
        stats
    }
}
