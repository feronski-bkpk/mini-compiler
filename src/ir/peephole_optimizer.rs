//! Peephole оптимизатор для IR

use super::basic_block::ProgramIR;
use super::ir_instructions::{IRInstruction, Operand};

/// Отчет об оптимизациях
#[derive(Debug, Default)]
pub struct OptimizationReport {
    pub changes_made: usize,
    pub instructions_removed: usize,
    pub instructions_added: usize,
    pub simplifications_applied: usize,
    pub dead_code_removed: usize,
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

            for func in program.functions.values_mut() {
                let report = Self::optimize_function(func);
                if report.changes_made > 0 {
                    changed = true;
                    total_report.add(&report);
                }
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

        let mut used_vars = std::collections::HashSet::new();
        let mut used_temps = std::collections::HashSet::new();

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
            }
            IRInstruction::Sub(dest, left, right) => {
                if let (Operand::IntLiteral(l), Operand::IntLiteral(r)) = (left, right) {
                    return (
                        IRInstruction::Move(dest.clone(), Operand::IntLiteral(l - r)),
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
            }
            IRInstruction::Move(dest, src) => {
                if let Operand::IntLiteral(val) = src {
                    return (
                        IRInstruction::Move(dest.clone(), Operand::IntLiteral(*val)),
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
