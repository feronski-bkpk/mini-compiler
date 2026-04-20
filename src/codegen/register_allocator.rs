use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

/// Регистры x86-64
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Register {
    RAX,
    RBX,
    RCX,
    RDX,
    RSI,
    RDI,
    RBP,
    RSP,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    EAX,
    EBX,
    ECX,
    EDX,
    ESI,
    EDI,
    EBP,
    ESP,
    AX,
    BX,
    CX,
    DX,
    SI,
    DI,
    BP,
    SP,
    AL,
    BL,
    CL,
    DL,
    SIL,
    DIL,
    BPL,
    SPL,
}

impl Register {
    pub fn name(&self) -> &'static str {
        match self {
            Register::RAX => "rax",
            Register::RBX => "rbx",
            Register::RCX => "rcx",
            Register::RDX => "rdx",
            Register::RSI => "rsi",
            Register::RDI => "rdi",
            Register::RBP => "rbp",
            Register::RSP => "rsp",
            Register::R8 => "r8",
            Register::R9 => "r9",
            Register::R10 => "r10",
            Register::R11 => "r11",
            Register::R12 => "r12",
            Register::R13 => "r13",
            Register::R14 => "r14",
            Register::R15 => "r15",
            Register::EAX => "eax",
            Register::EBX => "ebx",
            Register::ECX => "ecx",
            Register::EDX => "edx",
            Register::ESI => "esi",
            Register::EDI => "edi",
            Register::EBP => "ebp",
            Register::ESP => "esp",
            Register::AX => "ax",
            Register::BX => "bx",
            Register::CX => "cx",
            Register::DX => "dx",
            Register::SI => "si",
            Register::DI => "di",
            Register::BP => "bp",
            Register::SP => "sp",
            Register::AL => "al",
            Register::BL => "bl",
            Register::CL => "cl",
            Register::DL => "dl",
            Register::SIL => "sil",
            Register::DIL => "dil",
            Register::BPL => "bpl",
            Register::SPL => "spl",
        }
    }

    pub fn is_caller_saved(&self) -> bool {
        matches!(
            self,
            Register::RAX
                | Register::RCX
                | Register::RDX
                | Register::RSI
                | Register::RDI
                | Register::R8
                | Register::R9
                | Register::R10
                | Register::R11
        )
    }

    pub fn is_callee_saved(&self) -> bool {
        matches!(
            self,
            Register::RBX
                | Register::RBP
                | Register::RSP
                | Register::R12
                | Register::R13
                | Register::R14
                | Register::R15
        )
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Информация о временной переменной
#[derive(Debug, Clone)]
pub struct LiveRange {
    pub start: usize,
    pub end: usize,
    pub temp_name: String,
    pub size: usize,
}

/// Граф конфликтов для распределения регистров
#[derive(Debug)]
pub struct ConflictGraph {
    pub nodes: HashMap<String, HashSet<String>>,
    pub live_ranges: Vec<LiveRange>,
}

impl ConflictGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            live_ranges: Vec::new(),
        }
    }

    /// Добавляет конфликт между двумя временными переменными
    pub fn add_conflict(&mut self, t1: &str, t2: &str) {
        self.nodes
            .entry(t1.to_string())
            .or_insert_with(HashSet::new)
            .insert(t2.to_string());
        self.nodes
            .entry(t2.to_string())
            .or_insert_with(HashSet::new)
            .insert(t1.to_string());
    }

    /// Проверяет, конфликтуют ли две переменные
    pub fn has_conflict(&self, t1: &str, t2: &str) -> bool {
        self.nodes
            .get(t1)
            .map_or(false, |neighbors| neighbors.contains(t2))
    }

    /// Возвращает степень узла (количество конфликтов)
    pub fn degree(&self, node: &str) -> usize {
        self.nodes.get(node).map_or(0, |neighbors| neighbors.len())
    }
}

/// Расширенный аллокатор регистров с графом конфликтов
pub struct AdvancedRegisterAllocator {
    available_registers: Vec<Register>,
    allocated: HashMap<String, Register>,
    reverse_allocated: HashMap<Register, String>,
    used_registers: Vec<String>,
    next_index: usize,
    conflict_graph: ConflictGraph,
    live_ranges: Vec<LiveRange>,
    spill_count: usize,
    allocation_attempts: usize,
}

impl AdvancedRegisterAllocator {
    pub fn new() -> Self {
        let available = vec![
            Register::RAX,
            Register::RCX,
            Register::RDX,
            Register::RSI,
            Register::RDI,
            Register::R8,
            Register::R9,
            Register::R10,
            Register::R11,
        ];

        Self {
            available_registers: available,
            allocated: HashMap::new(),
            reverse_allocated: HashMap::new(),
            used_registers: Vec::new(),
            next_index: 0,
            conflict_graph: ConflictGraph::new(),
            live_ranges: Vec::new(),
            spill_count: 0,
            allocation_attempts: 0,
        }
    }

    /// Анализирует диапазоны жизни временных переменных
    pub fn analyze_live_ranges(&mut self, instructions: &[crate::ir::IRInstruction]) {
        self.live_ranges.clear();

        let mut first_use: HashMap<String, usize> = HashMap::new();
        let mut last_use: HashMap<String, usize> = HashMap::new();

        for (idx, instr) in instructions.iter().enumerate() {
            for op in instr.operands() {
                if let crate::ir::Operand::Temporary(name) = op {
                    first_use.entry(name.clone()).or_insert(idx);
                    last_use.insert(name.clone(), idx);
                }
            }

            match instr {
                crate::ir::IRInstruction::Add(dest, _, _)
                | crate::ir::IRInstruction::Sub(dest, _, _)
                | crate::ir::IRInstruction::Mul(dest, _, _)
                | crate::ir::IRInstruction::Div(dest, _, _)
                | crate::ir::IRInstruction::Mod(dest, _, _)
                | crate::ir::IRInstruction::Move(dest, _)
                | crate::ir::IRInstruction::Load(dest, _)
                | crate::ir::IRInstruction::Call(dest, _, _) => {
                    if let crate::ir::Operand::Temporary(name) = dest {
                        first_use.entry(name.clone()).or_insert(idx);
                        last_use.insert(name.clone(), idx);
                    }
                }
                _ => {}
            }
        }

        for (name, start) in first_use {
            if let Some(end) = last_use.get(&name) {
                self.live_ranges.push(LiveRange {
                    start,
                    end: *end,
                    temp_name: name,
                    size: 8,
                });
            }
        }

        self.build_conflict_graph();
    }

    /// Строит граф конфликтов на основе диапазонов жизни
    fn build_conflict_graph(&mut self) {
        self.conflict_graph = ConflictGraph::new();

        for i in 0..self.live_ranges.len() {
            for j in i + 1..self.live_ranges.len() {
                let range1 = &self.live_ranges[i];
                let range2 = &self.live_ranges[j];

                if range1.start <= range2.end && range2.start <= range1.end {
                    self.conflict_graph
                        .add_conflict(&range1.temp_name, &range2.temp_name);
                }
            }
        }
    }

    /// Алгоритм раскраски графа (упрощенный)
    pub fn graph_coloring_allocate(&mut self) -> bool {
        self.allocation_attempts += 1;

        let mut nodes: Vec<String> = self.conflict_graph.nodes.keys().cloned().collect();
        nodes.sort_by(|a, b| {
            self.conflict_graph
                .degree(b)
                .cmp(&self.conflict_graph.degree(a))
        });

        let mut colors: HashMap<String, Option<Register>> = HashMap::new();

        for node in &nodes {
            let mut neighbor_colors = HashSet::new();
            if let Some(neighbors) = self.conflict_graph.nodes.get(node) {
                for neighbor in neighbors {
                    if let Some(Some(reg)) = colors.get(neighbor) {
                        neighbor_colors.insert(*reg);
                    }
                }
            }

            let mut assigned = None;
            for reg in &self.available_registers {
                if !neighbor_colors.contains(reg) {
                    assigned = Some(*reg);
                    break;
                }
            }

            if let Some(reg) = assigned {
                colors.insert(node.clone(), Some(reg));
            } else {
                self.spill_count += 1;
                colors.insert(node.clone(), None);
            }
        }

        for (node, color) in colors {
            if let Some(reg) = color {
                self.allocated.insert(node.clone(), reg);
                self.reverse_allocated.insert(reg, node);

                let reg_name = reg.name().to_string();
                if !self.used_registers.contains(&reg_name) {
                    self.used_registers.push(reg_name);
                }
            }
        }

        self.spill_count == 0
    }

    /// Линейное сканирование (более простой алгоритм)
    pub fn linear_scan_allocate(&mut self) -> bool {
        self.allocation_attempts += 1;

        let mut sorted_ranges = self.live_ranges.clone();
        sorted_ranges.sort_by_key(|r| r.start);

        let mut active: VecDeque<LiveRange> = VecDeque::new();
        let mut free_registers: Vec<Register> = self.available_registers.clone();

        for range in &sorted_ranges {
            while let Some(active_range) = active.front() {
                if active_range.end < range.start {
                    if let Some(active_range) = active.pop_front() {
                        if let Some(reg) = self.allocated.get(&active_range.temp_name) {
                            free_registers.push(*reg);
                        }
                    }
                } else {
                    break;
                }
            }

            if let Some(reg) = free_registers.pop() {
                self.allocated.insert(range.temp_name.clone(), reg);
                self.reverse_allocated.insert(reg, range.temp_name.clone());

                let reg_name = reg.name().to_string();
                if !self.used_registers.contains(&reg_name) {
                    self.used_registers.push(reg_name);
                }

                active.push_back(range.clone());
            } else {
                self.spill_count += 1;

                if let Some(to_spill) = active.iter().max_by_key(|r| r.end) {
                    let to_spill_name = to_spill.temp_name.clone();
                    if let Some(reg) = self.allocated.remove(&to_spill_name) {
                        free_registers.push(reg);
                        self.reverse_allocated.remove(&reg);
                    }
                    return self.linear_scan_allocate();
                }
            }
        }

        self.spill_count == 0
    }

    /// Возвращает статистику использования регистров
    pub fn statistics(&self) -> RegisterStatistics {
        RegisterStatistics {
            total_temporaries: self.live_ranges.len(),
            allocated_registers: self.allocated.len(),
            spilled_temporaries: self.spill_count,
            allocation_attempts: self.allocation_attempts,
            used_registers: self.used_registers.clone(),
            conflict_graph_density: if self.conflict_graph.nodes.is_empty() {
                0.0
            } else {
                let total_edges: usize = self.conflict_graph.nodes.values().map(|n| n.len()).sum();
                total_edges as f64 / self.conflict_graph.nodes.len() as f64
            },
        }
    }

    pub fn reset(&mut self) {
        self.allocated.clear();
        self.reverse_allocated.clear();
        self.next_index = 0;
        self.spill_count = 0;
        self.conflict_graph = ConflictGraph::new();
        self.live_ranges.clear();
    }

    pub fn get_register_for_temp(&self, name: &str) -> Option<Register> {
        self.allocated.get(name).copied()
    }

    pub fn get_used_registers(&self) -> Vec<String> {
        self.used_registers.clone()
    }
}

/// Статистика распределения регистров
#[derive(Debug)]
pub struct RegisterStatistics {
    pub total_temporaries: usize,
    pub allocated_registers: usize,
    pub spilled_temporaries: usize,
    pub allocation_attempts: usize,
    pub used_registers: Vec<String>,
    pub conflict_graph_density: f64,
}

impl fmt::Display for RegisterStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== Register Allocation Statistics ===")?;
        writeln!(f, "Total temporaries: {}", self.total_temporaries)?;
        writeln!(f, "Allocated registers: {}", self.allocated_registers)?;
        writeln!(f, "Spilled temporaries: {}", self.spilled_temporaries)?;
        writeln!(f, "Allocation attempts: {}", self.allocation_attempts)?;
        writeln!(f, "Used registers: {:?}", self.used_registers)?;
        writeln!(
            f,
            "Conflict graph density: {:.2}",
            self.conflict_graph_density
        )?;
        writeln!(f, "Allocation success: {}", self.spilled_temporaries == 0)?;
        Ok(())
    }
}
