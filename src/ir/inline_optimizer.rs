//! Оптимизатор встраивания функций (function inlining)
//!
//! Заменяет вызовы маленьких функций на тело функции,
//! уменьшая накладные расходы на call/ret.

use super::basic_block::{BasicBlock, FunctionIR, ProgramIR};
use super::ir_instructions::{IRInstruction, Operand};
use std::collections::{HashMap, HashSet};

/// Статистика инлайнинга
#[derive(Debug, Default)]
pub struct InlineStats {
    pub functions_considered: usize,
    pub functions_inlined: usize,
    pub call_sites_inlined: usize,
    pub instructions_before: usize,
    pub instructions_after: usize,
}

/// Оптимизатор встраивания функций
pub struct InlineOptimizer {
    /// Максимальное количество IR-инструкций для инлайнинга
    max_instructions: usize,
    /// Максимальная глубина рекурсивного инлайнинга
    max_recursion_depth: usize,
    /// Счётчик для генерации уникальных имён
    rename_counter: usize,
    /// Статистика
    pub stats: InlineStats,
}

impl InlineOptimizer {
    pub fn new() -> Self {
        Self {
            max_instructions: 20,
            max_recursion_depth: 0,
            rename_counter: 0,
            stats: InlineStats::default(),
        }
    }

    /// Установить максимальное количество инструкций
    pub fn with_max_instructions(mut self, max: usize) -> Self {
        self.max_instructions = max;
        self
    }

    /// Основной метод оптимизации
    pub fn optimize(&mut self, program: &mut ProgramIR) -> InlineStats {
        self.stats = InlineStats::default();
        self.rename_counter = 0;

        self.stats.instructions_before = Self::count_instructions(program);

        let mut changed = true;
        let mut iteration = 0;
        const MAX_ITERATIONS: usize = 3;

        while changed && iteration < MAX_ITERATIONS {
            changed = false;
            iteration += 1;

            let candidates = self.find_candidates(program);

            for callee_name in &candidates {
                let call_sites = self.find_call_sites(program, callee_name);
                if let Some((caller_name, call_instr_idx, block_label)) = call_sites.first() {
                    let inlined = self.inline_call(
                        program,
                        caller_name,
                        callee_name,
                        block_label,
                        *call_instr_idx,
                    );
                    if inlined {
                        changed = true;
                        self.stats.call_sites_inlined += 1;
                        break;
                    }
                }
            }
        }

        self.remove_unused_functions(program);

        self.stats.functions_inlined = self.stats.call_sites_inlined;
        self.stats.instructions_after = Self::count_instructions(program);

        self.stats.clone()
    }

    /// Считает общее количество IR-инструкций в программе
    fn count_instructions(program: &ProgramIR) -> usize {
        program
            .functions
            .iter()
            .map(|f| {
                f.blocks
                    .values()
                    .map(|b| b.instructions.len())
                    .sum::<usize>()
            })
            .sum()
    }

    /// Находит функции-кандидаты для инлайнинга
    fn find_candidates(&mut self, program: &ProgramIR) -> Vec<String> {
        let mut candidates = Vec::new();

        for func in &program.functions {
            if func.blocks.is_empty() {
                continue;
            }

            let instr_count: usize = func.blocks.values().map(|b| b.instructions.len()).sum();

            if instr_count > self.max_instructions {
                continue;
            }

            if self.is_recursive(func, program) && self.max_recursion_depth == 0 {
                continue;
            }

            let has_loops = func.blocks.keys().any(|k| 
                k.contains("for_") || k.contains("while_"));
            if has_loops {
                continue;
            }

            let has_addrof = func.blocks.values().any(|b| 
                b.instructions.iter().any(|i| matches!(i, IRInstruction::AddrOf(_, _))));
            if has_addrof {
                continue;
            }

            if func.name == "swap" {
                continue;
            }

            self.stats.functions_considered += 1;
            candidates.push(func.name.clone());
        }

        candidates
    }

    /// Проверяет, является ли функция рекурсивной
    fn is_recursive(&self, func: &FunctionIR, _program: &ProgramIR) -> bool {
        let name = &func.name;
        for block in func.blocks.values() {
            for instr in &block.instructions {
                if let IRInstruction::Call(_, callee, _) = instr {
                    if let Operand::Label(n) = callee {
                        if n == name {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Находит все места вызова указанной функции
    fn find_call_sites(
        &self,
        program: &ProgramIR,
        callee_name: &str,
    ) -> Vec<(String, usize, String)> {
        let mut sites = Vec::new();

        for func in &program.functions {
            if func.name == callee_name || func.name.contains("_inl") {
                continue;
            }
            for (block_label, block) in &func.blocks {
                if block_label.contains("_inl") 
                    || block_label.contains("__after_inline") 
                    || block_label.contains("__inline_return") {
                    continue;
                }
                for (idx, instr) in block.instructions.iter().enumerate() {
                    if let IRInstruction::Call(_, callee, _) = instr {
                        if let Operand::Label(n) = callee {
                            if n == callee_name {
                                sites.push((func.name.clone(), idx, block_label.clone()));
                            }
                        }
                    }
                }
            }
        }

        sites
    }

    /// Встраивает вызов функции в место вызова
    fn inline_call(
        &mut self,
        program: &mut ProgramIR,
        caller_name: &str,
        callee_name: &str,
        block_label: &str,
        call_instr_idx: usize,
    ) -> bool {
        let callee = match program.functions.iter().find(|f| f.name == *callee_name) {
            Some(f) => f.clone(),
            None => return false,
        };

        let caller = match program
            .functions
            .iter_mut()
            .find(|f| f.name == *caller_name)
        {
            Some(f) => f,
            None => return false,
        };

        let (call_dest, call_args) = match &caller.blocks[block_label].instructions[call_instr_idx]
        {
            IRInstruction::Call(dest, _, args) => (dest.clone(), args.clone()),
            _ => return false,
        };

        let mut name_map: HashMap<String, String> = HashMap::new();
        for (i, (param_name, _)) in callee.parameters.iter().enumerate() {
            if i < call_args.len() {
                let arg_name = self.operand_to_string(&call_args[i]);
                name_map.insert(param_name.clone(), arg_name.clone());
            }
        }

        for (local_name, _) in &callee.locals {
            let new_name = self.rename_var(local_name);
            name_map.insert(local_name.clone(), new_name);
        }

        let suffix = self.rename_counter;
        let after_call_label = format!("__after_inline_{}", suffix);
        let return_label = format!("__inline_return_{}", suffix);

        let mut block_name_map: HashMap<String, String> = HashMap::new();
        for orig_label in callee.blocks.keys() {
            block_name_map.insert(orig_label.clone(), self.rename_label(orig_label));
        }

        let original_block = caller.blocks.get(block_label).unwrap().clone();
        let mut before_call = BasicBlock::new(block_label.to_string());
        let mut after_call = BasicBlock::new(after_call_label.clone());

        for (i, instr) in original_block.instructions.iter().enumerate() {
            if i < call_instr_idx {
                if !matches!(instr, IRInstruction::Param(_, _)) {
                    before_call.add_instruction(instr.clone());
                }
            } else if i > call_instr_idx {
                after_call.add_instruction(instr.clone());
            }
        }

        for (i, (param_name, _)) in callee.parameters.iter().enumerate() {
            if i < call_args.len() {
                let temp_name = self.rename_var(param_name);
                name_map.insert(param_name.clone(), temp_name.clone());
                caller.locals.push((temp_name.clone(), "int".to_string()));
                before_call.add_instruction(IRInstruction::Move(
                    Operand::Variable(temp_name),
                    call_args[i].clone(),
                ));
            }
        }

        let entry_label = block_name_map.get(&callee.entry_block).unwrap().clone();
        before_call.add_instruction(IRInstruction::Jump(Operand::Label(entry_label.clone())));

        let mut new_blocks: Vec<BasicBlock> = Vec::new();
        for (orig_label, block) in &callee.blocks {
            let new_label = block_name_map.get(orig_label).unwrap().clone();
            let mut new_block = BasicBlock::new(new_label);

            for instr in &block.instructions {
                let new_instr = self.rename_instruction(
                    instr,
                    &name_map,
                    &block_name_map,
                    &call_dest,
                    &after_call_label,
                    &return_label,
                );
                new_block.add_instruction(new_instr);
            }

            if !new_block.is_terminator() {
                new_block.add_instruction(IRInstruction::Jump(Operand::Label(
                    after_call_label.clone(),
                )));
            }

            new_blocks.push(new_block);
        }

        let mut return_block = BasicBlock::new(return_label.clone());
        return_block.add_instruction(IRInstruction::Jump(Operand::Label(
            after_call_label.clone(),
        )));
        new_blocks.push(return_block);

        for (local_name, local_type) in &callee.locals {
            let new_name = name_map.get(local_name).unwrap().clone();
            if !caller.locals.iter().any(|(n, _)| n == &new_name) {
                caller.locals.push((new_name, local_type.clone()));
            }
        }

        caller.blocks.remove(block_label);
        caller.add_block(before_call);
        caller.add_block(after_call);

        for block in &new_blocks {
            caller.add_block(block.clone());
        }

        for block in &new_blocks {
            if block.label != after_call_label && block.label != return_label {
                let successors: Vec<String> = block
                    .instructions
                    .iter()
                    .filter_map(|i| match i {
                        IRInstruction::Jump(l)
                        | IRInstruction::JumpIf(_, l)
                        | IRInstruction::JumpIfNot(_, l) => {
                            if let Operand::Label(n) = l {
                                Some(n.clone())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
                    .collect();
                if let Some(b) = caller.blocks.get_mut(&block.label) {
                    b.successors = successors;
                }
            }
        }

        self.stats.call_sites_inlined += 1;
        true
    }

    /// Переименовывает переменную для уникальности
    fn rename_var(&mut self, name: &str) -> String {
        self.rename_counter += 1;
        format!("{}_{}", name, self.rename_counter)
    }

    /// Переименовывает метку для уникальности
    fn rename_label(&mut self, label: &str) -> String {
        self.rename_counter += 1;
        format!("{}_inl{}", label, self.rename_counter)
    }

    /// Переименовывает операнды в инструкции
    fn rename_instruction(
        &self,
        instr: &IRInstruction,
        name_map: &HashMap<String, String>,
        _block_map: &HashMap<String, String>,
        return_dest: &Operand,
        after_label: &str,
        return_label: &str,
    ) -> IRInstruction {
        let rename_op = |op: &Operand| -> Operand {
            match op {
                Operand::Variable(n) | Operand::Temporary(n) => {
                    if let Some(new_name) = name_map.get(n) {
                        Operand::Variable(new_name.clone())
                    } else {
                        op.clone()
                    }
                }
                _ => op.clone(),
            }
        };

        match instr {
            IRInstruction::Return(Some(val)) => {
                IRInstruction::Move(return_dest.clone(), rename_op(val))
            }
            IRInstruction::Return(None) => {
                IRInstruction::Jump(Operand::Label(return_label.to_string()))
            }
            IRInstruction::Move(d, s) => IRInstruction::Move(rename_op(d), rename_op(s)),
            IRInstruction::Add(d, l, r) => {
                IRInstruction::Add(rename_op(d), rename_op(l), rename_op(r))
            }
            IRInstruction::Sub(d, l, r) => {
                IRInstruction::Sub(rename_op(d), rename_op(l), rename_op(r))
            }
            IRInstruction::Mul(d, l, r) => {
                IRInstruction::Mul(rename_op(d), rename_op(l), rename_op(r))
            }
            IRInstruction::Div(d, l, r) => {
                IRInstruction::Div(rename_op(d), rename_op(l), rename_op(r))
            }
            IRInstruction::Mod(d, l, r) => {
                IRInstruction::Mod(rename_op(d), rename_op(l), rename_op(r))
            }
            IRInstruction::Neg(d, s) => IRInstruction::Neg(rename_op(d), rename_op(s)),
            IRInstruction::And(d, l, r) => {
                IRInstruction::And(rename_op(d), rename_op(l), rename_op(r))
            }
            IRInstruction::Or(d, l, r) => {
                IRInstruction::Or(rename_op(d), rename_op(l), rename_op(r))
            }
            IRInstruction::Not(d, s) => IRInstruction::Not(rename_op(d), rename_op(s)),
            IRInstruction::Xor(d, l, r) => {
                IRInstruction::Xor(rename_op(d), rename_op(l), rename_op(r))
            }
            IRInstruction::Load(d, a) => IRInstruction::Load(rename_op(d), rename_op(a)),
            IRInstruction::Store(a, s) => IRInstruction::Store(rename_op(a), rename_op(s)),
            IRInstruction::ArrayLoad(d, b, i) => {
                IRInstruction::ArrayLoad(rename_op(d), rename_op(b), rename_op(i))
            }
            IRInstruction::ArrayStore(b, i, v) => {
                IRInstruction::ArrayStore(rename_op(b), rename_op(i), rename_op(v))
            }
            IRInstruction::Call(d, f, args) => {
                let new_args: Vec<Operand> = args.iter().map(|a| rename_op(a)).collect();
                IRInstruction::Call(rename_op(d), f.clone(), new_args)
            }
            IRInstruction::CmpEq(d, l, r) => {
                IRInstruction::CmpEq(rename_op(d), rename_op(l), rename_op(r))
            }
            IRInstruction::CmpNe(d, l, r) => {
                IRInstruction::CmpNe(rename_op(d), rename_op(l), rename_op(r))
            }
            IRInstruction::CmpLt(d, l, r) => {
                IRInstruction::CmpLt(rename_op(d), rename_op(l), rename_op(r))
            }
            IRInstruction::CmpLe(d, l, r) => {
                IRInstruction::CmpLe(rename_op(d), rename_op(l), rename_op(r))
            }
            IRInstruction::CmpGt(d, l, r) => {
                IRInstruction::CmpGt(rename_op(d), rename_op(l), rename_op(r))
            }
            IRInstruction::CmpGe(d, l, r) => {
                IRInstruction::CmpGe(rename_op(d), rename_op(l), rename_op(r))
            }
            IRInstruction::IntToFloat(d, s) => {
                IRInstruction::IntToFloat(rename_op(d), rename_op(s))
            }
            IRInstruction::FloatToInt(d, s) => {
                IRInstruction::FloatToInt(rename_op(d), rename_op(s))
            }
            IRInstruction::CmpJmp(l, r, _tl, _fl, c, j, f) => IRInstruction::CmpJmp(
                rename_op(l),
                rename_op(r),
                Operand::Label(after_label.to_string()),
                Operand::Label(return_label.to_string()),
                c.clone(),
                j.clone(),
                *f,
            ),
            IRInstruction::Jump(l) => IRInstruction::Jump(l.clone()),
            IRInstruction::JumpIf(c, l) => IRInstruction::JumpIf(rename_op(c), l.clone()),
            IRInstruction::JumpIfNot(c, l) => IRInstruction::JumpIfNot(rename_op(c), l.clone()),
            IRInstruction::Label(l) => IRInstruction::Label(l.clone()),
            IRInstruction::Param(_, _) => instr.clone(),
            _ => instr.clone(),
        }
    }

    /// Удаляет функции, которые больше никем не вызываются (кроме main)
    pub fn remove_unused_functions(&self, program: &mut ProgramIR) {
        let used: HashSet<String> = program
            .functions
            .iter()
            .flat_map(|f| {
                f.blocks.values().flat_map(|b| {
                    b.instructions.iter().filter_map(|i| {
                        if let IRInstruction::Call(_, callee, _) = i {
                            if let Operand::Label(n) = callee {
                                Some(n.clone())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                })
            })
            .collect();

        program
            .functions
            .retain(|f| f.name == "main" || used.contains(&f.name) || f.blocks.is_empty());
    }

    /// Преобразует операнд в строку
    fn operand_to_string(&self, op: &Operand) -> String {
        match op {
            Operand::Variable(n) | Operand::Temporary(n) => n.clone(),
            Operand::IntLiteral(v) => v.to_string(),
            _ => format!("{}", op),
        }
    }
}

impl Clone for InlineStats {
    fn clone(&self) -> Self {
        Self {
            functions_considered: self.functions_considered,
            functions_inlined: self.functions_inlined,
            call_sites_inlined: self.call_sites_inlined,
            instructions_before: self.instructions_before,
            instructions_after: self.instructions_after,
        }
    }
}
