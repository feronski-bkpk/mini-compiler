//! Построение графа потока управления и работа с PHI-узлами

use super::basic_block::FunctionIR;
use super::ir_instructions::{IRInstruction, Operand};
use std::collections::{HashMap, HashSet, VecDeque};

/// Граф потока управления
pub struct ControlFlowGraph {
    /// Функция
    pub function: FunctionIR,
    /// Доминаторы для каждого блока
    pub dominators: HashMap<String, HashSet<String>>,
    /// Фронт доминаторов
    pub dominance_frontier: HashMap<String, HashSet<String>>,
}

impl ControlFlowGraph {
    /// Создает CFG для функции
    pub fn new(function: FunctionIR) -> Self {
        let mut cfg = Self {
            function,
            dominators: HashMap::new(),
            dominance_frontier: HashMap::new(),
        };
        cfg.compute_dominators();
        cfg.compute_dominance_frontier();
        cfg
    }

    /// Вычисляет доминаторы для всех блоков
    fn compute_dominators(&mut self) {
        let blocks: Vec<String> = self.function.blocks.keys().cloned().collect();
        let entry = self.function.entry_block.clone();

        for block in &blocks {
            let mut dom_set = HashSet::new();
            if block == &entry {
                dom_set.insert(entry.clone());
            } else {
                dom_set.extend(blocks.iter().cloned());
            }
            self.dominators.insert(block.clone(), dom_set);
        }

        let mut changed = true;
        while changed {
            changed = false;

            for block in &blocks {
                if block == &entry {
                    continue;
                }

                let preds: Vec<&String> = self
                    .function
                    .get_block(block)
                    .map(|b| b.predecessors.iter().collect())
                    .unwrap_or_default();

                if preds.is_empty() {
                    continue;
                }

                let mut new_dom = self.dominators[preds[0]].clone();
                for pred in &preds[1..] {
                    new_dom = &new_dom & &self.dominators[*pred];
                }
                new_dom.insert(block.clone());

                if &new_dom != self.dominators.get(block).unwrap() {
                    self.dominators.insert(block.clone(), new_dom);
                    changed = true;
                }
            }
        }
    }

    /// Вычисляет фронт доминаторов
    fn compute_dominance_frontier(&mut self) {
        for block in self.function.blocks.keys() {
            self.dominance_frontier
                .insert(block.clone(), HashSet::new());
        }

        for block in self.function.blocks.keys() {
            let block_dom = self.dominators.get(block).unwrap();

            for succ in self
                .function
                .get_block(block)
                .map(|b| b.successors.clone())
                .unwrap_or_default()
            {
                if !block_dom.contains(&succ) {
                    self.dominance_frontier.get_mut(block).unwrap().insert(succ);
                }
            }
        }
    }

    /// Добавляет PHI-узлы в точки слияния
    pub fn add_phi_nodes(&mut self, variable: &str) {
        let mut worklist: VecDeque<String> = VecDeque::new();
        let mut visited = HashSet::new();

        let defining_blocks: Vec<String> = self
            .function
            .blocks
            .iter()
            .filter(|(_, block)| {
                block.instructions.iter().any(|instr| match instr {
                    IRInstruction::Move(dest, _) => {
                        if let Operand::Variable(name) = dest {
                            name == variable
                        } else {
                            false
                        }
                    }
                    _ => false,
                })
            })
            .map(|(label, _)| label.clone())
            .collect();

        for block in defining_blocks {
            worklist.push_back(block.clone());
            visited.insert(block);
        }

        while let Some(block) = worklist.pop_front() {
            let frontier: Vec<String> = self
                .dominance_frontier
                .get(&block)
                .unwrap()
                .iter()
                .cloned()
                .collect();

            for df in frontier {
                if !self.has_phi_node(&df, variable) {
                    self.insert_phi_node(&df, variable);
                    if !visited.contains(&df) {
                        worklist.push_back(df.clone());
                        visited.insert(df.clone());
                    }
                }
            }
        }
    }

    /// Проверяет наличие PHI-узла для переменной в блоке
    fn has_phi_node(&self, block: &str, variable: &str) -> bool {
        if let Some(b) = self.function.get_block(block) {
            b.instructions.iter().any(|instr| match instr {
                IRInstruction::Phi(dest, _) => {
                    if let Operand::Variable(name) = dest {
                        name == variable
                    } else {
                        false
                    }
                }
                _ => false,
            })
        } else {
            false
        }
    }

    /// Вставляет PHI-узел в блок
    fn insert_phi_node(&mut self, block: &str, variable: &str) {
        let dest = Operand::Variable(variable.to_string());
        let phi = IRInstruction::Phi(dest, Vec::new());

        if let Some(b) = self.function.get_block_mut(block) {
            b.instructions.insert(0, phi);
        }
    }

    /// Заполняет аргументы PHI-узлов
    pub fn fill_phi_arguments(&mut self) {
        let blocks: Vec<String> = self.function.blocks.keys().cloned().collect();

        for block in blocks {
            let phi_nodes: Vec<_> = if let Some(b) = self.function.get_block(&block) {
                b.instructions
                    .iter()
                    .enumerate()
                    .filter_map(|(i, instr)| match instr {
                        IRInstruction::Phi(dest, _) => Some((i, dest.clone())),
                        _ => None,
                    })
                    .collect()
            } else {
                continue;
            };

            for (idx, dest) in phi_nodes {
                if let Operand::Variable(ref var_name) = dest {
                    let mut args = Vec::new();

                    if let Some(b) = self.function.get_block(&block) {
                        for pred in &b.predecessors {
                            let last_def = self.find_last_definition(pred, var_name);
                            if let Some(val) = last_def {
                                args.push((val, Operand::Label(pred.clone())));
                            }
                        }
                    }

                    if let Some(b) = self.function.get_block_mut(&block) {
                        if idx < b.instructions.len() {
                            b.instructions[idx] = IRInstruction::Phi(dest, args);
                        }
                    }
                }
            }
        }
    }

    /// Находит последнее определение переменной в блоке
    fn find_last_definition(&self, block: &str, var_name: &str) -> Option<Operand> {
        if let Some(b) = self.function.get_block(block) {
            for instr in b.instructions.iter().rev() {
                match instr {
                    IRInstruction::Move(dest, src) => {
                        if let Operand::Variable(name) = dest {
                            if name == var_name {
                                return Some(src.clone());
                            }
                        }
                    }
                    IRInstruction::Phi(dest, _) => {
                        if let Operand::Variable(name) = dest {
                            if name == var_name {
                                return Some(dest.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        None
    }

    /// Выполняет анализ потока данных (достигающие определения)
    pub fn reaching_definitions(&self) -> HashMap<String, HashSet<String>> {
        let mut reaching = HashMap::new();
        let blocks: Vec<String> = self.function.blocks.keys().cloned().collect();

        for block in &blocks {
            reaching.insert(block.clone(), HashSet::new());
        }

        let mut gen_set: HashMap<String, HashSet<String>> = HashMap::new();
        let mut kill: HashMap<String, HashSet<String>> = HashMap::new();

        for block in &blocks {
            let mut block_gen = HashSet::new();
            let mut block_kill = HashSet::new();

            if let Some(b) = self.function.get_block(block) {
                for instr in &b.instructions {
                    match instr {
                        IRInstruction::Move(dest, _) => {
                            if let Operand::Variable(name) = dest {
                                block_gen.insert(name.clone());
                                block_kill.insert(name.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }

            gen_set.insert(block.clone(), block_gen);
            kill.insert(block.clone(), block_kill);
        }

        let mut changed = true;
        while changed {
            changed = false;

            for block in &blocks {
                let mut new_in = HashSet::new();

                if let Some(b) = self.function.get_block(block) {
                    for pred in &b.predecessors {
                        if let Some(pred_out) = reaching.get(pred) {
                            new_in.extend(pred_out.iter().cloned());
                        }
                    }
                }

                let out: HashSet<String> = new_in
                    .difference(kill.get(block).unwrap())
                    .chain(gen_set.get(block).unwrap())
                    .cloned()
                    .collect();

                if out != *reaching.get(block).unwrap() {
                    reaching.insert(block.clone(), out);
                    changed = true;
                }
            }
        }

        reaching
    }
}

/// Строит CFG для функции
pub fn build_cfg(func_ir: &mut FunctionIR) {
    let blocks: Vec<String> = func_ir.blocks.keys().cloned().collect();

    for block_label in blocks {
        let terminator = if let Some(block) = func_ir.get_block(&block_label) {
            block.terminator().cloned()
        } else {
            None
        };

        let successors = match terminator {
            Some(IRInstruction::Jump(label)) => {
                if let Operand::Label(l) = label {
                    vec![l]
                } else {
                    vec![]
                }
            }
            Some(IRInstruction::JumpIf(_, label)) => {
                if let Operand::Label(l) = label {
                    vec![l]
                } else {
                    vec![]
                }
            }
            Some(IRInstruction::JumpIfNot(_, label)) => {
                if let Operand::Label(l) = label {
                    vec![l]
                } else {
                    vec![]
                }
            }
            Some(IRInstruction::Return(_)) => vec![],
            _ => vec![],
        };

        for succ in successors {
            func_ir
                .get_block_mut(&block_label)
                .unwrap()
                .add_successor(succ.clone());
            func_ir
                .get_block_mut(&succ)
                .unwrap()
                .add_predecessor(block_label.clone());
        }
    }
}
