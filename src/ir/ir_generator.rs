//! Генератор IR из декорированного AST

use super::basic_block::{BasicBlock, FunctionIR, ProgramIR};
use super::ir_instructions::{IRInstruction, Operand};
use crate::parser::ast::*;
use crate::semantic::symbol_table::SymbolTable;
use std::collections::HashMap;

pub struct IRGenerator {
    pub symbol_table: SymbolTable,
    program: ProgramIR,
    current_function: Option<String>,
    temp_counter: usize,
    label_counter: usize,
    var_to_temp: HashMap<String, String>,
    current_locals: Vec<(String, String)>,
    function_counter: usize,
    break_labels: Vec<String>,
    continue_labels: Vec<String>,
}

impl IRGenerator {
    pub fn new(symbol_table: SymbolTable) -> Self {
        Self {
            symbol_table,
            program: ProgramIR::new(),
            current_function: None,
            temp_counter: 0,
            label_counter: 0,
            var_to_temp: HashMap::new(),
            current_locals: Vec::new(),
            function_counter: 0,
            break_labels: Vec::new(),
            continue_labels: Vec::new(),
        }
    }

    fn new_temp(&mut self) -> Operand {
        self.temp_counter += 1;
        Operand::Temporary(format!("t{}", self.temp_counter))
    }

    fn new_label(&mut self) -> Operand {
        self.label_counter += 1;
        let func_prefix = format!("L{}_{:03}", self.function_counter, self.label_counter);
        Operand::Label(func_prefix)
    }

    fn label_to_string(label: &Operand) -> String {
        match label {
            Operand::Label(l) => l.clone(),
            _ => panic!("Expected label operand"),
        }
    }

    pub fn generate(&mut self, program: Program) -> ProgramIR {
        let mut func_list = Vec::new();
        let mut global_vars = Vec::new();
        for decl in program.declarations {
            match decl {
                Declaration::Function(func) => func_list.push(func),
                Declaration::Struct(_) => {}
                Declaration::Variable(var) => global_vars.push(var),
            }
        }
        for var in global_vars {
            self.program.add_global(var.name, var.var_type.to_string());
        }
        func_list.sort_by(|a, b| {
            if a.name == "main" {
                std::cmp::Ordering::Less
            } else if b.name == "main" {
                std::cmp::Ordering::Greater
            } else {
                a.name.cmp(&b.name)
            }
        });
        for func in &func_list {
            self.generate_function(func);
        }
        self.program.sort_functions_by_name();
        self.program.clone()
    }

    fn generate_function(&mut self, func: &FunctionDecl) {
        let return_type = func.return_type.to_string();
        let mut func_ir = FunctionIR::new(func.name.clone(), return_type);
        for param in &func.parameters {
            let param_type = param.param_type.to_string();
            func_ir.parameters.push((param.name.clone(), param_type));
        }
        self.current_function = Some(func.name.clone());
        self.temp_counter = 0;
        self.label_counter = 0;
        self.function_counter += 1;
        self.var_to_temp.clear();
        self.current_locals.clear();
        self.break_labels.clear();
        self.continue_labels.clear();
        let mut all_blocks: Vec<BasicBlock> = Vec::new();
        let mut current_block = self.create_block("entry");
        let entry_label = current_block.label.clone();
        for stmt in &func.body.statements {
            self.generate_statement(stmt, &mut current_block, &mut all_blocks);
        }
        all_blocks.push(current_block);
        func_ir.set_entry(entry_label);
        for (name, typ) in &self.current_locals {
            func_ir.locals.push((name.clone(), typ.clone()));
        }
        for block in &all_blocks {
            func_ir.add_block(block.clone());
        }
        super::control_flow::build_cfg(&mut func_ir);
        self.program.add_function(func_ir);
        self.current_function = None;
    }

    fn create_block(&mut self, suffix: &str) -> BasicBlock {
        let label = self.new_label();
        let mut label_str = Self::label_to_string(&label);
        label_str.push_str(&format!("_{}", suffix));
        BasicBlock::new(label_str)
    }

    fn generate_statement(
        &mut self,
        stmt: &Statement,
        current_block: &mut BasicBlock,
        all_blocks: &mut Vec<BasicBlock>,
    ) {
        match stmt {
            Statement::VariableDecl(var) => {
                self.current_locals
                    .push((var.name.clone(), var.var_type.to_string()));
                if let Some(init) = &var.initializer {
                    let value = self.generate_expression(init, current_block, all_blocks);
                    current_block.add_instruction(IRInstruction::Move(
                        Operand::Variable(var.name.clone()),
                        value,
                    ));
                }
            }
            Statement::Expression(es) => {
                self.generate_expression(&es.expr, current_block, all_blocks);
            }
            Statement::If(is) => {
                let mb = self.generate_if_statement(is, current_block, all_blocks);
                let old = std::mem::replace(current_block, mb);
                all_blocks.push(old);
            }
            Statement::While(ws) => {
                let mb = self.generate_while_statement(ws, current_block, all_blocks);
                let old = std::mem::replace(current_block, mb);
                all_blocks.push(old);
            }
            Statement::For(fs) => {
                let mb = self.generate_for_statement(fs, current_block, all_blocks);
                let old = std::mem::replace(current_block, mb);
                all_blocks.push(old);
            }
            Statement::Return(rs) => {
                if let Some(v) = &rs.value {
                    let val = self.generate_expression(v, current_block, all_blocks);
                    current_block.add_instruction(IRInstruction::Return(Some(val)));
                } else {
                    current_block.add_instruction(IRInstruction::Return(None));
                }
            }
            Statement::Break(_) => {
                if let Some(label) = self.break_labels.last() {
                    current_block
                        .add_instruction(IRInstruction::Jump(Operand::Label(label.clone())));
                }
            }
            Statement::Continue(_) => {
                if let Some(label) = self.continue_labels.last() {
                    current_block
                        .add_instruction(IRInstruction::Jump(Operand::Label(label.clone())));
                }
            }
            Statement::Switch(ss) => {
                let mb = self.generate_switch(ss, current_block, all_blocks);
                let old = std::mem::replace(current_block, mb);
                all_blocks.push(old);
            }
            Statement::Block(bs) => {
                for s in &bs.statements {
                    self.generate_statement(s, current_block, all_blocks);
                }
            }
            Statement::Empty(_) => {}
        }
    }

    fn is_float_operand(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Identifier(i) => self
                .symbol_table
                .lookup(&i.name)
                .map(|s| matches!(s.typ, crate::semantic::type_system::Type::Float))
                .unwrap_or(false),
            Expression::Literal(l) => matches!(l.value, LiteralValue::Float(_)),
            Expression::Binary(b) => {
                self.is_float_operand(&b.left) || self.is_float_operand(&b.right)
            }
            Expression::Grouped(g) => self.is_float_operand(&g.expr),
            Expression::Unary(u) => self.is_float_operand(&u.operand),
            _ => false,
        }
    }

    fn get_expression_type(&self, expr: &Expression) -> Option<crate::semantic::type_system::Type> {
        match expr {
            Expression::Identifier(i) => self.symbol_table.lookup(&i.name).map(|s| s.typ.clone()),
            Expression::Literal(l) => match &l.value {
                LiteralValue::Int(_) => Some(crate::semantic::type_system::Type::Int),
                LiteralValue::Float(_) => Some(crate::semantic::type_system::Type::Float),
                LiteralValue::Bool(_) => Some(crate::semantic::type_system::Type::Bool),
                LiteralValue::String(_) => Some(crate::semantic::type_system::Type::String),
            },
            _ => None,
        }
    }

    fn convert_type(
        &mut self,
        expr: Operand,
        from: &crate::semantic::type_system::Type,
        to: &crate::semantic::type_system::Type,
        cb: &mut BasicBlock,
    ) -> Operand {
        match (from, to) {
            (
                crate::semantic::type_system::Type::Int,
                crate::semantic::type_system::Type::Float,
            ) => {
                let d = self.new_temp();
                cb.add_instruction(IRInstruction::IntToFloat(d.clone(), expr));
                d
            }
            (
                crate::semantic::type_system::Type::Float,
                crate::semantic::type_system::Type::Int,
            ) => {
                let d = self.new_temp();
                cb.add_instruction(IRInstruction::FloatToInt(d.clone(), expr));
                d
            }
            _ => expr,
        }
    }

    fn generate_expression(
        &mut self,
        expr: &Expression,
        current_block: &mut BasicBlock,
        all_blocks: &mut Vec<BasicBlock>,
    ) -> Operand {
        match expr {
            Expression::Literal(l) => self.generate_literal(l),
            Expression::Identifier(i) => self.generate_identifier(i),
            Expression::Binary(b) => match b.operator {
                BinaryOp::And => self.generate_short_circuit_and(b, current_block, all_blocks),
                BinaryOp::Or => self.generate_short_circuit_or(b, current_block, all_blocks),
                _ => {
                    let mut left = self.generate_expression(&b.left, current_block, all_blocks);
                    let mut right = self.generate_expression(&b.right, current_block, all_blocks);
                    let lt = self.get_expression_type(&b.left);
                    let rt = self.get_expression_type(&b.right);
                    match (&lt, &rt) {
                        (
                            Some(crate::semantic::type_system::Type::Int),
                            Some(crate::semantic::type_system::Type::Float),
                        ) => {
                            left = self.convert_type(
                                left,
                                &crate::semantic::type_system::Type::Int,
                                &crate::semantic::type_system::Type::Float,
                                current_block,
                            );
                        }
                        (
                            Some(crate::semantic::type_system::Type::Float),
                            Some(crate::semantic::type_system::Type::Int),
                        ) => {
                            right = self.convert_type(
                                right,
                                &crate::semantic::type_system::Type::Int,
                                &crate::semantic::type_system::Type::Float,
                                current_block,
                            );
                        }
                        _ => {}
                    }
                    let d = self.new_temp();
                    let is_float =
                        self.is_float_operand(&b.left) || self.is_float_operand(&b.right);
                    let instr = match b.operator {
                        BinaryOp::Add => IRInstruction::Add(d.clone(), left, right),
                        BinaryOp::Sub => IRInstruction::Sub(d.clone(), left, right),
                        BinaryOp::Mul => IRInstruction::Mul(d.clone(), left, right),
                        BinaryOp::Div => IRInstruction::Div(d.clone(), left, right),
                        BinaryOp::Mod => IRInstruction::Mod(d.clone(), left, right),
                        BinaryOp::Eq => {
                            if is_float {
                                IRInstruction::CmpEqF(d.clone(), left, right)
                            } else {
                                IRInstruction::CmpEq(d.clone(), left, right)
                            }
                        }
                        BinaryOp::Ne => {
                            if is_float {
                                IRInstruction::CmpNeF(d.clone(), left, right)
                            } else {
                                IRInstruction::CmpNe(d.clone(), left, right)
                            }
                        }
                        BinaryOp::Lt => {
                            if is_float {
                                IRInstruction::CmpLtF(d.clone(), left, right)
                            } else {
                                IRInstruction::CmpLt(d.clone(), left, right)
                            }
                        }
                        BinaryOp::Le => {
                            if is_float {
                                IRInstruction::CmpLeF(d.clone(), left, right)
                            } else {
                                IRInstruction::CmpLe(d.clone(), left, right)
                            }
                        }
                        BinaryOp::Gt => {
                            if is_float {
                                IRInstruction::CmpGtF(d.clone(), left, right)
                            } else {
                                IRInstruction::CmpGt(d.clone(), left, right)
                            }
                        }
                        BinaryOp::Ge => {
                            if is_float {
                                IRInstruction::CmpGeF(d.clone(), left, right)
                            } else {
                                IRInstruction::CmpGe(d.clone(), left, right)
                            }
                        }
                        _ => unreachable!(),
                    };
                    current_block.add_instruction(instr);
                    d
                }
            },
            Expression::Unary(u) => {
                let op = self.generate_expression(&u.operand, current_block, all_blocks);
                let d = self.new_temp();
                match u.operator {
                    UnaryOp::Neg => {
                        current_block.add_instruction(IRInstruction::Neg(d.clone(), op))
                    }
                    UnaryOp::Not => {
                        let one = Operand::IntLiteral(1);
                        current_block.add_instruction(IRInstruction::Xor(d.clone(), op, one));
                    }
                    UnaryOp::Plus => {
                        current_block.add_instruction(IRInstruction::Move(d.clone(), op))
                    }
                    UnaryOp::PreIncrement => {
                        let one = Operand::IntLiteral(1);
                        current_block.add_instruction(IRInstruction::Add(
                            d.clone(),
                            op.clone(),
                            one,
                        ));
                        current_block.add_instruction(IRInstruction::Move(op, d.clone()));
                    }
                    UnaryOp::PostIncrement => {
                        let t = self.new_temp();
                        let one = Operand::IntLiteral(1);
                        current_block.add_instruction(IRInstruction::Move(t.clone(), op.clone()));
                        current_block.add_instruction(IRInstruction::Add(
                            d.clone(),
                            op.clone(),
                            one,
                        ));
                        current_block.add_instruction(IRInstruction::Move(op, d.clone()));
                        return t;
                    }
                    UnaryOp::PreDecrement => {
                        let one = Operand::IntLiteral(1);
                        current_block.add_instruction(IRInstruction::Sub(
                            d.clone(),
                            op.clone(),
                            one,
                        ));
                        current_block.add_instruction(IRInstruction::Move(op, d.clone()));
                    }
                    UnaryOp::PostDecrement => {
                        let t = self.new_temp();
                        let one = Operand::IntLiteral(1);
                        current_block.add_instruction(IRInstruction::Move(t.clone(), op.clone()));
                        current_block.add_instruction(IRInstruction::Sub(
                            d.clone(),
                            op.clone(),
                            one,
                        ));
                        current_block.add_instruction(IRInstruction::Move(op, d.clone()));
                        return t;
                    }
                }
                d
            }
            Expression::Assignment(a) => {
                let val = self.generate_expression(&a.value, current_block, all_blocks);
                let tgt = match &*a.target {
                    Expression::Identifier(i) => Operand::Variable(i.name.clone()),
                    _ => self.generate_expression(&a.target, current_block, all_blocks),
                };
                match a.operator {
                    AssignmentOp::Assign => {
                        current_block.add_instruction(IRInstruction::Move(tgt.clone(), val.clone()))
                    }
                    AssignmentOp::AddAssign => {
                        let t = self.new_temp();
                        current_block.add_instruction(IRInstruction::Add(
                            t.clone(),
                            tgt.clone(),
                            val.clone(),
                        ));
                        current_block.add_instruction(IRInstruction::Move(tgt.clone(), t));
                    }
                    AssignmentOp::SubAssign => {
                        let t = self.new_temp();
                        current_block.add_instruction(IRInstruction::Sub(
                            t.clone(),
                            tgt.clone(),
                            val.clone(),
                        ));
                        current_block.add_instruction(IRInstruction::Move(tgt.clone(), t));
                    }
                    AssignmentOp::MulAssign => {
                        let t = self.new_temp();
                        current_block.add_instruction(IRInstruction::Mul(
                            t.clone(),
                            tgt.clone(),
                            val.clone(),
                        ));
                        current_block.add_instruction(IRInstruction::Move(tgt.clone(), t));
                    }
                    AssignmentOp::DivAssign => {
                        let t = self.new_temp();
                        current_block.add_instruction(IRInstruction::Div(
                            t.clone(),
                            tgt.clone(),
                            val.clone(),
                        ));
                        current_block.add_instruction(IRInstruction::Move(tgt.clone(), t));
                    }
                }
                val
            }
            Expression::Call(c) => self.generate_call(c, current_block, all_blocks),
            Expression::StructAccess(sa) => {
                self.generate_expression(&sa.object, current_block, all_blocks)
            }
            Expression::Grouped(g) => self.generate_expression(&g.expr, current_block, all_blocks),
            Expression::ArrayAccess(aa) => {
                let arr = self.generate_expression(&aa.array, current_block, all_blocks);
                let idx = self.generate_expression(&aa.index, current_block, all_blocks);
                let d = self.new_temp();
                current_block.add_instruction(IRInstruction::ArrayLoad(d.clone(), arr, idx));
                d
            }
        }
    }

    fn generate_short_circuit_and(
        &mut self,
        b: &BinaryExpr,
        cb: &mut BasicBlock,
        ab: &mut Vec<BasicBlock>,
    ) -> Operand {
        let left = self.generate_expression(&b.left, cb, ab);
        let r = self.new_temp();
        let el = self.new_label();
        let ml = self.new_label();
        cb.add_instruction(IRInstruction::Move(r.clone(), Operand::IntLiteral(0)));
        cb.add_instruction(IRInstruction::JumpIfNot(left, ml.clone()));
        let mut eb = BasicBlock::new(Self::label_to_string(&el));
        let right = self.generate_expression(&b.right, &mut eb, ab);
        eb.add_instruction(IRInstruction::Move(r.clone(), right));
        eb.add_instruction(IRInstruction::Jump(ml.clone()));
        ab.push(eb);
        let old = std::mem::replace(cb, BasicBlock::new(Self::label_to_string(&ml)));
        ab.push(old);
        r
    }

    fn generate_short_circuit_or(
        &mut self,
        b: &BinaryExpr,
        cb: &mut BasicBlock,
        ab: &mut Vec<BasicBlock>,
    ) -> Operand {
        let left = self.generate_expression(&b.left, cb, ab);
        let r = self.new_temp();
        let el = self.new_label();
        let ml = self.new_label();
        cb.add_instruction(IRInstruction::Move(r.clone(), Operand::IntLiteral(1)));
        cb.add_instruction(IRInstruction::JumpIf(left, ml.clone()));
        let mut eb = BasicBlock::new(Self::label_to_string(&el));
        let right = self.generate_expression(&b.right, &mut eb, ab);
        eb.add_instruction(IRInstruction::Move(r.clone(), right));
        eb.add_instruction(IRInstruction::Jump(ml.clone()));
        ab.push(eb);
        let old = std::mem::replace(cb, BasicBlock::new(Self::label_to_string(&ml)));
        ab.push(old);
        r
    }

    fn generate_if_statement(
        &mut self,
        is: &IfStmt,
        cb: &mut BasicBlock,
        ab: &mut Vec<BasicBlock>,
    ) -> BasicBlock {
        if cb.is_terminator() {
            let old = std::mem::replace(cb, self.create_block("cond_cont"));
            ab.push(old);
        }
        let cond = self.generate_expression(&is.condition, cb, ab);
        let mut tb = self.create_block("then");
        let mut eb_opt = if is.else_branch.is_some() {
            Some(self.create_block("else"))
        } else {
            None
        };
        let mb = self.create_block("endif");
        let tl = Operand::Label(tb.label.clone());
        let el = eb_opt.as_ref().map(|b| Operand::Label(b.label.clone()));
        let ml = Operand::Label(mb.label.clone());
        if let Some(ref e) = el {
            cb.add_instruction(IRInstruction::JumpIfNot(cond, e.clone()));
        } else {
            cb.add_instruction(IRInstruction::JumpIfNot(cond, ml.clone()));
        }
        cb.add_instruction(IRInstruction::Jump(tl.clone()));
        self.generate_statement(&is.then_branch, &mut tb, ab);
        if !tb.is_terminator() {
            tb.add_instruction(IRInstruction::Jump(ml.clone()));
        }
        ab.push(tb);
        if let Some(ebr) = &is.else_branch {
            if let Some(ref mut eb) = eb_opt {
                self.generate_statement(ebr, eb, ab);
                if !eb.is_terminator() {
                    eb.add_instruction(IRInstruction::Jump(ml.clone()));
                }
                ab.push(eb.clone());
            }
        }
        mb
    }

    fn generate_while_statement(
        &mut self,
        ws: &WhileStmt,
        cb: &mut BasicBlock,
        ab: &mut Vec<BasicBlock>,
    ) -> BasicBlock {
        let mut condb = self.create_block("while_cond");
        let mut bodyb = self.create_block("while_body");
        let mb = self.create_block("while_end");
        let cl = Operand::Label(condb.label.clone());
        let bl = Operand::Label(bodyb.label.clone());
        let ml = Operand::Label(mb.label.clone());
        self.break_labels.push(mb.label.clone());
        self.continue_labels.push(condb.label.clone());
        cb.add_instruction(IRInstruction::Jump(cl.clone()));
        let cond = self.generate_expression(&ws.condition, &mut condb, ab);
        if condb.is_terminator() {
            let old = std::mem::replace(&mut condb, self.create_block("while_cond_cont"));
            ab.push(old);
        }
        condb.add_instruction(IRInstruction::JumpIfNot(cond, ml.clone()));
        condb.add_instruction(IRInstruction::Jump(bl.clone()));
        ab.push(condb);
        self.generate_statement(&ws.body, &mut bodyb, ab);
        if !bodyb.is_terminator() {
            bodyb.add_instruction(IRInstruction::Jump(cl));
        }
        ab.push(bodyb);
        self.break_labels.pop();
        self.continue_labels.pop();
        mb
    }

    fn generate_for_statement(
        &mut self,
        fs: &ForStmt,
        cb: &mut BasicBlock,
        ab: &mut Vec<BasicBlock>,
    ) -> BasicBlock {
        if let Some(mb) = self.optimize_counted_loop(fs, cb, ab) {
            return mb;
        }
        if let Some(init) = &fs.init {
            self.generate_statement(init, cb, ab);
        }
        let mut condb = self.create_block("for_cond");
        let mut bodyb = self.create_block("for_body");
        let mut updb = self.create_block("for_update");
        let mb = self.create_block("for_end");
        let cl = Operand::Label(condb.label.clone());
        let bl = Operand::Label(bodyb.label.clone());
        let ul = Operand::Label(updb.label.clone());
        let ml = Operand::Label(mb.label.clone());
        self.break_labels.push(mb.label.clone());
        self.continue_labels.push(updb.label.clone());
        cb.add_instruction(IRInstruction::Jump(cl.clone()));
        if let Some(cond) = &fs.condition {
            let c = self.generate_expression(cond, &mut condb, ab);
            condb.add_instruction(IRInstruction::JumpIfNot(c, ml.clone()));
            condb.add_instruction(IRInstruction::Jump(bl.clone()));
        } else {
            condb.add_instruction(IRInstruction::Jump(bl.clone()));
        }
        ab.push(condb);
        self.generate_statement(&fs.body, &mut bodyb, ab);
        bodyb.add_instruction(IRInstruction::Jump(ul.clone()));
        ab.push(bodyb);
        if let Some(upd) = &fs.update {
            self.generate_expression(upd, &mut updb, ab);
        }
        updb.add_instruction(IRInstruction::Jump(cl));
        ab.push(updb);
        self.break_labels.pop();
        self.continue_labels.pop();
        mb
    }

    fn generate_call(
        &mut self,
        call: &CallExpr,
        cb: &mut BasicBlock,
        ab: &mut Vec<BasicBlock>,
    ) -> Operand {
        let fname = match &*call.callee {
            Expression::Identifier(i) => i.name.clone(),
            _ => return self.new_temp(),
        };
        let mut args = Vec::new();
        for (i, arg) in call.arguments.iter().enumerate() {
            let av = self.generate_expression(arg, cb, ab);
            cb.add_instruction(IRInstruction::Param(i as u32, av.clone()));
            args.push(av);
        }
        let d = self.new_temp();
        cb.add_instruction(IRInstruction::Call(d.clone(), Operand::Label(fname), args));
        d
    }

    fn optimize_counted_loop(
        &mut self,
        fs: &ForStmt,
        cb: &mut BasicBlock,
        ab: &mut Vec<BasicBlock>,
    ) -> Option<BasicBlock> {
        if let (Some(init), Some(cond), Some(update)) = (&fs.init, &fs.condition, &fs.update) {
            if let Statement::Expression(es) = init.as_ref() {
                if let Expression::Assignment(a) = &*es.expr {
                    if a.operator == AssignmentOp::Assign {
                        if let Expression::Identifier(id) = &*a.target {
                            let lv = id.name.clone();
                            if let Expression::Literal(lit) = &*a.value {
                                if let LiteralValue::Int(sv) = lit.value {
                                    if let Expression::Binary(bin) = cond.as_ref() {
                                        if bin.operator == BinaryOp::Lt {
                                            if let Expression::Identifier(ci) = &*bin.left {
                                                if ci.name == lv {
                                                    if let Expression::Literal(el) = &*bin.right {
                                                        if let LiteralValue::Int(ev) = el.value {
                                                            let is_inc = match update.as_ref() {
                                                                Expression::Assignment(ua) => {
                                                                    ua.operator
                                                                        == AssignmentOp::Assign
                                                                        && matches!(&*ua.target, Expression::Identifier(i) if i.name == lv)
                                                                        && matches!(&*ua.value, Expression::Binary(b) if b.operator == BinaryOp::Add
                                                                            && matches!(&*b.left, Expression::Identifier(i) if i.name == lv)
                                                                            && matches!(&*b.right, Expression::Literal(l) if l.value == LiteralValue::Int(1)))
                                                                }
                                                                _ => false,
                                                            };
                                                            if is_inc {
                                                                return Some(
                                                                    self.generate_counted_loop(
                                                                        fs, cb, ab, &lv, sv, ev,
                                                                    ),
                                                                );
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn generate_counted_loop(
        &mut self,
        fs: &ForStmt,
        cb: &mut BasicBlock,
        ab: &mut Vec<BasicBlock>,
        lv: &str,
        sv: i32,
        ev: i32,
    ) -> BasicBlock {
        let mut condb = self.create_block("for_cond");
        let mut bodyb = self.create_block("for_body");
        let mb = self.create_block("for_end");
        let cl = Operand::Label(condb.label.clone());
        let bl = Operand::Label(bodyb.label.clone());
        let ml = Operand::Label(mb.label.clone());
        self.break_labels.push(mb.label.clone());
        self.continue_labels.push(condb.label.clone());
        cb.add_instruction(IRInstruction::Move(
            Operand::Variable(lv.to_string()),
            Operand::IntLiteral(sv),
        ));
        cb.add_instruction(IRInstruction::Jump(cl.clone()));
        let ct = self.new_temp();
        condb.add_instruction(IRInstruction::CmpLt(
            ct.clone(),
            Operand::Variable(lv.to_string()),
            Operand::IntLiteral(ev),
        ));
        condb.add_instruction(IRInstruction::JumpIfNot(ct, ml.clone()));
        condb.add_instruction(IRInstruction::Jump(bl.clone()));
        ab.push(condb);
        self.generate_statement(&fs.body, &mut bodyb, ab);
        let it = self.new_temp();
        bodyb.add_instruction(IRInstruction::Add(
            it.clone(),
            Operand::Variable(lv.to_string()),
            Operand::IntLiteral(1),
        ));
        bodyb.add_instruction(IRInstruction::Move(Operand::Variable(lv.to_string()), it));
        bodyb.add_instruction(IRInstruction::Jump(cl));
        ab.push(bodyb);
        self.break_labels.pop();
        self.continue_labels.pop();
        mb
    }

    fn generate_literal(&self, lit: &Literal) -> Operand {
        match &lit.value {
            LiteralValue::Int(v) => Operand::IntLiteral(*v),
            LiteralValue::Float(v) => Operand::FloatLiteral(*v),
            LiteralValue::Bool(v) => Operand::BoolLiteral(*v),
            LiteralValue::String(v) => Operand::StringLiteral(v.clone()),
        }
    }

    fn generate_identifier(&mut self, ident: &IdentifierExpr) -> Operand {
        if let Some(t) = self.var_to_temp.get(&ident.name) {
            return Operand::Temporary(t.clone());
        }
        if let Some(s) = self.symbol_table.lookup(&ident.name) {
            if let crate::semantic::type_system::Type::Function { .. } = &s.typ {
                return Operand::Label(ident.name.clone());
            }
        }
        Operand::Variable(ident.name.clone())
    }

    fn generate_switch(
        &mut self,
        ss: &SwitchStmt,
        cb: &mut BasicBlock,
        ab: &mut Vec<BasicBlock>,
    ) -> BasicBlock {
        let ev = self.generate_expression(&ss.expression, cb, ab);
        let mb = self.create_block("switch_end");
        let ml = Operand::Label(mb.label.clone());
        let mut case_blocks: Vec<BasicBlock> = Vec::new();
        for case in ss.cases.iter() {
            let cr = self.new_temp();
            let lo = self.generate_literal(&case.value);
            cb.add_instruction(IRInstruction::CmpEq(cr.clone(), ev.clone(), lo));
            let mut caseb = self.create_block("case");
            let case_l = Operand::Label(caseb.label.clone());
            cb.add_instruction(IRInstruction::JumpIf(cr, case_l.clone()));
            self.generate_statement(&case.body, &mut caseb, ab);
            if !caseb.is_terminator() {
                caseb.add_instruction(IRInstruction::Jump(ml.clone()));
            }
            case_blocks.push(caseb);
        }
        if let Some(ds) = &ss.default {
            let mut db = self.create_block("default");
            let dl = Operand::Label(db.label.clone());
            cb.add_instruction(IRInstruction::Jump(dl.clone()));
            self.generate_statement(ds, &mut db, ab);
            if !db.is_terminator() {
                db.add_instruction(IRInstruction::Jump(ml.clone()));
            }
            ab.push(db);
        } else {
            cb.add_instruction(IRInstruction::Jump(ml.clone()));
        }
        for cb2 in case_blocks {
            ab.push(cb2);
        }
        mb
    }
}
