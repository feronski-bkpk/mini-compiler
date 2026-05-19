//! Паттерн Visitor для обхода и анализа AST

use crate::parser::ast::*;

/// Трейт для Visitor, который обходит AST и возвращает результат
pub trait Visitor<T> {
    fn visit_program(&mut self, program: &Program) -> T;
    fn visit_function_decl(&mut self, func: &FunctionDecl) -> T;
    fn visit_struct_decl(&mut self, struct_decl: &StructDecl) -> T;
    fn visit_var_decl(&mut self, var_decl: &VarDecl) -> T;
    fn visit_param(&mut self, param: &Param) -> T;
    fn visit_block(&mut self, block: &BlockStmt) -> T;
    fn visit_if_stmt(&mut self, if_stmt: &IfStmt) -> T;
    fn visit_while_stmt(&mut self, while_stmt: &WhileStmt) -> T;
    fn visit_for_stmt(&mut self, for_stmt: &ForStmt) -> T;
    fn visit_return_stmt(&mut self, return_stmt: &ReturnStmt) -> T;
    fn visit_expr_stmt(&mut self, expr_stmt: &ExprStmt) -> T;
    fn visit_empty_stmt(&mut self, empty_stmt: &EmptyStmt) -> T;
    fn visit_break_stmt(&mut self, break_stmt: &BreakStmt) -> T;
    fn visit_continue_stmt(&mut self, continue_stmt: &ContinueStmt) -> T;
    fn visit_switch_stmt(&mut self, switch_stmt: &SwitchStmt) -> T;
    fn visit_case_stmt(&mut self, case_stmt: &CaseStmt) -> T;
    fn visit_literal(&mut self, literal: &Literal) -> T;
    fn visit_identifier(&mut self, identifier: &IdentifierExpr) -> T;
    fn visit_binary(&mut self, binary: &BinaryExpr) -> T;
    fn visit_unary(&mut self, unary: &UnaryExpr) -> T;
    fn visit_assignment(&mut self, assignment: &AssignmentExpr) -> T;
    fn visit_call(&mut self, call: &CallExpr) -> T;
    fn visit_struct_access(&mut self, access: &StructAccessExpr) -> T;
    fn visit_grouped(&mut self, grouped: &GroupedExpr) -> T;
    fn visit_array_access(&mut self, access: &ArrayAccessExpr) -> T;
}

/// Трейт для Visitor, который модифицирует AST
pub trait VisitorMut {
    fn visit_program(&mut self, program: &mut Program);
    fn visit_function_decl(&mut self, func: &mut FunctionDecl);
    fn visit_struct_decl(&mut self, struct_decl: &mut StructDecl);
    fn visit_var_decl(&mut self, var_decl: &mut VarDecl);
    fn visit_param(&mut self, param: &mut Param);
    fn visit_block(&mut self, block: &mut BlockStmt);
    fn visit_if_stmt(&mut self, if_stmt: &mut IfStmt);
    fn visit_while_stmt(&mut self, while_stmt: &mut WhileStmt);
    fn visit_for_stmt(&mut self, for_stmt: &mut ForStmt);
    fn visit_return_stmt(&mut self, return_stmt: &mut ReturnStmt);
    fn visit_expr_stmt(&mut self, expr_stmt: &mut ExprStmt);
    fn visit_empty_stmt(&mut self, empty_stmt: &mut EmptyStmt);
    fn visit_break_stmt(&mut self, break_stmt: &mut BreakStmt);
    fn visit_continue_stmt(&mut self, continue_stmt: &mut ContinueStmt);
    fn visit_switch_stmt(&mut self, switch_stmt: &mut SwitchStmt);
    fn visit_case_stmt(&mut self, case_stmt: &mut CaseStmt);
    fn visit_literal(&mut self, literal: &mut Literal);
    fn visit_identifier(&mut self, identifier: &mut IdentifierExpr);
    fn visit_binary(&mut self, binary: &mut BinaryExpr);
    fn visit_unary(&mut self, unary: &mut UnaryExpr);
    fn visit_assignment(&mut self, assignment: &mut AssignmentExpr);
    fn visit_call(&mut self, call: &mut CallExpr);
    fn visit_struct_access(&mut self, access: &mut StructAccessExpr);
    fn visit_grouped(&mut self, grouped: &mut GroupedExpr);
    fn visit_array_access(&mut self, access: &mut ArrayAccessExpr);
}

/// Базовый Visitor, который ничего не делает (возвращает ())
pub struct DefaultVisitor;

impl DefaultVisitor {
    /// Вспомогательный метод для обхода выражений
    fn visit_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Identifier(i) => self.visit_identifier(i),
            Expression::Binary(b) => self.visit_binary(b),
            Expression::Unary(u) => self.visit_unary(u),
            Expression::Assignment(a) => self.visit_assignment(a),
            Expression::Call(c) => self.visit_call(c),
            Expression::StructAccess(s) => self.visit_struct_access(s),
            Expression::Grouped(g) => self.visit_grouped(g),
            Expression::ArrayAccess(a) => self.visit_array_access(a),
            Expression::ArrayInitializer(arr) => {
                for elem in &arr.elements {
                    self.visit_expression(elem);
                }
            }
        }
    }

    /// Вспомогательный метод для обхода инструкций
    fn visit_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::VariableDecl(v) => self.visit_var_decl(v),
            Statement::Expression(e) => self.visit_expr_stmt(e),
            Statement::If(i) => self.visit_if_stmt(i),
            Statement::While(w) => self.visit_while_stmt(w),
            Statement::For(f) => self.visit_for_stmt(f),
            Statement::Return(r) => self.visit_return_stmt(r),
            Statement::Block(b) => self.visit_block(b),
            Statement::Empty(e) => self.visit_empty_stmt(e),
            Statement::Break(b) => self.visit_break_stmt(b),
            Statement::Continue(c) => self.visit_continue_stmt(c),
            Statement::Switch(s) => self.visit_switch_stmt(s),
        }
    }
}

impl Visitor<()> for DefaultVisitor {
    fn visit_program(&mut self, program: &Program) {
        for decl in &program.declarations {
            match decl {
                Declaration::Function(f) => self.visit_function_decl(f),
                Declaration::ExternFunction(ext) => {
                    let func = ext.to_function_decl();
                    self.visit_function_decl(&func);
                }
                Declaration::Struct(s) => self.visit_struct_decl(s),
                Declaration::Variable(v) => self.visit_var_decl(v),
            }
        }
    }

    fn visit_function_decl(&mut self, func: &FunctionDecl) {
        for param in &func.parameters {
            self.visit_param(param);
        }
        self.visit_block(&func.body);
    }

    fn visit_struct_decl(&mut self, struct_decl: &StructDecl) {
        for field in &struct_decl.fields {
            self.visit_var_decl(field);
        }
    }

    fn visit_var_decl(&mut self, var_decl: &VarDecl) {
        if let Some(init) = &var_decl.initializer {
            self.visit_expression(init);
        }
    }

    fn visit_param(&mut self, _param: &Param) {}

    fn visit_block(&mut self, block: &BlockStmt) {
        for stmt in &block.statements {
            self.visit_statement(stmt);
        }
    }

    fn visit_if_stmt(&mut self, if_stmt: &IfStmt) {
        self.visit_expression(&if_stmt.condition);
        self.visit_statement(&if_stmt.then_branch);
        if let Some(else_branch) = &if_stmt.else_branch {
            self.visit_statement(else_branch);
        }
    }

    fn visit_switch_stmt(&mut self, switch_stmt: &SwitchStmt) {
        self.visit_expression(&switch_stmt.expression);
        for case in &switch_stmt.cases {
            self.visit_case_stmt(case);
        }
        if let Some(default) = &switch_stmt.default {
            self.visit_statement(default);
        }
    }

    fn visit_case_stmt(&mut self, case_stmt: &CaseStmt) {
        self.visit_literal(&case_stmt.value);
        self.visit_statement(&case_stmt.body);
    }

    fn visit_while_stmt(&mut self, while_stmt: &WhileStmt) {
        self.visit_expression(&while_stmt.condition);
        self.visit_statement(&while_stmt.body);
    }

    fn visit_for_stmt(&mut self, for_stmt: &ForStmt) {
        if let Some(init) = &for_stmt.init {
            self.visit_statement(init);
        }
        if let Some(condition) = &for_stmt.condition {
            self.visit_expression(condition);
        }
        if let Some(update) = &for_stmt.update {
            self.visit_expression(update);
        }
        self.visit_statement(&for_stmt.body);
    }

    fn visit_return_stmt(&mut self, return_stmt: &ReturnStmt) {
        if let Some(value) = &return_stmt.value {
            self.visit_expression(value);
        }
    }

    fn visit_expr_stmt(&mut self, expr_stmt: &ExprStmt) {
        self.visit_expression(&expr_stmt.expr);
    }

    fn visit_empty_stmt(&mut self, _empty_stmt: &EmptyStmt) {}
    fn visit_break_stmt(&mut self, _break_stmt: &BreakStmt) {}
    fn visit_continue_stmt(&mut self, _continue_stmt: &ContinueStmt) {}
    fn visit_literal(&mut self, _literal: &Literal) {}
    fn visit_identifier(&mut self, _identifier: &IdentifierExpr) {}

    fn visit_binary(&mut self, binary: &BinaryExpr) {
        self.visit_expression(&binary.left);
        self.visit_expression(&binary.right);
    }

    fn visit_unary(&mut self, unary: &UnaryExpr) {
        self.visit_expression(&unary.operand);
    }

    fn visit_assignment(&mut self, assignment: &AssignmentExpr) {
        self.visit_expression(&assignment.target);
        self.visit_expression(&assignment.value);
    }

    fn visit_call(&mut self, call: &CallExpr) {
        self.visit_expression(&call.callee);
        for arg in &call.arguments {
            self.visit_expression(arg);
        }
    }

    fn visit_struct_access(&mut self, access: &StructAccessExpr) {
        self.visit_expression(&access.object);
    }

    fn visit_grouped(&mut self, grouped: &GroupedExpr) {
        self.visit_expression(&grouped.expr);
    }

    fn visit_array_access(&mut self, aa: &ArrayAccessExpr) {
        self.visit_expression(&aa.array);
        self.visit_expression(&aa.index);
    }
}
