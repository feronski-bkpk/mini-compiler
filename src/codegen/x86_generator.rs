//! Генератор x86-64 ассемблерного кода из IR

use crate::ir::{FunctionIR, IRInstruction, Operand, ProgramIR};
use std::collections::HashMap;

pub struct X86Generator {
    string_counter: usize,
    string_literals: Vec<(String, String)>,
    var_offsets: HashMap<String, i32>,
    temp_offsets: HashMap<String, i32>,
    stack_size: usize,
    current_function: Option<String>,
}

impl X86Generator {
    pub fn new() -> Self {
        Self {
            string_counter: 0,
            string_literals: Vec::new(),
            var_offsets: HashMap::new(),
            temp_offsets: HashMap::new(),
            stack_size: 0,
            current_function: None,
        }
    }

    pub fn generate(&mut self, program: &ProgramIR) -> super::CodegenResult {
        let mut output = String::new();
        let mut data_section = String::new();
        self.string_counter = 0;
        self.string_literals.clear();

        let mut text_output = String::new();
        text_output.push_str("section .text\n");
        text_output.push_str("default rel\n");
        text_output.push_str("global main\n");
        text_output.push_str("global _start\n\n");

        for func in &program.functions {
            text_output.push_str(&self.generate_function(func));
            text_output.push_str("\n");
        }

        text_output.push_str("_start:\n    call main\n    mov rdi, rax\n    call exit\n\n");
        text_output.push_str("exit:\n    mov rax, 60\n    syscall\n");

        if !self.string_literals.is_empty() {
            data_section.push_str("section .data\n");
            for (label, string) in &self.string_literals {
                data_section.push_str(&format!("{}: {}\n", label, string));
            }
        }

        output.push_str(&data_section);
        output.push_str("\n");
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

        super::CodegenResult {
            assembly: output,
            registers_used: vec!["rax", "rcx", "rdx", "rsi", "rdi", "r8", "r9", "r10", "r11"]
                .into_iter()
                .map(String::from)
                .collect(),
            frame_size: self.stack_size,
            instruction_count,
        }
    }

    fn generate_function(&mut self, func: &FunctionIR) -> String {
        let mut output = String::new();
        self.current_function = Some(func.name.clone());
        self.var_offsets.clear();
        self.temp_offsets.clear();

        output.push_str(&format!("{}:\n", func.name));
        output.push_str("    push rbp\n    mov rbp, rsp\n");

        let param_regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
        let mut param_offset = 16;
        for (i, (param_name, _)) in func.parameters.iter().enumerate() {
            if i < param_regs.len() {
                output.push_str(&format!(
                    "    mov [rbp+{}], {}\n",
                    param_offset, param_regs[i]
                ));
            }
            self.var_offsets.insert(param_name.clone(), param_offset);
            param_offset += 8;
        }

        let mut local_offset: i32 = 8;
        for (name, _) in &func.locals {
            self.var_offsets.insert(name.clone(), -local_offset);
            local_offset += 8;
        }

        let mut max_temp: i32 = 0;
        for block in func.blocks.values() {
            for instr in &block.instructions {
                match instr {
                    IRInstruction::Add(dest, _, _)
                    | IRInstruction::Sub(dest, _, _)
                    | IRInstruction::Mul(dest, _, _)
                    | IRInstruction::Div(dest, _, _)
                    | IRInstruction::Mod(dest, _, _)
                    | IRInstruction::Move(dest, _)
                    | IRInstruction::Load(dest, _)
                    | IRInstruction::Call(dest, _, _)
                    | IRInstruction::And(dest, _, _)
                    | IRInstruction::Or(dest, _, _)
                    | IRInstruction::Xor(dest, _, _)
                    | IRInstruction::Not(dest, _)
                    | IRInstruction::Neg(dest, _)
                    | IRInstruction::IntToFloat(dest, _)
                    | IRInstruction::FloatToInt(dest, _) => {
                        if let Operand::Temporary(name) = dest {
                            if let Ok(num) = name[1..].parse::<i32>() {
                                if num > max_temp {
                                    max_temp = num;
                                }
                            }
                        }
                    }
                    _ => {}
                }
                for op in instr.operands() {
                    if let Operand::Temporary(name) = op {
                        if let Ok(num) = name[1..].parse::<i32>() {
                            if num > max_temp {
                                max_temp = num;
                            }
                        }
                    }
                }
            }
        }

        let locals_size = (local_offset - 8) as usize;
        let temps_size = (max_temp as usize) * 8;
        let total_size = locals_size + temps_size;
        if total_size > 0 {
            let aligned = ((total_size + 15) & !15) as i32;
            output.push_str(&format!("    sub rsp, {}\n", aligned));
            self.stack_size = aligned as usize;
        }

        for i in 1..=max_temp {
            let temp_name = format!("t{}", i);
            let offset = locals_size as i32 + (i - 1) * 8 + 8;
            self.temp_offsets.insert(temp_name, -(offset));
        }

        let mut blocks: Vec<&String> = func.blocks.keys().collect();
        blocks.sort();
        for block_label in blocks {
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

    fn generate_instruction(&mut self, instr: &IRInstruction) -> String {
        match instr {
            IRInstruction::Move(d, s) => self.gen_move(d, s),
            IRInstruction::Return(Some(v)) => format!(
                "    mov rax, {}\n    mov rsp, rbp\n    pop rbp\n    ret\n",
                self.op(v)
            ),
            IRInstruction::Return(None) => "    mov rsp, rbp\n    pop rbp\n    ret\n".into(),
            IRInstruction::Add(d, l, r) => self.gen_binop("add", d, l, r),
            IRInstruction::Sub(d, l, r) => self.gen_binop("sub", d, l, r),
            IRInstruction::Mul(d, l, r) => self.gen_binop("imul", d, l, r),
            IRInstruction::Div(d, l, r) => self.gen_div(d, l, r, false),
            IRInstruction::Mod(d, l, r) => self.gen_div(d, l, r, true),
            IRInstruction::And(d, l, r) => self.gen_binop("and", d, l, r),
            IRInstruction::Or(d, l, r) => self.gen_binop("or", d, l, r),
            IRInstruction::Xor(d, l, r) => self.gen_binop("xor", d, l, r),
            IRInstruction::Not(d, o) => format!(
                "    mov rax, {}\n    not rax\n    mov {}, rax\n",
                self.op(o),
                self.op(d)
            ),
            IRInstruction::Neg(d, o) => format!(
                "    mov rax, {}\n    neg rax\n    mov {}, rax\n",
                self.op(o),
                self.op(d)
            ),
            IRInstruction::CmpEq(d, l, r) => self.gen_cmp("sete", d, l, r, false),
            IRInstruction::CmpNe(d, l, r) => self.gen_cmp("setne", d, l, r, false),
            IRInstruction::CmpLt(d, l, r) => self.gen_cmp("setl", d, l, r, false),
            IRInstruction::CmpLe(d, l, r) => self.gen_cmp("setle", d, l, r, false),
            IRInstruction::CmpGt(d, l, r) => self.gen_cmp("setg", d, l, r, false),
            IRInstruction::CmpGe(d, l, r) => self.gen_cmp("setge", d, l, r, false),
            IRInstruction::CmpEqF(d, l, r) => self.gen_cmp("sete", d, l, r, true),
            IRInstruction::CmpNeF(d, l, r) => self.gen_cmp("setne", d, l, r, true),
            IRInstruction::CmpLtF(d, l, r) => self.gen_cmp("setb", d, l, r, true),
            IRInstruction::CmpLeF(d, l, r) => self.gen_cmp("setbe", d, l, r, true),
            IRInstruction::CmpGtF(d, l, r) => self.gen_cmp("seta", d, l, r, true),
            IRInstruction::CmpGeF(d, l, r) => self.gen_cmp("setae", d, l, r, true),
            IRInstruction::CmpLtU(d, l, r) => self.gen_cmp("setb", d, l, r, false),
            IRInstruction::CmpLeU(d, l, r) => self.gen_cmp("setbe", d, l, r, false),
            IRInstruction::CmpGtU(d, l, r) => self.gen_cmp("seta", d, l, r, false),
            IRInstruction::CmpGeU(d, l, r) => self.gen_cmp("setae", d, l, r, false),
            IRInstruction::IntToFloat(d, s) => {
                format!(
                    "    cvtsi2sd xmm0, {}\n    movq {}, xmm0\n",
                    self.op(s),
                    self.op(d)
                )
            }
            IRInstruction::FloatToInt(d, s) => {
                let ss = self.op(s);
                if ss.starts_with('[') {
                    format!(
                        "    cvttsd2si rax, qword {}\n    mov {}, rax\n",
                        ss,
                        self.op(d)
                    )
                } else {
                    format!("    cvttsd2si rax, {}\n    mov {}, rax\n", ss, self.op(d))
                }
            }
            IRInstruction::Jump(l) => format!("    jmp .{}\n", self.lbl(l)),
            IRInstruction::JumpIf(c, l) => {
                if matches!(c, Operand::BoolLiteral(true))
                    || matches!(c, Operand::IntLiteral(v) if *v != 0)
                {
                    return format!("    jmp .{}\n", self.lbl(l));
                }
                if matches!(c, Operand::BoolLiteral(false)) || matches!(c, Operand::IntLiteral(0)) {
                    return String::new();
                }
                let cs = self.op(c);
                let ls = self.lbl(l);
                if cs.starts_with('[') {
                    format!("    cmp qword {}, 0\n    jne .{}\n", cs, ls)
                } else {
                    format!("    cmp {}, 0\n    jne .{}\n", cs, ls)
                }
            }
            IRInstruction::JumpIfNot(c, l) => {
                if matches!(c, Operand::BoolLiteral(false)) || matches!(c, Operand::IntLiteral(0)) {
                    return format!("    jmp .{}\n", self.lbl(l));
                }
                if matches!(c, Operand::BoolLiteral(true))
                    || matches!(c, Operand::IntLiteral(v) if *v != 0)
                {
                    return String::new();
                }
                let cs = self.op(c);
                let ls = self.lbl(l);
                if cs.starts_with('[') {
                    format!("    cmp qword {}, 0\n    je .{}\n", cs, ls)
                } else {
                    format!("    cmp {}, 0\n    je .{}\n", cs, ls)
                }
            }
            IRInstruction::ArrayLoad(dest, base, index) => {
                let ds = self.op(dest);
                let bs = self.op(base);
                let istr = self.op(index);
                // Вычисляем адрес: base + index * 8
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
            IRInstruction::Load(d, a) => format!(
                "    mov rax, [{}]\n    mov {}, rax\n",
                self.op(a),
                self.op(d)
            ),
            IRInstruction::Store(a, s) => {
                format!("    mov qword [{}], {}\n", self.op(a), self.op(s))
            }
            IRInstruction::Alloca(d, sz) => {
                format!("    sub rsp, {}\n    mov {}, rsp\n", sz, self.op(d))
            }
            _ => String::new(),
        }
    }

    fn gen_move(&mut self, d: &Operand, s: &Operand) -> String {
        let ds = self.op(d);

        if let Operand::FloatLiteral(v) = s {
            let lb = format!("L_flt{}", self.string_counter);
            self.string_counter += 1;
            self.string_literals.push((lb.clone(), format!("dq {}", v)));
            return format!(
                "    movsd xmm0, qword [{}]\n    movsd qword {}, xmm0\n",
                lb, ds
            );
        }

        let ss = self.op(s);
        if ss.parse::<i32>().is_ok() || ss == "0" || ss == "1" {
            format!("    mov qword {}, {}\n", ds, ss)
        } else if ds.starts_with('[') && ss.starts_with('[') {
            format!("    mov rax, qword {}\n    mov qword {}, rax\n", ss, ds)
        } else if ds.starts_with('[') {
            format!("    mov qword {}, {}\n", ds, ss)
        } else if ss.starts_with('[') {
            format!("    mov {}, qword {}\n", ds, ss)
        } else {
            format!("    mov {}, {}\n", ds, ss)
        }
    }

    fn gen_binop(&mut self, op: &str, d: &Operand, l: &Operand, r: &Operand) -> String {
        let ls = self.op(l);
        let rs = self.op(r);
        let ds = self.op(d);

        // Проверяем, float ли операнды
        let is_float = matches!(l, Operand::FloatLiteral(_))
            || matches!(r, Operand::FloatLiteral(_))
            || ls.contains("flt")
            || rs.contains("flt");

        if is_float {
            // Используем SSE для float
            let op_f = match op {
                "add" => "addsd",
                "sub" => "subsd",
                "imul" => "mulsd",
                "and" => "andpd",
                "or" => "orpd",
                "xor" => "xorpd",
                _ => op,
            };
            if ls.starts_with('[') {
                format!(
                    "    movsd xmm0, qword {}\n    {} xmm0, qword {}\n    movsd qword {}, xmm0\n",
                    ls, op_f, rs, ds
                )
            } else {
                format!(
                    "    mov rax, {}\n    movq xmm0, rax\n    {} xmm0, qword {}\n    movsd qword {}, xmm0\n",
                    ls, op_f, rs, ds
                )
            }
        } else {
            let rsz = if rs.starts_with('[') {
                format!("qword {}", rs)
            } else {
                rs
            };
            if ls.starts_with('[') {
                format!(
                    "    mov rax, qword {}\n    {} rax, {}\n    mov {}, rax\n",
                    ls, op, rsz, ds
                )
            } else {
                format!(
                    "    mov rax, {}\n    {} rax, {}\n    mov {}, rax\n",
                    ls, op, rsz, ds
                )
            }
        }
    }

    fn gen_div(&mut self, d: &Operand, l: &Operand, r: &Operand, m: bool) -> String {
        let ls = self.op(l);
        let rs = self.op(r);
        let ds = self.op(d);
        let rr = if m { "rdx" } else { "rax" };
        let mut o = format!("    mov rax, {}\n    cdq\n", ls);
        if rs.parse::<i32>().is_ok() {
            o.push_str(&format!("    mov rcx, {}\n    idiv rcx\n", rs));
        } else if rs.starts_with('[') {
            o.push_str(&format!("    idiv qword {}\n", rs));
        } else {
            o.push_str(&format!("    idiv {}\n", rs));
        }
        o.push_str(&format!("    mov {}, {}\n", ds, rr));
        o
    }

    fn gen_cmp(&mut self, set: &str, d: &Operand, l: &Operand, r: &Operand, flt: bool) -> String {
        let ls = self.op(l);
        let rs = self.op(r);
        let ds = self.op(d);
        let mut o = String::new();
        if flt {
            if ls.starts_with('[') {
                o.push_str(&format!("    movsd xmm0, qword {}\n", ls));
            } else {
                o.push_str(&format!("    mov rax, {}\n    movq xmm0, rax\n", ls));
            }
            if rs.starts_with('[') {
                o.push_str(&format!("    ucomisd xmm0, qword {}\n", rs));
            } else {
                o.push_str(&format!(
                    "    mov rax, {}\n    movq xmm1, rax\n    ucomisd xmm0, xmm1\n",
                    rs
                ));
            }
        } else {
            let li = ls.parse::<i32>().is_ok();
            let ri = rs.parse::<i32>().is_ok();
            if li && ri {
                o.push_str(&format!("    mov rax, {}\n    cmp rax, {}\n", ls, rs));
            } else if ls.starts_with('[') && ri {
                o.push_str(&format!("    mov rax, qword {}\n    cmp rax, {}\n", ls, rs));
            } else if li && rs.starts_with('[') {
                o.push_str(&format!("    mov rax, {}\n    cmp rax, qword {}\n", ls, rs));
            } else if ls.starts_with('[') && rs.starts_with('[') {
                o.push_str(&format!(
                    "    mov rax, qword {}\n    cmp rax, qword {}\n",
                    ls, rs
                ));
            } else if ls.starts_with('[') {
                o.push_str(&format!("    mov rax, qword {}\n    cmp rax, {}\n", ls, rs));
            } else if rs.starts_with('[') {
                o.push_str(&format!("    mov rax, {}\n    cmp rax, qword {}\n", ls, rs));
            } else {
                o.push_str(&format!("    mov rax, {}\n    cmp rax, {}\n", ls, rs));
            }
        }
        o.push_str(&format!(
            "    {} al\n    movzx rax, al\n    mov {}, rax\n",
            set, ds
        ));
        o
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
            Operand::Variable(n) => {
                if let Some(&off) = self.var_offsets.get(n) {
                    if off > 0 {
                        format!("[rbp+{}]", off)
                    } else {
                        format!("[rbp-{}]", -off)
                    }
                } else if let Some(&off) = self.temp_offsets.get(n) {
                    format!("[rbp-{}]", -off)
                } else {
                    "[rbp-8]".into()
                }
            }
            Operand::Temporary(n) => {
                if let Some(&off) = self.temp_offsets.get(n) {
                    format!("[rbp-{}]", -off)
                } else {
                    let tn: i32 = n[1..].parse().unwrap_or(0);
                    let lsz = self.var_offsets.len() as i32 * 8;
                    format!("[rbp-{}]", lsz + (tn - 1) * 8 + 8)
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
            Operand::BoolLiteral(v) => if *v { "1" } else { "0" }.into(),
            Operand::StringLiteral(s) => {
                let lb = format!(".L.str{}", self.string_counter);
                self.string_counter += 1;
                self.string_literals
                    .push((lb.clone(), format!("db {}", self.escape_string(s))));
                lb
            }
            Operand::MemoryAddress { base, offset } => {
                if *offset == 0 {
                    format!("[{}]", base)
                } else if *offset > 0 {
                    format!("[{}+{}]", base, offset)
                } else {
                    format!("[{}-{}]", base, -offset)
                }
            }
            Operand::ArrayAccess {
                base,
                index,
                stride: _,
            } => {
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
    fn default() -> Self {
        Self::new()
    }
}
