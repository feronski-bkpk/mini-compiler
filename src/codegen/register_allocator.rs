//! Linear Scan регистровый аллокатор для x86-64
//!
//! Реализует алгоритм linear scan для распределения регистров
//! под переменные и временные значения вместо хранения в стеке.
//! Поддерживает spill при нехватке регистров.

use std::collections::HashMap;
use std::fmt;

/// Регистры x86-64, доступные для аллокации
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Register {
    RBX,
    R12,
    R13,
    R14,
    R15,
    R8,
    R9,
    R10,
    R11,
    RDI,
    RSI,
    RDX,
    RCX,
}

impl Register {
    pub fn name(&self) -> &'static str {
        match self {
            Register::RBX => "rbx",
            Register::R12 => "r12",
            Register::R13 => "r13",
            Register::R14 => "r14",
            Register::R15 => "r15",
            Register::R8 => "r8",
            Register::R9 => "r9",
            Register::R10 => "r10",
            Register::R11 => "r11",
            Register::RDI => "rdi",
            Register::RSI => "rsi",
            Register::RDX => "rdx",
            Register::RCX => "rcx",
        }
    }

    pub fn is_callee_saved(&self) -> bool {
        matches!(
            self,
            Register::RBX | Register::R12 | Register::R13 | Register::R14 | Register::R15
        )
    }

    pub fn all_allocatable() -> Vec<Register> {
        vec![Register::RBX, Register::R12]
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Интервал жизни переменной
#[derive(Debug, Clone)]
pub struct LiveInterval {
    pub var_name: String,
    pub start: usize,
    pub end: usize,
    pub spilled: bool,
    pub spill_slot: Option<i32>,
}

impl LiveInterval {
    pub fn new(var_name: String, start: usize, end: usize) -> Self {
        Self {
            var_name,
            start,
            end,
            spilled: false,
            spill_slot: None,
        }
    }
}

/// Результат аллокации для одной переменной
#[derive(Debug, Clone)]
pub enum Allocation {
    Register(Register),
    Stack(i32),
}

/// Основной аллокатор регистров на базе linear scan
pub struct AdvancedRegisterAllocator {
    free_registers: Vec<Register>,
    allocation: HashMap<String, Allocation>,
    active: Vec<LiveInterval>,
    intervals: Vec<LiveInterval>,
    spill_counter: i32,
    stats: RegisterStatistics,
    min_spill_offset: i32,
}

impl AdvancedRegisterAllocator {
    pub fn new() -> Self {
        Self {
            free_registers: Register::all_allocatable(),
            allocation: HashMap::new(),
            active: Vec::new(),
            intervals: Vec::new(),
            spill_counter: 0,
            stats: RegisterStatistics::default(),
            min_spill_offset: 0,
        }
    }

    pub fn set_min_spill_offset(&mut self, offset: i32) {
        self.min_spill_offset = offset;
    }

    pub fn analyze_live_ranges(&mut self, instructions: &[crate::ir::IRInstruction]) {
        self.intervals.clear();
        self.allocation.clear();
        self.active.clear();
        self.spill_counter = 0;

        let mut first_def: HashMap<String, usize> = HashMap::new();
        let mut last_use: HashMap<String, usize> = HashMap::new();

        for (idx, instr) in instructions.iter().enumerate() {
            let dests: Vec<String> = match instr {
                crate::ir::IRInstruction::Add(d, _, _)
                | crate::ir::IRInstruction::Sub(d, _, _)
                | crate::ir::IRInstruction::Mul(d, _, _)
                | crate::ir::IRInstruction::Div(d, _, _)
                | crate::ir::IRInstruction::Mod(d, _, _)
                | crate::ir::IRInstruction::And(d, _, _)
                | crate::ir::IRInstruction::Or(d, _, _)
                | crate::ir::IRInstruction::Xor(d, _, _)
                | crate::ir::IRInstruction::Not(d, _)
                | crate::ir::IRInstruction::Neg(d, _)
                | crate::ir::IRInstruction::Move(d, _)
                | crate::ir::IRInstruction::Load(d, _)
                | crate::ir::IRInstruction::Call(d, _, _)
                | crate::ir::IRInstruction::IntToFloat(d, _)
                | crate::ir::IRInstruction::FloatToInt(d, _)
                | crate::ir::IRInstruction::Alloca(d, _)
                | crate::ir::IRInstruction::ArrayLoad(d, _, _)
                | crate::ir::IRInstruction::CmpEq(d, _, _)
                | crate::ir::IRInstruction::CmpNe(d, _, _)
                | crate::ir::IRInstruction::CmpLt(d, _, _)
                | crate::ir::IRInstruction::CmpLe(d, _, _)
                | crate::ir::IRInstruction::CmpGt(d, _, _)
                | crate::ir::IRInstruction::CmpGe(d, _, _)
                | crate::ir::IRInstruction::CmpEqF(d, _, _)
                | crate::ir::IRInstruction::CmpNeF(d, _, _)
                | crate::ir::IRInstruction::CmpLtF(d, _, _)
                | crate::ir::IRInstruction::CmpLeF(d, _, _)
                | crate::ir::IRInstruction::CmpGtF(d, _, _)
                | crate::ir::IRInstruction::CmpGeF(d, _, _)
                | crate::ir::IRInstruction::CmpLtU(d, _, _)
                | crate::ir::IRInstruction::CmpLeU(d, _, _)
                | crate::ir::IRInstruction::CmpGtU(d, _, _)
                | crate::ir::IRInstruction::CmpGeU(d, _, _) => {
                    vec![Self::operand_name_to_string(d)]
                }
                _ => vec![],
            };

            for name in dests {
                if !name.is_empty() {
                    first_def.entry(name.clone()).or_insert(idx);
                    last_use.insert(name.clone(), idx);
                }
            }

            for op in instr.operands() {
                let name = Self::operand_name_to_string(op);
                if !name.is_empty() {
                    first_def.entry(name.clone()).or_insert(idx);
                    last_use.insert(name.clone(), idx);
                }
            }
        }

        for (name, start) in first_def {
            let end = last_use.get(&name).copied().unwrap_or(start);
            self.intervals.push(LiveInterval::new(name, start, end));
        }

        self.intervals.sort_by_key(|i| i.start);
    }

    fn operand_name_to_string(op: &crate::ir::Operand) -> String {
        match op {
            crate::ir::Operand::Temporary(name) => name.clone(),
            crate::ir::Operand::Variable(name) => name.clone(),
            _ => String::new(),
        }
    }

    pub fn linear_scan_allocate(&mut self) -> bool {
        self.stats = RegisterStatistics::default();
        self.stats.total_intervals = self.intervals.len();

        eprintln!(
            "Intervals: {:?}",
            self.intervals
                .iter()
                .map(|i| format!("{}:[{}-{}]", i.var_name, i.start, i.end))
                .collect::<Vec<_>>()
        );
        eprintln!("Free regs: {:?}", self.free_registers);

        for interval in self.intervals.clone() {
            self.expire_old_intervals(interval.start);

            if self.free_registers.is_empty() {
                self.spill_at_interval(&interval);
                if self.free_registers.is_empty() {
                    self.spill_current(&interval);
                }
            } else {
                let reg = self.free_registers.pop().unwrap();
                self.allocation
                    .insert(interval.var_name.clone(), Allocation::Register(reg));
                self.active.push(interval);
                self.stats.allocated_registers += 1;
            }
        }

        self.stats.allocation_success = self.stats.spilled_intervals == 0;
        self.stats.allocation_success
    }

    fn spill_current(&mut self, interval: &LiveInterval) {
        self.spill_counter += 1;
        let slot = -(self.min_spill_offset + self.spill_counter * 8);
        let mut spilled = interval.clone();
        spilled.spilled = true;
        spilled.spill_slot = Some(slot);
        self.allocation
            .insert(spilled.var_name.clone(), Allocation::Stack(slot));
        self.active.push(spilled);
        self.stats.spilled_intervals += 1;
    }

    fn expire_old_intervals(&mut self, position: usize) {
        self.active.retain(|i| {
            if i.end < position {
                if let Some(Allocation::Register(reg)) = self.allocation.remove(&i.var_name) {
                    self.free_registers.push(reg);
                }
                false
            } else {
                true
            }
        });
    }

    fn spill_at_interval(&mut self, interval: &LiveInterval) {
        if let Some(spill_idx) = self
            .active
            .iter()
            .enumerate()
            .max_by_key(|(_, i)| i.end)
            .map(|(idx, _)| idx)
        {
            let spill_interval = &self.active[spill_idx];

            if spill_interval.end > interval.end {
                if let Some(Allocation::Register(reg)) =
                    self.allocation.remove(&spill_interval.var_name)
                {
                    let mut spilled = spill_interval.clone();
                    spilled.spilled = true;
                    self.spill_counter += 1;
                    spilled.spill_slot = Some(-(self.min_spill_offset + self.spill_counter * 8));
                    self.allocation.insert(
                        spilled.var_name.clone(),
                        Allocation::Stack(spilled.spill_slot.unwrap()),
                    );
                    self.allocation
                        .insert(interval.var_name.clone(), Allocation::Register(reg));
                    self.active.remove(spill_idx);
                    self.active.push(interval.clone());
                    self.stats.spilled_intervals += 1;
                    self.stats.allocated_registers += 1;
                    return;
                }
            }
        }

        self.spill_counter += 1;
        let slot = -(self.min_spill_offset + self.spill_counter * 8);
        let mut spilled = interval.clone();
        spilled.spilled = true;
        spilled.spill_slot = Some(slot);
        self.allocation
            .insert(spilled.var_name.clone(), Allocation::Stack(slot));
        self.active.push(spilled);
        self.stats.spilled_intervals += 1;
    }

    pub fn get_allocation(&self, name: &str) -> Option<&Allocation> {
        self.allocation.get(name)
    }

    pub fn reset(&mut self) {
        self.free_registers = Register::all_allocatable();
        self.allocation.clear();
        self.active.clear();
        self.intervals.clear();
        self.spill_counter = 0;
        self.stats = RegisterStatistics::default();
    }

    pub fn statistics(&self) -> &RegisterStatistics {
        &self.stats
    }

    pub fn used_callee_saved(&self) -> Vec<Register> {
        let mut result = Vec::new();
        for alloc in self.allocation.values() {
            if let Allocation::Register(reg) = alloc {
                if reg.is_callee_saved() && !result.contains(reg) {
                    result.push(*reg);
                }
            }
        }
        result
    }
}

#[derive(Debug, Clone, Default)]
pub struct RegisterStatistics {
    pub total_intervals: usize,
    pub allocated_registers: usize,
    pub spilled_intervals: usize,
    pub allocation_success: bool,
}

impl fmt::Display for RegisterStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== Linear Scan Register Allocation ===")?;
        writeln!(f, "Total intervals: {}", self.total_intervals)?;
        writeln!(f, "Allocated to registers: {}", self.allocated_registers)?;
        writeln!(f, "Spilled to stack: {}", self.spilled_intervals)?;
        writeln!(
            f,
            "Success: {}",
            if self.allocation_success {
                "YES (no spills)"
            } else {
                "NO (spills occurred)"
            }
        )?;
        Ok(())
    }
}

impl Default for AdvancedRegisterAllocator {
    fn default() -> Self {
        Self::new()
    }
}
