//! Peephole оптимизатор для IR

use super::basic_block::ProgramIR;
use super::ir_instructions::{IRInstruction, Operand};
use std::collections::HashSet;

/// Отчет об оптимизациях
#[derive(Debug, Default)]
pub struct OptimizationReport {
    pub changes_made: usize,
    pub instructions_removed: usize,
    pub instructions_added: usize,
    pub simplifications_applied: usize,
    pub dead_code_removed: usize,
    pub licm_moved: usize,
}

impl OptimizationReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, other: &OptimizationReport) {
        self.changes_made += other.changes_made;
        self.instructions_removed += other.instructions_removed;
        self.instructions_added += other.instructions_added;
        self.simplifications_applied += other.simplifications_applied;
        self.dead_code_removed += other.dead_code_removed;
        self.licm_moved += other.licm_moved;
    }
}

pub struct PeepholeOptimizer;

impl PeepholeOptimizer {
    pub fn optimize(program: &mut ProgramIR) -> OptimizationReport {
        let mut total_report = OptimizationReport::new();

        let mut changed = true;
        let mut iteration = 0;
        const MAX_ITERATIONS: usize = 10;

        while changed && iteration < MAX_ITERATIONS {
            changed = false;
            iteration += 1;

            for func in program.functions.iter_mut() {
                let report = Self::optimize_function(func);
                if report.changes_made > 0 {
                    changed = true;
                    total_report.add(&report);
                }
            }
        }

        for func in program.functions.iter_mut() {
            let report = Self::optimize_loops(func);
            if report.changes_made > 0 {
                total_report.add(&report);
            }
        }

        total_report
    }

    fn optimize_function(func: &mut super::basic_block::FunctionIR) -> OptimizationReport {
        let mut report = OptimizationReport::new();

        for block in func.blocks.values_mut() {
            let block_report = Self::optimize_block(block);
            if block_report.changes_made > 0 {
                report.add(&block_report);
            }
        }

        report
    }

    /// Оптимизация циклов: вынос инвариантных инструкций
    fn optimize_loops(func: &mut super::basic_block::FunctionIR) -> OptimizationReport {
        let mut report = OptimizationReport::new();

        let cond_blocks: Vec<String> = func
            .blocks
            .keys()
            .filter(|k| k.contains("while_cond") || k.contains("for_cond"))
            .cloned()
            .collect();

        for cond_label in &cond_blocks {
            let body_label = cond_label
                .replace("while_cond", "while_body")
                .replace("for_cond", "for_body");
            let _end_label = cond_label
                .replace("while_cond", "while_end")
                .replace("for_cond", "for_end");

            let mut invoke_instructions: Vec<(usize, IRInstruction)> = Vec::new();

            if let Some(body_block) = func.blocks.get(&body_label).cloned() {
                for (i, instr) in body_block.instructions.iter().enumerate() {
                    if Self::is_loop_invariant(instr, cond_label, &func.blocks) {
                        invoke_instructions.push((i, instr.clone()));
                    }
                }
            }

            if !invoke_instructions.is_empty() {
                let mut preheader = cond_label.clone();
                if let Some(cond_block) = func.blocks.get(cond_label) {
                    for pred in &cond_block.predecessors {
                        if !pred.contains("while_body")
                            && !pred.contains("for_body")
                            && !pred.contains("for_update")
                        {
                            preheader = pred.clone();
                            break;
                        }
                    }
                }

                if let Some(target_block) = func.blocks.get_mut(&preheader) {
                    let insert_pos = target_block
                        .instructions
                        .len()
                        .saturating_sub(if target_block.is_terminator() { 1 } else { 0 });

                    for (_, instr) in invoke_instructions.iter().rev() {
                        target_block.instructions.insert(insert_pos, instr.clone());
                        report.instructions_added += 1;
                        report.licm_moved += 1;
                    }
                }

                if let Some(body_block) = func.blocks.get_mut(&body_label) {
                    let indices: HashSet<usize> =
                        invoke_instructions.iter().map(|(i, _)| *i).collect();
                    body_block.instructions = body_block
                        .instructions
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| !indices.contains(i))
                        .map(|(_, instr)| instr.clone())
                        .collect();
                    report.instructions_removed += invoke_instructions.len();
                }

                report.changes_made += invoke_instructions.len() * 2;
            }
        }

        report
    }

    /// Проверяет, является ли инструкция инвариантной в цикле
    fn is_loop_invariant(
        instr: &IRInstruction,
        cond_label: &str,
        blocks: &std::collections::HashMap<String, super::basic_block::BasicBlock>,
    ) -> bool {
        match instr {
            IRInstruction::Store(_, _)
            | IRInstruction::Jump(_)
            | IRInstruction::JumpIf(_, _)
            | IRInstruction::JumpIfNot(_, _)
            | IRInstruction::Label(_)
            | IRInstruction::Call(_, _, _)
            | IRInstruction::Param(_, _)
            | IRInstruction::Return(_)
            | IRInstruction::Load(_, _) => return false,
            _ => {}
        }

        for op in instr.operands() {
            match op {
                Operand::Variable(name) => {
                    if let Some(body_label) = cond_label
                        .replace("while_cond", "while_body")
                        .replace("for_cond", "for_body")
                        .into()
                    {
                        if let Some(body_block) = blocks.get(
                            &(cond_label
                                .replace("while_cond", "while_body")
                                .replace("for_cond", "for_body")
                                + &body_label),
                        ) {
                            for body_instr in &body_block.instructions {
                                if Self::modifies_variable(body_instr, name) {
                                    return false;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        true
    }

    /// Проверяет, модифицирует ли инструкция переменную
    fn modifies_variable(instr: &IRInstruction, var_name: &str) -> bool {
        match instr {
            IRInstruction::Move(dest, _)
            | IRInstruction::Add(dest, _, _)
            | IRInstruction::Sub(dest, _, _)
            | IRInstruction::Mul(dest, _, _)
            | IRInstruction::Div(dest, _, _)
            | IRInstruction::Mod(dest, _, _)
            | IRInstruction::Neg(dest, _)
            | IRInstruction::And(dest, _, _)
            | IRInstruction::Or(dest, _, _)
            | IRInstruction::Not(dest, _)
            | IRInstruction::Xor(dest, _, _)
            | IRInstruction::Load(dest, _)
            | IRInstruction::Call(dest, _, _)
            | IRInstruction::IntToFloat(dest, _)
            | IRInstruction::FloatToInt(dest, _) => {
                if let Operand::Variable(name) = dest {
                    name == var_name
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn optimize_block(block: &mut super::basic_block::BasicBlock) -> OptimizationReport {
        let mut report = OptimizationReport::new();
        let mut new_instructions = Vec::new();

        for instr in &block.instructions {
            let (folded, applied) = Self::fold_constants(instr);
            if applied {
                new_instructions.push(folded);
                report.changes_made += 1;
                report.simplifications_applied += 1;
                continue;
            }

            let (simplified, applied) = Self::algebraic_simplify(instr);
            if applied {
                new_instructions.push(simplified);
                report.changes_made += 1;
                report.simplifications_applied += 1;
                continue;
            }

            new_instructions.push(instr.clone());
        }

        let mut used_vars = HashSet::new();
        let mut used_temps = HashSet::new();

        for instr in &new_instructions {
            for op in instr.operands() {
                match op {
                    Operand::Temporary(name) => {
                        used_temps.insert(name.clone());
                    }
                    Operand::Variable(name) => {
                        used_vars.insert(name.clone());
                    }
                    _ => {}
                }
            }
        }

        let mut after_dead = Vec::new();
        for instr in new_instructions {
            let keep = match &instr {
                IRInstruction::Store(_, _)
                | IRInstruction::Param(_, _)
                | IRInstruction::Return(_)
                | IRInstruction::Jump(_)
                | IRInstruction::JumpIf(_, _)
                | IRInstruction::JumpIfNot(_, _)
                | IRInstruction::Label(_) => true,

                IRInstruction::Move(dest, _)
                | IRInstruction::Add(dest, _, _)
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
                | IRInstruction::CmpEqF(dest, _, _)
                | IRInstruction::CmpNeF(dest, _, _)
                | IRInstruction::CmpLtF(dest, _, _)
                | IRInstruction::CmpLeF(dest, _, _)
                | IRInstruction::CmpGtF(dest, _, _)
                | IRInstruction::CmpGeF(dest, _, _)
                | IRInstruction::CmpLtU(dest, _, _)
                | IRInstruction::CmpLeU(dest, _, _)
                | IRInstruction::CmpGtU(dest, _, _)
                | IRInstruction::CmpGeU(dest, _, _)
                | IRInstruction::IntToFloat(dest, _)
                | IRInstruction::FloatToInt(dest, _)
                | IRInstruction::Load(dest, _)
                | IRInstruction::Alloca(dest, _)
                | IRInstruction::Gep(dest, _, _)
                | IRInstruction::Call(dest, _, _) => match dest {
                    Operand::Temporary(name) => used_temps.contains(name),
                    Operand::Variable(name) => used_vars.contains(name),
                    _ => true,
                },
                _ => true,
            };

            if keep {
                after_dead.push(instr);
            } else {
                report.changes_made += 1;
                report.dead_code_removed += 1;
                report.instructions_removed += 1;
            }
        }

        let mut final_instructions = Vec::new();
        for instr in after_dead {
            if let IRInstruction::Move(dest, src) = &instr {
                if dest == src {
                    report.changes_made += 1;
                    report.instructions_removed += 1;
                    continue;
                }
            }
            final_instructions.push(instr);
        }

        block.instructions = final_instructions;
        report
    }

    fn fold_constants(instr: &IRInstruction) -> (IRInstruction, bool) {
        match instr {
            IRInstruction::Add(dest, left, right) => {
                if let (Operand::IntLiteral(l), Operand::IntLiteral(r)) = (left, right) {
                    return (
                        IRInstruction::Move(dest.clone(), Operand::IntLiteral(l + r)),
                        true,
                    );
                }
                if let (Operand::FloatLiteral(l), Operand::FloatLiteral(r)) = (left, right) {
                    return (
                        IRInstruction::Move(dest.clone(), Operand::FloatLiteral(l + r)),
                        true,
                    );
                }
            }
            IRInstruction::Sub(dest, left, right) => {
                if let (Operand::IntLiteral(l), Operand::IntLiteral(r)) = (left, right) {
                    return (
                        IRInstruction::Move(dest.clone(), Operand::IntLiteral(l - r)),
                        true,
                    );
                }
                if let (Operand::FloatLiteral(l), Operand::FloatLiteral(r)) = (left, right) {
                    return (
                        IRInstruction::Move(dest.clone(), Operand::FloatLiteral(l - r)),
                        true,
                    );
                }
            }
            IRInstruction::Mul(dest, left, right) => {
                if let (Operand::IntLiteral(l), Operand::IntLiteral(r)) = (left, right) {
                    return (
                        IRInstruction::Move(dest.clone(), Operand::IntLiteral(l * r)),
                        true,
                    );
                }
                if let (Operand::FloatLiteral(l), Operand::FloatLiteral(r)) = (left, right) {
                    return (
                        IRInstruction::Move(dest.clone(), Operand::FloatLiteral(l * r)),
                        true,
                    );
                }
            }
            IRInstruction::Div(dest, left, right) => {
                if let (Operand::IntLiteral(l), Operand::IntLiteral(r)) = (left, right) {
                    if *r != 0 {
                        return (
                            IRInstruction::Move(dest.clone(), Operand::IntLiteral(l / r)),
                            true,
                        );
                    }
                }
                if let (Operand::FloatLiteral(l), Operand::FloatLiteral(r)) = (left, right) {
                    if *r != 0.0 {
                        return (
                            IRInstruction::Move(dest.clone(), Operand::FloatLiteral(l / r)),
                            true,
                        );
                    }
                }
            }
            IRInstruction::Move(dest, src) => {
                if let Operand::IntLiteral(val) = src {
                    return (
                        IRInstruction::Move(dest.clone(), Operand::IntLiteral(*val)),
                        false,
                    );
                }
                if let Operand::FloatLiteral(val) = src {
                    return (
                        IRInstruction::Move(dest.clone(), Operand::FloatLiteral(*val)),
                        false,
                    );
                }
            }
            _ => {}
        }
        (instr.clone(), false)
    }

    fn algebraic_simplify(instr: &IRInstruction) -> (IRInstruction, bool) {
        match instr {
            IRInstruction::Add(dest, left, right) => {
                if let Operand::IntLiteral(0) = right {
                    return (IRInstruction::Move(dest.clone(), left.clone()), true);
                }
                if let Operand::IntLiteral(0) = left {
                    return (IRInstruction::Move(dest.clone(), right.clone()), true);
                }
                if let Operand::FloatLiteral(v) = right {
                    if *v == 0.0 {
                        return (IRInstruction::Move(dest.clone(), left.clone()), true);
                    }
                }
                if let Operand::FloatLiteral(v) = left {
                    if *v == 0.0 {
                        return (IRInstruction::Move(dest.clone(), right.clone()), true);
                    }
                }
            }
            IRInstruction::Sub(dest, left, right) => {
                if let Operand::IntLiteral(0) = right {
                    return (IRInstruction::Move(dest.clone(), left.clone()), true);
                }
                if left == right {
                    return (
                        IRInstruction::Move(dest.clone(), Operand::IntLiteral(0)),
                        true,
                    );
                }
            }
            IRInstruction::Mul(dest, left, right) => {
                if let Operand::IntLiteral(1) = right {
                    return (IRInstruction::Move(dest.clone(), left.clone()), true);
                }
                if let Operand::IntLiteral(1) = left {
                    return (IRInstruction::Move(dest.clone(), right.clone()), true);
                }
                if let Operand::IntLiteral(0) = right {
                    return (
                        IRInstruction::Move(dest.clone(), Operand::IntLiteral(0)),
                        true,
                    );
                }
                if let Operand::IntLiteral(0) = left {
                    return (
                        IRInstruction::Move(dest.clone(), Operand::IntLiteral(0)),
                        true,
                    );
                }
                if let Operand::FloatLiteral(v) = right {
                    if *v == 1.0 {
                        return (IRInstruction::Move(dest.clone(), left.clone()), true);
                    }
                    if *v == 0.0 {
                        return (
                            IRInstruction::Move(dest.clone(), Operand::FloatLiteral(0.0)),
                            true,
                        );
                    }
                }
                if let Operand::FloatLiteral(v) = left {
                    if *v == 1.0 {
                        return (IRInstruction::Move(dest.clone(), right.clone()), true);
                    }
                    if *v == 0.0 {
                        return (
                            IRInstruction::Move(dest.clone(), Operand::FloatLiteral(0.0)),
                            true,
                        );
                    }
                }
            }
            IRInstruction::Div(dest, left, right) => {
                if let Operand::IntLiteral(1) = right {
                    return (IRInstruction::Move(dest.clone(), left.clone()), true);
                }
                if left == right {
                    return (
                        IRInstruction::Move(dest.clone(), Operand::IntLiteral(1)),
                        true,
                    );
                }
            }
            _ => {}
        }
        (instr.clone(), false)
    }
}
