//! Генератор x86-64 ассемблерного кода из IR
//!
//! Использует linear scan регистровый аллокатор для хранения
//! переменных в регистрах вместо стека, генерирует прямые условные
//! переходы (je/jg/jl/jb/ja) и поддерживает глобальные переменные.

use crate::ir::{FunctionIR, IRInstruction, Operand, ProgramIR};
use super::register_allocator::{AdvancedRegisterAllocator, Allocation};
use std::collections::HashMap;

pub struct X86Generator {
    string_counter: usize,
    string_literals: Vec<(String, String)>,
    global_vars: Vec<(String, String)>,
    stack_size: usize,
    current_function: Option<String>,
    allocator: AdvancedRegisterAllocator,
    spill_offsets: HashMap<String, i32>,
    param_offsets: HashMap<String, i32>,
    spill_total: i32,
    used_callee_saved: Vec<super::register_allocator::Register>,
}

impl X86Generator {
    pub fn new() -> Self {
        Self {
            string_counter: 0,
            string_literals: Vec::new(),
            global_vars: Vec::new(),
            stack_size: 0,
            current_function: None,
            allocator: AdvancedRegisterAllocator::new(),
            spill_offsets: HashMap::new(),
            param_offsets: HashMap::new(),
            spill_total: 0,
            used_callee_saved: Vec::new(),
        }
    }

    pub fn generate(&mut self, program: &ProgramIR) -> super::CodegenResult {
        let mut output = String::new();
        let mut data_section = String::new();
        self.string_counter = 0;
        self.string_literals.clear();
        self.global_vars.clear();

        for (name, _typ) in &program.globals {
            self.global_vars.push((name.clone(), _typ.clone()));
        }

        let mut has_data = !self.global_vars.is_empty();

        let mut text_output = String::new();
        text_output.push_str("section .text\n");
        text_output.push_str("default rel\n");

        for func in &program.functions {
            text_output.push_str(&format!("global {}\n", func.name));
        }
        for (name, _) in &self.global_vars {
            text_output.push_str(&format!("global {}\n", name));
        }
        text_output.push_str("global main\n");
        text_output.push_str("global _start\n\n");

        for func in &program.functions {
            text_output.push_str(&self.generate_function(func));
            text_output.push_str("\n");
        }

        text_output.push_str("_start:\n    call main\n    mov rdi, rax\n    call exit\n\n");
        text_output.push_str("exit:\n    mov rax, 60\n    syscall\n");

        if has_data {
            data_section.push_str("section .data\n");
            for (name, _typ) in &self.global_vars {
                data_section.push_str(&format!("    {}: dq 0\n", name));
            }
        }
        if !self.string_literals.is_empty() {
            if !has_data {
                data_section.push_str("section .data\n");
                has_data = true;
            }
            for (label, string) in &self.string_literals {
                data_section.push_str(&format!("{}: {}\n", label, string));
            }
        }

        output.push_str(&data_section);
        if has_data {
            output.push_str("\n");
        }
        output.push_str(&text_output);

        let instruction_count = text_output
            .lines()
            .filter(|l| {
                let t = l.trim();
                !t.is_empty()
                    && !t.starts_with(';')
                    && !t.starts_with("section")
                    && !t.starts_with("global")
                    && !t.starts_with("default")
                    && !t.ends_with(':')
            })
            .count();

        let mut registers_used = vec!["rax", "rcx", "rsi", "rdi", "r8", "r9", "r10", "r11"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();
        for reg in &self.used_callee_saved {
            registers_used.push(reg.name().to_string());
        }

        super::CodegenResult {
            assembly: output,
            registers_used,
            frame_size: self.stack_size,
            instruction_count,
        }
    }

    fn generate_function(&mut self, func: &FunctionIR) -> String {
        let mut output = String::new();
        self.current_function = Some(func.name.clone());
        self.spill_offsets.clear();
        self.param_offsets.clear();
        self.spill_total = 0;
        self.used_callee_saved.clear();

        let mut all_instructions: Vec<IRInstruction> = Vec::new();
        let mut block_order: Vec<String> = Vec::new();
        let mut blocks: Vec<&String> = func.blocks.keys().collect();
        blocks.sort();
        for label in &blocks {
            if let Some(block) = func.blocks.get(*label) {
                block_order.push(label.to_string());
                for instr in &block.instructions {
                    all_instructions.push(instr.clone());
                }
            }
        }

        self.allocator.reset();
        self.allocator.analyze_live_ranges(&all_instructions);
        self.allocator.linear_scan_allocate();
        self.used_callee_saved = self.allocator.used_callee_saved();
        self.collect_spill_slots(&all_instructions);

        output.push_str(&format!("{}:\n", func.name));
        output.push_str("    push rbp\n    mov rbp, rsp\n");

        for reg in &self.used_callee_saved {
            output.push_str(&format!("    push {}\n", reg.name()));
        }

        let param_regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
        let mut param_offset = 16;
        for (i, (param_name, _)) in func.parameters.iter().enumerate() {
            self.param_offsets.insert(param_name.clone(), param_offset);
            if i < param_regs.len() {
                if let Some(alloc) = self.allocator.get_allocation(param_name) {
                    if let Allocation::Register(reg) = alloc {
                        output.push_str(&format!("    mov {}, {}\n", reg.name(), param_regs[i]));
                    }
                } else {
                    output.push_str(&format!("    mov [rbp+{}], {}\n", param_offset, param_regs[i]));
                }
            }
            param_offset += 8;
        }

        let spill_size = self.spill_total as usize;
        let aligned_spill = ((spill_size + 15) & !15) as i32;
        if aligned_spill > 0 {
            output.push_str(&format!("    sub rsp, {}\n", aligned_spill));
            self.stack_size = aligned_spill as usize;
        }

        for block_label in &block_order {
            if let Some(block) = func.blocks.get(block_label) {
                output.push_str(&format!(".{}:\n", block_label));
                for instr in &block.instructions {
                    let asm = self.generate_instruction(instr);
                    if !asm.is_empty() {
                        output.push_str(&asm);
                    }
                }
            }
        }

        self.current_function = None;
        output
    }

    fn collect_spill_slots(&mut self, instructions: &[IRInstruction]) {
        let mut spilled: HashMap<String, i32> = HashMap::new();
        for instr in instructions {
            for op in instr.all_operands() {
                let name = self.get_operand_name(op);
                if name.is_empty() {
                    continue;
                }
                if let Some(Allocation::Stack(offset)) = self.allocator.get_allocation(name) {
                    spilled.insert(name.to_string(), *offset);
                }
            }
        }
        let max_offset = spilled.values().map(|o| o.abs()).max().unwrap_or(0);
        self.spill_offsets = spilled;
        self.spill_total = max_offset;
    }

    fn get_operand_name<'a>(&self, op: &'a Operand) -> &'a str {
        match op {
            Operand::Temporary(name) => name.as_str(),
            Operand::Variable(name) => {
                if self.global_vars.iter().any(|(n, _)| n == name) {
                    ""
                } else {
                    name.as_str()
                }
            }
            _ => "",
        }
    }

    fn generate_instruction(&mut self, instr: &IRInstruction) -> String {
        match instr {
            IRInstruction::Move(d, s) => self.gen_move(d, s),
            IRInstruction::Return(Some(v)) => {
                let vs = self.op(v);
                let vq = if vs.starts_with('[') { format!("qword {}", vs) } else { vs.clone() };
                let mut o = format!("    mov rax, {}\n", vq);
                for reg in self.used_callee_saved.iter().rev() {
                    o.push_str(&format!("    pop {}\n", reg.name()));
                }
                o.push_str("    mov rsp, rbp\n    pop rbp\n    ret\n");
                o
            }
            IRInstruction::Return(None) => {
                let mut o = String::new();
                for reg in self.used_callee_saved.iter().rev() {
                    o.push_str(&format!("    pop {}\n", reg.name()));
                }
                o.push_str("    mov rsp, rbp\n    pop rbp\n    ret\n");
                o
            }
            IRInstruction::Add(d, l, r) => self.gen_binop("add", d, l, r),
            IRInstruction::Sub(d, l, r) => self.gen_binop("sub", d, l, r),
            IRInstruction::Mul(d, l, r) => self.gen_binop("imul", d, l, r),
            IRInstruction::Div(d, l, r) => self.gen_div(d, l, r, false),
            IRInstruction::Mod(d, l, r) => self.gen_div(d, l, r, true),
            IRInstruction::And(d, l, r) => self.gen_binop("and", d, l, r),
            IRInstruction::Or(d, l, r) => self.gen_binop("or", d, l, r),
            IRInstruction::Xor(d, l, r) => self.gen_binop("xor", d, l, r),
            IRInstruction::Not(d, o) => {
                let os = self.op(o);
                let oq = if os.starts_with('[') { format!("qword {}", os) } else { os.clone() };
                format!("    mov rax, {}\n    not rax\n    mov {}, rax\n", oq, self.op(d))
            },
            IRInstruction::Neg(d, o) => {
                let os = self.op(o);
                let oq = if os.starts_with('[') { format!("qword {}", os) } else { os.clone() };
                format!("    mov rax, {}\n    neg rax\n    mov {}, rax\n", oq, self.op(d))
            },
            IRInstruction::Jump(l) => format!("    jmp .{}\n", self.lbl(l)),
            IRInstruction::JumpIf(c, l) => self.gen_jump_if(c, l, false),
            IRInstruction::JumpIfNot(c, l) => self.gen_jump_if(c, l, true),
            IRInstruction::CmpJmp(l, r, tl, fl, _cmp_str, jcc_str, is_float) => {
                let ls = self.op(l);
                let rs = self.op(r);
                let tls = self.lbl(tl);
                let fls = self.lbl(fl);
                let lq = if ls.starts_with('[') { format!("qword {}", ls) } else { ls.clone() };
                let rq = if rs.starts_with('[') { format!("qword {}", rs) } else { rs.clone() };
                if *is_float {
                    format!(
                        "    movsd xmm0, {}\n    ucomisd xmm0, {}\n    {} .{}\n    jmp .{}\n",
                        lq, rq, jcc_str, tls, fls
                    )
                } else {
                    format!(
                        "    cmp {}, {}\n    {} .{}\n    jmp .{}\n",
                        lq, rq, jcc_str, tls, fls
                    )
                }
            }
            IRInstruction::CmpEq(d, l, r) => self.gen_cmp_value("sete", d, l, r, false),
            IRInstruction::CmpNe(d, l, r) => self.gen_cmp_value("setne", d, l, r, false),
            IRInstruction::CmpLt(d, l, r) => self.gen_cmp_value("setl", d, l, r, false),
            IRInstruction::CmpLe(d, l, r) => self.gen_cmp_value("setle", d, l, r, false),
            IRInstruction::CmpGt(d, l, r) => self.gen_cmp_value("setg", d, l, r, false),
            IRInstruction::CmpGe(d, l, r) => self.gen_cmp_value("setge", d, l, r, false),
            IRInstruction::CmpEqF(d, l, r) => self.gen_cmp_value("sete", d, l, r, true),
            IRInstruction::CmpNeF(d, l, r) => self.gen_cmp_value("setne", d, l, r, true),
            IRInstruction::CmpLtF(d, l, r) => self.gen_cmp_value("setb", d, l, r, true),
            IRInstruction::CmpLeF(d, l, r) => self.gen_cmp_value("setbe", d, l, r, true),
            IRInstruction::CmpGtF(d, l, r) => self.gen_cmp_value("seta", d, l, r, true),
            IRInstruction::CmpGeF(d, l, r) => self.gen_cmp_value("setae", d, l, r, true),
            IRInstruction::CmpLtU(d, l, r) => self.gen_cmp_value("setb", d, l, r, false),
            IRInstruction::CmpLeU(d, l, r) => self.gen_cmp_value("setbe", d, l, r, false),
            IRInstruction::CmpGtU(d, l, r) => self.gen_cmp_value("seta", d, l, r, false),
            IRInstruction::CmpGeU(d, l, r) => self.gen_cmp_value("setae", d, l, r, false),
            IRInstruction::IntToFloat(d, s) => {
                let ss = self.op(s);
                let sq = if ss.starts_with('[') { format!("qword {}", ss) } else { ss.clone() };
                format!("    cvtsi2sd xmm0, {}\n    movq {}, xmm0\n", sq, self.op(d))
            }
            IRInstruction::FloatToInt(d, s) => {
                let ss = self.op(s);
                let sq = if ss.starts_with('[') { format!("qword {}", ss) } else { ss.clone() };
                format!("    cvttsd2si rax, {}\n    mov {}, rax\n", sq, self.op(d))
            }
            IRInstruction::ArrayLoad(dest, base, index) => {
                let ds = self.op(dest);
                let bs = self.op(base);
                let istr = self.op(index);
                format!(
                    "    mov rax, {}\n    imul rax, 8\n    add rax, {}\n    mov rax, [rax]\n    mov {}, rax\n",
                    istr, bs, ds
                )
            }
            IRInstruction::ArrayStore(base, index, value) => {
                let bs = self.op(base);
                let istr = self.op(index);
                let vs = self.op(value);
                format!(
                    "    mov rax, {}\n    imul rax, 8\n    add rax, {}\n    mov qword [rax], {}\n",
                    istr, bs, vs
                )
            }
            IRInstruction::Label(l) => format!(".{}:\n", self.lbl(l)),
            IRInstruction::Call(d, f, a) => self.gen_call(d, f, a),
            IRInstruction::Load(d, a) => {
                let as_ = self.op(a);
                let aq = if as_.starts_with('[') { format!("qword {}", as_) } else { as_.clone() };
                format!("    mov rax, {}\n    mov {}, rax\n", aq, self.op(d))
            },
            IRInstruction::Store(a, s) => format!("    mov qword {}, {}\n", self.op(a), self.op(s)),
            IRInstruction::Alloca(d, sz) => format!("    sub rsp, {}\n    mov {}, rsp\n", sz, self.op(d)),
            _ => String::new(),
        }
    }

    fn gen_jump_if(&mut self, cond: &Operand, label: &Operand, negate: bool) -> String {
        if !negate {
            if matches!(cond, Operand::BoolLiteral(true)) || matches!(cond, Operand::IntLiteral(v) if *v != 0) {
                return format!("    jmp .{}\n", self.lbl(label));
            }
            if matches!(cond, Operand::BoolLiteral(false)) || matches!(cond, Operand::IntLiteral(0)) {
                return String::new();
            }
        } else {
            if matches!(cond, Operand::BoolLiteral(false)) || matches!(cond, Operand::IntLiteral(0)) {
                return format!("    jmp .{}\n", self.lbl(label));
            }
            if matches!(cond, Operand::BoolLiteral(true)) || matches!(cond, Operand::IntLiteral(v) if *v != 0) {
                return String::new();
            }
        }
        let cs = self.op(cond);
        let ls = self.lbl(label);
        let jcc = if negate { "je" } else { "jne" };
        if cs.starts_with('[') {
            format!("    cmp qword {}, 0\n    {} .{}\n", cs, jcc, ls)
        } else {
            format!("    cmp {}, 0\n    {} .{}\n", cs, jcc, ls)
        }
    }

    fn gen_cmp_value(&mut self, set: &str, d: &Operand, l: &Operand, r: &Operand, flt: bool) -> String {
        let ls = self.op(l);
        let rs = self.op(r);
        let ds = self.op(d);
        let mut o = String::new();
        if flt {
            o.push_str(&format!("    movsd xmm0, {}\n    ucomisd xmm0, {}\n", ls, rs));
        } else {
            let lq = if ls.starts_with('[') { format!("qword {}", ls) } else { ls.clone() };
            let rq = if rs.starts_with('[') { format!("qword {}", rs) } else { rs.clone() };
            o.push_str(&format!("    cmp {}, {}\n", lq, rq));
        }
        o.push_str(&format!("    {} al\n    movzx rax, al\n    mov {}, rax\n", set, ds));
        o
    }

    fn gen_move(&mut self, d: &Operand, s: &Operand) -> String {
        let ds = self.op(d);
        let ss = self.op(s);
        if let Operand::FloatLiteral(v) = s {
            let lb = format!("L_flt{}", self.string_counter);
            self.string_counter += 1;
            self.string_literals.push((lb.clone(), format!("dq {}", v)));
            return format!("    movsd xmm0, qword [{}]\n    movq {}, xmm0\n", lb, ds);
        }
        if !ds.starts_with('[') && !ss.starts_with('[') {
            return format!("    mov {}, {}\n", ds, ss);
        }
        if ds.starts_with('[') && ss.starts_with('[') {
            return format!("    mov rax, qword {}\n    mov qword {}, rax\n", ss, ds);
        }
        if ds.starts_with('[') {
            return format!("    mov qword {}, {}\n", ds, ss);
        }
        format!("    mov {}, qword {}\n", ds, ss)
    }

    fn gen_binop(&mut self, op: &str, d: &Operand, l: &Operand, r: &Operand) -> String {
        let ls = self.op(l);
        let rs = self.op(r);
        let ds = self.op(d);
        let is_float = matches!(l, Operand::FloatLiteral(_))
            || matches!(r, Operand::FloatLiteral(_))
            || ls.contains("xmm") || rs.contains("xmm");
        if is_float {
            let op_f = match op { "add" => "addsd", "sub" => "subsd", "imul" => "mulsd", _ => op };
            return format!("    movsd xmm0, {}\n    {} xmm0, {}\n    movq {}, xmm0\n", ls, op_f, rs, ds);
        }
        let lq = if ls.starts_with('[') { format!("qword {}", ls) } else { ls.clone() };
        let rq = if rs.starts_with('[') { format!("qword {}", rs) } else { rs.clone() };
        if !ds.starts_with('[') && ls == ds {
            return format!("    {} {}, {}\n", op, ds, rq);
        }
        if !ds.starts_with('[') && rs == ds && (op == "add" || op == "imul" || op == "and" || op == "or" || op == "xor") {
            return format!("    {} {}, {}\n", op, ds, lq);
        }
        let dq = if ds.starts_with('[') { format!("qword {}", ds) } else { ds.clone() };
        format!("    mov rax, {}\n    {} rax, {}\n    mov {}, rax\n", lq, op, rq, dq)
    }

    fn gen_div(&mut self, d: &Operand, l: &Operand, r: &Operand, m: bool) -> String {
        let ls = self.op(l);
        let rs = self.op(r);
        let ds = self.op(d);
        let rr = if m { "rdx" } else { "rax" };
        let lq = if ls.starts_with('[') { format!("qword {}", ls) } else { ls.clone() };
        let rq = if rs.starts_with('[') { format!("qword {}", rs) } else { rs.clone() };
        format!(
            "    push r12\n    mov rax, {}\n    cdq\n    mov r12, {}\n    idiv r12\n    mov {}, {}\n    pop r12\n",
            lq, rq, ds, rr
        )
    }

    fn gen_call(&mut self, d: &Operand, f: &Operand, a: &[Operand]) -> String {
        let mut o = String::new();
        o.push_str("    push rcx\n    push rdx\n    push rsi\n    push rdi\n");
        o.push_str("    push r8\n    push r9\n    push r10\n    push r11\n");
        let regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
        for (i, arg) in a.iter().enumerate() {
            if i < 6 {
                o.push_str(&format!("    mov {}, {}\n", regs[i], self.op(arg)));
            } else {
                o.push_str(&format!("    push {}\n", self.op(arg)));
            }
        }
        o.push_str(&format!("    call {}\n", self.op(f)));
        if a.len() > 6 {
            o.push_str(&format!("    add rsp, {}\n", (a.len() - 6) * 8));
        }
        o.push_str(&format!("    mov {}, rax\n", self.op(d)));
        o.push_str("    pop r11\n    pop r10\n    pop r9\n    pop r8\n");
        o.push_str("    pop rdi\n    pop rsi\n    pop rdx\n    pop rcx\n");
        o
    }

    fn op(&mut self, op: &Operand) -> String {
        match op {
            Operand::Variable(n) | Operand::Temporary(n) => {
                if self.global_vars.iter().any(|(name, _)| name == n) {
                    return format!("[rel {}]", n);
                }
                if let Some(alloc) = self.allocator.get_allocation(n) {
                    match alloc {
                        Allocation::Register(reg) => reg.name().to_string(),
                        Allocation::Stack(offset) => {
                            if *offset < 0 { format!("[rbp-{}]", -offset) }
                            else { format!("[rbp+{}]", offset) }
                        }
                    }
                } else if let Some(&off) = self.param_offsets.get(n) {
                    format!("[rbp+{}]", off)
                } else if let Some(&off) = self.spill_offsets.get(n) {
                    if off < 0 { format!("[rbp-{}]", -off) }
                    else { format!("[rbp+{}]", off) }
                } else {
                    let off = self.spill_offsets.len() as i32 * 8 + 8;
                    let slot = format!("[rbp-{}]", off);
                    self.spill_offsets.insert(n.to_string(), -(off));
                    slot
                }
            }
            Operand::Label(n) => n.clone(),
            Operand::IntLiteral(v) => v.to_string(),
            Operand::FloatLiteral(v) => {
                let lb = format!("L_flt{}", self.string_counter);
                self.string_counter += 1;
                self.string_literals.push((lb.clone(), format!("dq {}", v)));
                format!("qword [{}]", lb)
            }
            Operand::BoolLiteral(v) => if *v { "1".to_string() } else { "0".to_string() },
            Operand::StringLiteral(s) => {
                let lb = format!(".L.str{}", self.string_counter);
                self.string_counter += 1;
                self.string_literals.push((lb.clone(), format!("db {}", self.escape_string(s))));
                lb
            }
            Operand::MemoryAddress { base, offset } => {
                if *offset == 0 { format!("[{}]", base) }
                else if *offset > 0 { format!("[{}+{}]", base, offset) }
                else { format!("[{}-{}]", base, -offset) }
            }
            Operand::ArrayAccess { base, index, stride: _ } => {
                format!("[{} + {}*8]", base, self.op(index))
            }
        }
    }

    fn lbl(&self, l: &Operand) -> String {
        match l {
            Operand::Label(n) => n.clone(),
            _ => format!("{:?}", l),
        }
    }

    fn escape_string(&self, s: &str) -> String {
        let mut e = String::new();
        for c in s.chars() {
            match c {
                '"' => e.push_str("\\\""),
                '\\' => e.push_str("\\\\"),
                '\n' => e.push_str("\\n"),
                '\r' => e.push_str("\\r"),
                '\t' => e.push_str("\\t"),
                _ => e.push(c),
            }
        }
        format!("\"{}\"", e)
    }
}

impl Default for X86Generator {
    fn default() -> Self { Self::new() }
}