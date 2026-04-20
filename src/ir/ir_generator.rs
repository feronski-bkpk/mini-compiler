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
        }
    }

    fn new_temp(&mut self) -> Operand {
        self.temp_counter += 1;
        Operand::Temporary(format!("t{}", self.temp_counter))
    }

    fn new_label(&mut self) -> Operand {
        self.label_counter += 1;
        let func_prefix = if let Some(name) = &self.current_function {
            format!("{}_", name)
        } else {
            String::new()
        };
        Operand::Label(format!("{}{}", func_prefix, self.label_counter))
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

        let entry_label = self.new_label();
        let entry_label_str = match &entry_label {
            Operand::Label(l) => l.clone(),
            _ => unreachable!(),
        };

        let mut entry_block = BasicBlock::new(entry_label_str.clone());
        func_ir.set_entry(entry_label_str.clone());

        let mut all_blocks: Vec<BasicBlock> = Vec::new();
        let mut current_block = entry_block.clone();

        self.generate_blocks(
            &func.body,
            &mut entry_block,
            &mut current_block,
            &mut all_blocks,
        );

        for (name, typ) in &self.current_locals {
            func_ir.locals.push((name.clone(), typ.clone()));
        }

        for block in all_blocks {
            func_ir.add_block(block);
        }
        func_ir.add_block(entry_block);

        if !current_block.instructions.is_empty() {
            if current_block.label == entry_label_str {
                let new_label = self.new_label();
                let new_label_str = match &new_label {
                    Operand::Label(l) => l.clone(),
                    _ => unreachable!(),
                };
                let mut new_block = BasicBlock::new(new_label_str);
                new_block.instructions = current_block.instructions;
                new_block.predecessors = current_block.predecessors;
                new_block.successors = current_block.successors;
                func_ir.add_block(new_block);
            } else {
                func_ir.add_block(current_block);
            }
        }

        super::control_flow::build_cfg(&mut func_ir);
        self.program.add_function(func_ir);
        self.current_function = None;
    }

    fn generate_blocks(
        &mut self,
        block: &BlockStmt,
        entry_block: &mut BasicBlock,
        current_block: &mut BasicBlock,
        all_blocks: &mut Vec<BasicBlock>,
    ) {
        for stmt in &block.statements {
            self.generate_statement(stmt, entry_block, current_block, all_blocks);
        }
    }

    fn generate_statement(
        &mut self,
        stmt: &Statement,
        entry_block: &mut BasicBlock,
        current_block: &mut BasicBlock,
        all_blocks: &mut Vec<BasicBlock>,
    ) {
        match stmt {
            Statement::VariableDecl(var) => {
                self.current_locals
                    .push((var.name.clone(), var.var_type.to_string()));
                if let Some(init) = &var.initializer {
                    let value = self.generate_expression(init, entry_block, all_blocks);
                    let temp = self.new_temp();
                    entry_block.add_instruction(IRInstruction::Move(temp.clone(), value.clone()));
                    entry_block.add_instruction(IRInstruction::Move(
                        Operand::Variable(var.name.clone()),
                        temp,
                    ));
                    if let Operand::Temporary(t) = value {
                        self.var_to_temp.insert(var.name.clone(), t);
                    }
                }
            }
            Statement::Expression(expr_stmt) => {
                self.generate_expression(&expr_stmt.expr, current_block, all_blocks);
            }
            Statement::If(if_stmt) => {
                let end_block = self.generate_if(if_stmt, entry_block, all_blocks);
                *current_block = end_block;
            }
            Statement::While(while_stmt) => {
                let end_block = self.generate_while(while_stmt, current_block, all_blocks);
                *current_block = end_block;
            }
            Statement::For(for_stmt) => {
                let end_block = self.generate_for(for_stmt, current_block, all_blocks);
                *current_block = end_block;
            }
            Statement::Return(return_stmt) => {
                if let Some(value) = &return_stmt.value {
                    let val = self.generate_expression(value, current_block, all_blocks);
                    current_block.add_instruction(IRInstruction::Return(Some(val)));
                } else {
                    current_block.add_instruction(IRInstruction::Return(None));
                }
            }
            Statement::Block(block_stmt) => {
                self.generate_blocks(block_stmt, entry_block, current_block, all_blocks);
            }
            Statement::Empty(_) => {}
        }
    }

    fn generate_expression(
        &mut self,
        expr: &Expression,
        current_block: &mut BasicBlock,
        all_blocks: &mut Vec<BasicBlock>,
    ) -> Operand {
        match expr {
            Expression::Literal(lit) => self.generate_literal(lit),
            Expression::Identifier(ident) => self.generate_identifier(ident),
            Expression::Binary(binary) => {
                let left = self.generate_expression(&binary.left, current_block, all_blocks);
                let right = self.generate_expression(&binary.right, current_block, all_blocks);
                let dest = self.new_temp();
                let instr = match binary.operator {
                    BinaryOp::Add => IRInstruction::Add(dest.clone(), left, right),
                    BinaryOp::Sub => IRInstruction::Sub(dest.clone(), left, right),
                    BinaryOp::Mul => IRInstruction::Mul(dest.clone(), left, right),
                    BinaryOp::Div => IRInstruction::Div(dest.clone(), left, right),
                    BinaryOp::Mod => IRInstruction::Mod(dest.clone(), left, right),
                    BinaryOp::Eq => IRInstruction::CmpEq(dest.clone(), left, right),
                    BinaryOp::Ne => IRInstruction::CmpNe(dest.clone(), left, right),
                    BinaryOp::Lt => IRInstruction::CmpLt(dest.clone(), left, right),
                    BinaryOp::Le => IRInstruction::CmpLe(dest.clone(), left, right),
                    BinaryOp::Gt => IRInstruction::CmpGt(dest.clone(), left, right),
                    BinaryOp::Ge => IRInstruction::CmpGe(dest.clone(), left, right),
                    BinaryOp::And => IRInstruction::And(dest.clone(), left, right),
                    BinaryOp::Or => IRInstruction::Or(dest.clone(), left, right),
                };
                current_block.add_instruction(instr);
                dest
            }
            Expression::Unary(unary) => {
                let operand = self.generate_expression(&unary.operand, current_block, all_blocks);
                let dest = self.new_temp();
                match unary.operator {
                    UnaryOp::Neg => {
                        current_block.add_instruction(IRInstruction::Neg(dest.clone(), operand))
                    }
                    UnaryOp::Not => {
                        current_block.add_instruction(IRInstruction::Not(dest.clone(), operand))
                    }
                    UnaryOp::Plus => {
                        current_block.add_instruction(IRInstruction::Move(dest.clone(), operand))
                    }
                    UnaryOp::PreIncrement => {
                        let one = Operand::IntLiteral(1);
                        current_block.add_instruction(IRInstruction::Add(
                            dest.clone(),
                            operand.clone(),
                            one,
                        ));
                        current_block.add_instruction(IRInstruction::Move(operand, dest.clone()));
                    }
                    UnaryOp::PostIncrement => {
                        let temp = self.new_temp();
                        let one = Operand::IntLiteral(1);
                        current_block
                            .add_instruction(IRInstruction::Move(temp.clone(), operand.clone()));
                        current_block.add_instruction(IRInstruction::Add(
                            dest.clone(),
                            operand.clone(),
                            one,
                        ));
                        current_block.add_instruction(IRInstruction::Move(operand, dest.clone()));
                        return temp;
                    }
                    UnaryOp::PreDecrement => {
                        let one = Operand::IntLiteral(1);
                        current_block.add_instruction(IRInstruction::Sub(
                            dest.clone(),
                            operand.clone(),
                            one,
                        ));
                        current_block.add_instruction(IRInstruction::Move(operand, dest.clone()));
                    }
                    UnaryOp::PostDecrement => {
                        let temp = self.new_temp();
                        let one = Operand::IntLiteral(1);
                        current_block
                            .add_instruction(IRInstruction::Move(temp.clone(), operand.clone()));
                        current_block.add_instruction(IRInstruction::Sub(
                            dest.clone(),
                            operand.clone(),
                            one,
                        ));
                        current_block.add_instruction(IRInstruction::Move(operand, dest.clone()));
                        return temp;
                    }
                }
                dest
            }
            Expression::Assignment(assign) => {
                let value = self.generate_expression(&assign.value, current_block, all_blocks);
                let target = self.generate_expression(&assign.target, current_block, all_blocks);
                if matches!(assign.operator, AssignmentOp::Assign) {
                    current_block.add_instruction(IRInstruction::Move(target, value.clone()));
                }
                value
            }
            Expression::Call(call) => {
                let func_name = match &*call.callee {
                    Expression::Identifier(ident) => ident.name.clone(),
                    _ => return self.new_temp(),
                };
                let mut args = Vec::new();
                for (i, arg) in call.arguments.iter().enumerate() {
                    let arg_val = self.generate_expression(arg, current_block, all_blocks);
                    current_block.add_instruction(IRInstruction::Param(i as u32, arg_val.clone()));
                    args.push(arg_val);
                }
                let dest = self.new_temp();
                current_block.add_instruction(IRInstruction::Call(
                    dest.clone(),
                    Operand::Label(func_name),
                    args,
                ));
                dest
            }
            Expression::StructAccess(access) => {
                self.generate_expression(&access.object, current_block, all_blocks)
            }
            Expression::Grouped(grouped) => {
                self.generate_expression(&grouped.expr, current_block, all_blocks)
            }
        }
    }

    fn generate_if(
        &mut self,
        if_stmt: &IfStmt,
        entry_block: &mut BasicBlock,
        all_blocks: &mut Vec<BasicBlock>,
    ) -> BasicBlock {
        let cond = self.generate_expression(&if_stmt.condition, entry_block, all_blocks);

        let then_label = self.new_label();
        let else_label = self.new_label();
        let end_label = self.new_label();

        let then_label_str = match &then_label {
            Operand::Label(l) => l.clone(),
            _ => unreachable!(),
        };
        let else_label_str = match &else_label {
            Operand::Label(l) => l.clone(),
            _ => unreachable!(),
        };
        let end_label_str = match &end_label {
            Operand::Label(l) => l.clone(),
            _ => unreachable!(),
        };

        entry_block.add_instruction(IRInstruction::JumpIf(cond, then_label.clone()));
        entry_block.add_instruction(IRInstruction::Jump(else_label.clone()));

        let mut then_block = BasicBlock::new(then_label_str.clone());
        if let Statement::Block(block_stmt) = &*if_stmt.then_branch {
            let mut temp_block = then_block.clone();
            self.generate_block_content(block_stmt, &mut temp_block, all_blocks);
            then_block = temp_block;
        } else {
            self.generate_statement_content(&if_stmt.then_branch, &mut then_block, all_blocks);
        }
        if !then_block.is_terminator() {
            then_block.add_instruction(IRInstruction::Jump(end_label.clone()));
        }
        all_blocks.push(then_block);

        let mut else_block = BasicBlock::new(else_label_str.clone());
        if let Some(else_branch) = &if_stmt.else_branch {
            if let Statement::Block(block_stmt) = &**else_branch {
                let mut temp_block = else_block.clone();
                self.generate_block_content(block_stmt, &mut temp_block, all_blocks);
                else_block = temp_block;
            } else {
                self.generate_statement_content(else_branch, &mut else_block, all_blocks);
            }
        }
        if !else_block.is_terminator() {
            else_block.add_instruction(IRInstruction::Jump(end_label.clone()));
        }
        all_blocks.push(else_block);

        entry_block.add_successor(then_label_str);
        entry_block.add_successor(else_label_str);

        let end_block = BasicBlock::new(end_label_str);
        all_blocks.push(end_block.clone());

        end_block
    }

    fn generate_block_content(
        &mut self,
        block: &BlockStmt,
        current_block: &mut BasicBlock,
        all_blocks: &mut Vec<BasicBlock>,
    ) {
        for stmt in &block.statements {
            self.generate_statement_content(stmt, current_block, all_blocks);
        }
    }

    fn generate_statement_content(
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
                    let temp = self.new_temp();
                    current_block.add_instruction(IRInstruction::Move(temp.clone(), value.clone()));
                    current_block.add_instruction(IRInstruction::Move(
                        Operand::Variable(var.name.clone()),
                        temp,
                    ));
                    if let Operand::Temporary(t) = value {
                        self.var_to_temp.insert(var.name.clone(), t);
                    }
                }
            }
            Statement::Expression(expr_stmt) => {
                self.generate_expression(&expr_stmt.expr, current_block, all_blocks);
            }
            Statement::If(if_stmt) => {
                let mut dummy_entry = current_block.clone();
                let end_block = self.generate_if(if_stmt, &mut dummy_entry, all_blocks);
                *current_block = end_block;
            }
            Statement::While(while_stmt) => {
                let end_block = self.generate_while(while_stmt, current_block, all_blocks);
                *current_block = end_block;
            }
            Statement::For(for_stmt) => {
                let end_block = self.generate_for(for_stmt, current_block, all_blocks);
                *current_block = end_block;
            }
            Statement::Return(return_stmt) => {
                if let Some(value) = &return_stmt.value {
                    let val = self.generate_expression(value, current_block, all_blocks);
                    current_block.add_instruction(IRInstruction::Return(Some(val)));
                } else {
                    current_block.add_instruction(IRInstruction::Return(None));
                }
            }
            Statement::Block(block_stmt) => {
                self.generate_block_content(block_stmt, current_block, all_blocks);
            }
            Statement::Empty(_) => {}
        }
    }

    fn generate_while(
        &mut self,
        while_stmt: &WhileStmt,
        current_block: &mut BasicBlock,
        all_blocks: &mut Vec<BasicBlock>,
    ) -> BasicBlock {
        let cond_label = self.new_label();
        let body_label = self.new_label();
        let end_label = self.new_label();

        let cond_label_str = match &cond_label {
            Operand::Label(l) => l.clone(),
            _ => unreachable!(),
        };
        let body_label_str = match &body_label {
            Operand::Label(l) => l.clone(),
            _ => unreachable!(),
        };
        let end_label_str = match &end_label {
            Operand::Label(l) => l.clone(),
            _ => unreachable!(),
        };

        current_block.add_instruction(IRInstruction::Jump(cond_label.clone()));

        let mut cond_block = BasicBlock::new(cond_label_str);
        let cond = self.generate_expression(&while_stmt.condition, &mut cond_block, all_blocks);
        cond_block.add_instruction(IRInstruction::JumpIfNot(cond, end_label.clone()));
        cond_block.add_instruction(IRInstruction::Jump(body_label.clone()));
        all_blocks.push(cond_block);

        let mut body_block = BasicBlock::new(body_label_str);
        if let Statement::Block(block_stmt) = &*while_stmt.body {
            self.generate_blocks(block_stmt, current_block, &mut body_block, all_blocks);
        } else {
            let mut dummy_entry = body_block.clone();
            self.generate_statement(
                &while_stmt.body,
                &mut dummy_entry,
                &mut body_block,
                all_blocks,
            );
        }
        body_block.add_instruction(IRInstruction::Jump(cond_label.clone()));
        all_blocks.push(body_block);

        let end_block = BasicBlock::new(end_label_str);
        all_blocks.push(end_block.clone());

        end_block
    }

    fn generate_for(
        &mut self,
        for_stmt: &ForStmt,
        current_block: &mut BasicBlock,
        all_blocks: &mut Vec<BasicBlock>,
    ) -> BasicBlock {
        if let Some(init) = &for_stmt.init {
            let mut dummy = current_block.clone();
            self.generate_statement(init, current_block, &mut dummy, all_blocks);
            *current_block = dummy;
        }

        let cond_label = self.new_label();
        let body_label = self.new_label();
        let update_label = self.new_label();
        let end_label = self.new_label();

        let cond_label_str = match &cond_label {
            Operand::Label(l) => l.clone(),
            _ => unreachable!(),
        };
        let body_label_str = match &body_label {
            Operand::Label(l) => l.clone(),
            _ => unreachable!(),
        };
        let update_label_str = match &update_label {
            Operand::Label(l) => l.clone(),
            _ => unreachable!(),
        };
        let end_label_str = match &end_label {
            Operand::Label(l) => l.clone(),
            _ => unreachable!(),
        };

        current_block.add_instruction(IRInstruction::Jump(cond_label.clone()));

        let mut cond_block = BasicBlock::new(cond_label_str);
        if let Some(condition) = &for_stmt.condition {
            let cond = self.generate_expression(condition, &mut cond_block, all_blocks);
            cond_block.add_instruction(IRInstruction::JumpIfNot(cond, end_label.clone()));
            cond_block.add_instruction(IRInstruction::Jump(body_label.clone()));
        } else {
            cond_block.add_instruction(IRInstruction::Jump(body_label.clone()));
        }
        all_blocks.push(cond_block);

        let mut body_block = BasicBlock::new(body_label_str);
        if let Statement::Block(block_stmt) = &*for_stmt.body {
            self.generate_blocks(block_stmt, current_block, &mut body_block, all_blocks);
        } else {
            self.generate_statement(&for_stmt.body, current_block, &mut body_block, all_blocks);
        }
        body_block.add_instruction(IRInstruction::Jump(update_label.clone()));
        all_blocks.push(body_block);

        let mut update_block = BasicBlock::new(update_label_str);
        if let Some(update) = &for_stmt.update {
            self.generate_expression(update, &mut update_block, all_blocks);
        }
        update_block.add_instruction(IRInstruction::Jump(cond_label.clone()));
        all_blocks.push(update_block);

        let end_block = BasicBlock::new(end_label_str);
        all_blocks.push(end_block.clone());

        end_block
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
        if let Some(temp) = self.var_to_temp.get(&ident.name) {
            return Operand::Temporary(temp.clone());
        }

        if let Some(symbol) = self.symbol_table.lookup(&ident.name) {
            if let crate::semantic::type_system::Type::Function { .. } = &symbol.typ {
                return Operand::Label(ident.name.clone());
            }
        }

        Operand::Variable(ident.name.clone())
    }
}
