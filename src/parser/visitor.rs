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
    fn visit_literal(&mut self, literal: &Literal) -> T;
    fn visit_identifier(&mut self, identifier: &IdentifierExpr) -> T;
    fn visit_binary(&mut self, binary: &BinaryExpr) -> T;
    fn visit_unary(&mut self, unary: &UnaryExpr) -> T;
    fn visit_assignment(&mut self, assignment: &AssignmentExpr) -> T;
    fn visit_call(&mut self, call: &CallExpr) -> T;
    fn visit_struct_access(&mut self, access: &StructAccessExpr) -> T;
    fn visit_grouped(&mut self, grouped: &GroupedExpr) -> T;
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
    fn visit_literal(&mut self, literal: &mut Literal);
    fn visit_identifier(&mut self, identifier: &mut IdentifierExpr);
    fn visit_binary(&mut self, binary: &mut BinaryExpr);
    fn visit_unary(&mut self, unary: &mut UnaryExpr);
    fn visit_assignment(&mut self, assignment: &mut AssignmentExpr);
    fn visit_call(&mut self, call: &mut CallExpr);
    fn visit_struct_access(&mut self, access: &mut StructAccessExpr);
    fn visit_grouped(&mut self, grouped: &mut GroupedExpr);
}

/// Базовый Visitor, который ничего не делает (возвращает ())
pub struct DefaultVisitor;

impl Visitor<()> for DefaultVisitor {
    fn visit_program(&mut self, program: &Program) {
        for decl in &program.declarations {
            match decl {
                Declaration::Function(f) => {
                    self.visit_function_decl(f);
                }
                Declaration::Struct(s) => {
                    self.visit_struct_decl(s);
                }
                Declaration::Variable(v) => {
                    self.visit_var_decl(v);
                }
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
            match init.as_ref() {
                Expression::Literal(l) => {
                    self.visit_literal(l);
                }
                Expression::Identifier(i) => {
                    self.visit_identifier(i);
                }
                Expression::Binary(b) => {
                    self.visit_binary(b);
                }
                Expression::Unary(u) => {
                    self.visit_unary(u);
                }
                Expression::Assignment(a) => {
                    self.visit_assignment(a);
                }
                Expression::Call(c) => {
                    self.visit_call(c);
                }
                Expression::StructAccess(sa) => {
                    self.visit_struct_access(sa);
                }
                Expression::Grouped(g) => {
                    self.visit_grouped(g);
                }
            }
        }
    }

    fn visit_param(&mut self, _param: &Param) {}

    fn visit_block(&mut self, block: &BlockStmt) {
        for stmt in &block.statements {
            match stmt {
                Statement::VariableDecl(v) => {
                    self.visit_var_decl(v);
                }
                Statement::Expression(e) => {
                    self.visit_expr_stmt(e);
                }
                Statement::If(i) => {
                    self.visit_if_stmt(i);
                }
                Statement::While(w) => {
                    self.visit_while_stmt(w);
                }
                Statement::For(f) => {
                    self.visit_for_stmt(f);
                }
                Statement::Return(r) => {
                    self.visit_return_stmt(r);
                }
                Statement::Block(b) => {
                    self.visit_block(b);
                }
                Statement::Empty(e) => {
                    self.visit_empty_stmt(e);
                }
            }
        }
    }

    fn visit_if_stmt(&mut self, if_stmt: &IfStmt) {
        match if_stmt.condition.as_ref() {
            Expression::Literal(l) => {
                self.visit_literal(l);
            }
            Expression::Identifier(i) => {
                self.visit_identifier(i);
            }
            Expression::Binary(b) => {
                self.visit_binary(b);
            }
            Expression::Unary(u) => {
                self.visit_unary(u);
            }
            Expression::Assignment(a) => {
                self.visit_assignment(a);
            }
            Expression::Call(c) => {
                self.visit_call(c);
            }
            Expression::StructAccess(sa) => {
                self.visit_struct_access(sa);
            }
            Expression::Grouped(g) => {
                self.visit_grouped(g);
            }
        }
        match if_stmt.then_branch.as_ref() {
            Statement::VariableDecl(v) => {
                self.visit_var_decl(v);
            }
            Statement::Expression(e) => {
                self.visit_expr_stmt(e);
            }
            Statement::If(i) => {
                self.visit_if_stmt(i);
            }
            Statement::While(w) => {
                self.visit_while_stmt(w);
            }
            Statement::For(f) => {
                self.visit_for_stmt(f);
            }
            Statement::Return(r) => {
                self.visit_return_stmt(r);
            }
            Statement::Block(b) => {
                self.visit_block(b);
            }
            Statement::Empty(e) => {
                self.visit_empty_stmt(e);
            }
        }
        if let Some(else_branch) = &if_stmt.else_branch {
            match else_branch.as_ref() {
                Statement::VariableDecl(v) => {
                    self.visit_var_decl(v);
                }
                Statement::Expression(e) => {
                    self.visit_expr_stmt(e);
                }
                Statement::If(i) => {
                    self.visit_if_stmt(i);
                }
                Statement::While(w) => {
                    self.visit_while_stmt(w);
                }
                Statement::For(f) => {
                    self.visit_for_stmt(f);
                }
                Statement::Return(r) => {
                    self.visit_return_stmt(r);
                }
                Statement::Block(b) => {
                    self.visit_block(b);
                }
                Statement::Empty(e) => {
                    self.visit_empty_stmt(e);
                }
            }
        }
    }

    fn visit_while_stmt(&mut self, while_stmt: &WhileStmt) {
        match while_stmt.condition.as_ref() {
            Expression::Literal(l) => {
                self.visit_literal(l);
            }
            Expression::Identifier(i) => {
                self.visit_identifier(i);
            }
            Expression::Binary(b) => {
                self.visit_binary(b);
            }
            Expression::Unary(u) => {
                self.visit_unary(u);
            }
            Expression::Assignment(a) => {
                self.visit_assignment(a);
            }
            Expression::Call(c) => {
                self.visit_call(c);
            }
            Expression::StructAccess(sa) => {
                self.visit_struct_access(sa);
            }
            Expression::Grouped(g) => {
                self.visit_grouped(g);
            }
        }
        match while_stmt.body.as_ref() {
            Statement::VariableDecl(v) => {
                self.visit_var_decl(v);
            }
            Statement::Expression(e) => {
                self.visit_expr_stmt(e);
            }
            Statement::If(i) => {
                self.visit_if_stmt(i);
            }
            Statement::While(w) => {
                self.visit_while_stmt(w);
            }
            Statement::For(f) => {
                self.visit_for_stmt(f);
            }
            Statement::Return(r) => {
                self.visit_return_stmt(r);
            }
            Statement::Block(b) => {
                self.visit_block(b);
            }
            Statement::Empty(e) => {
                self.visit_empty_stmt(e);
            }
        }
    }

    fn visit_for_stmt(&mut self, for_stmt: &ForStmt) {
        if let Some(init) = &for_stmt.init {
            match init.as_ref() {
                Statement::VariableDecl(v) => {
                    self.visit_var_decl(v);
                }
                Statement::Expression(e) => {
                    self.visit_expr_stmt(e);
                }
                Statement::If(i) => {
                    self.visit_if_stmt(i);
                }
                Statement::While(w) => {
                    self.visit_while_stmt(w);
                }
                Statement::For(f) => {
                    self.visit_for_stmt(f);
                }
                Statement::Return(r) => {
                    self.visit_return_stmt(r);
                }
                Statement::Block(b) => {
                    self.visit_block(b);
                }
                Statement::Empty(e) => {
                    self.visit_empty_stmt(e);
                }
            }
        }
        if let Some(condition) = &for_stmt.condition {
            match condition.as_ref() {
                Expression::Literal(l) => {
                    self.visit_literal(l);
                }
                Expression::Identifier(i) => {
                    self.visit_identifier(i);
                }
                Expression::Binary(b) => {
                    self.visit_binary(b);
                }
                Expression::Unary(u) => {
                    self.visit_unary(u);
                }
                Expression::Assignment(a) => {
                    self.visit_assignment(a);
                }
                Expression::Call(c) => {
                    self.visit_call(c);
                }
                Expression::StructAccess(sa) => {
                    self.visit_struct_access(sa);
                }
                Expression::Grouped(g) => {
                    self.visit_grouped(g);
                }
            }
        }
        if let Some(update) = &for_stmt.update {
            match update.as_ref() {
                Expression::Literal(l) => {
                    self.visit_literal(l);
                }
                Expression::Identifier(i) => {
                    self.visit_identifier(i);
                }
                Expression::Binary(b) => {
                    self.visit_binary(b);
                }
                Expression::Unary(u) => {
                    self.visit_unary(u);
                }
                Expression::Assignment(a) => {
                    self.visit_assignment(a);
                }
                Expression::Call(c) => {
                    self.visit_call(c);
                }
                Expression::StructAccess(sa) => {
                    self.visit_struct_access(sa);
                }
                Expression::Grouped(g) => {
                    self.visit_grouped(g);
                }
            }
        }
        match for_stmt.body.as_ref() {
            Statement::VariableDecl(v) => {
                self.visit_var_decl(v);
            }
            Statement::Expression(e) => {
                self.visit_expr_stmt(e);
            }
            Statement::If(i) => {
                self.visit_if_stmt(i);
            }
            Statement::While(w) => {
                self.visit_while_stmt(w);
            }
            Statement::For(f) => {
                self.visit_for_stmt(f);
            }
            Statement::Return(r) => {
                self.visit_return_stmt(r);
            }
            Statement::Block(b) => {
                self.visit_block(b);
            }
            Statement::Empty(e) => {
                self.visit_empty_stmt(e);
            }
        }
    }

    fn visit_return_stmt(&mut self, return_stmt: &ReturnStmt) {
        if let Some(value) = &return_stmt.value {
            match value.as_ref() {
                Expression::Literal(l) => {
                    self.visit_literal(l);
                }
                Expression::Identifier(i) => {
                    self.visit_identifier(i);
                }
                Expression::Binary(b) => {
                    self.visit_binary(b);
                }
                Expression::Unary(u) => {
                    self.visit_unary(u);
                }
                Expression::Assignment(a) => {
                    self.visit_assignment(a);
                }
                Expression::Call(c) => {
                    self.visit_call(c);
                }
                Expression::StructAccess(sa) => {
                    self.visit_struct_access(sa);
                }
                Expression::Grouped(g) => {
                    self.visit_grouped(g);
                }
            }
        }
    }

    fn visit_expr_stmt(&mut self, expr_stmt: &ExprStmt) {
        match expr_stmt.expr.as_ref() {
            Expression::Literal(l) => {
                self.visit_literal(l);
            }
            Expression::Identifier(i) => {
                self.visit_identifier(i);
            }
            Expression::Binary(b) => {
                self.visit_binary(b);
            }
            Expression::Unary(u) => {
                self.visit_unary(u);
            }
            Expression::Assignment(a) => {
                self.visit_assignment(a);
            }
            Expression::Call(c) => {
                self.visit_call(c);
            }
            Expression::StructAccess(sa) => {
                self.visit_struct_access(sa);
            }
            Expression::Grouped(g) => {
                self.visit_grouped(g);
            }
        }
    }

    fn visit_empty_stmt(&mut self, _empty_stmt: &EmptyStmt) {}

    fn visit_literal(&mut self, _literal: &Literal) {}

    fn visit_identifier(&mut self, _identifier: &IdentifierExpr) {}

    fn visit_binary(&mut self, binary: &BinaryExpr) {
        match binary.left.as_ref() {
            Expression::Literal(l) => {
                self.visit_literal(l);
            }
            Expression::Identifier(i) => {
                self.visit_identifier(i);
            }
            Expression::Binary(b) => {
                self.visit_binary(b);
            }
            Expression::Unary(u) => {
                self.visit_unary(u);
            }
            Expression::Assignment(a) => {
                self.visit_assignment(a);
            }
            Expression::Call(c) => {
                self.visit_call(c);
            }
            Expression::StructAccess(sa) => {
                self.visit_struct_access(sa);
            }
            Expression::Grouped(g) => {
                self.visit_grouped(g);
            }
        }
        match binary.right.as_ref() {
            Expression::Literal(l) => {
                self.visit_literal(l);
            }
            Expression::Identifier(i) => {
                self.visit_identifier(i);
            }
            Expression::Binary(b) => {
                self.visit_binary(b);
            }
            Expression::Unary(u) => {
                self.visit_unary(u);
            }
            Expression::Assignment(a) => {
                self.visit_assignment(a);
            }
            Expression::Call(c) => {
                self.visit_call(c);
            }
            Expression::StructAccess(sa) => {
                self.visit_struct_access(sa);
            }
            Expression::Grouped(g) => {
                self.visit_grouped(g);
            }
        }
    }

    fn visit_unary(&mut self, unary: &UnaryExpr) {
        match unary.operand.as_ref() {
            Expression::Literal(l) => {
                self.visit_literal(l);
            }
            Expression::Identifier(i) => {
                self.visit_identifier(i);
            }
            Expression::Binary(b) => {
                self.visit_binary(b);
            }
            Expression::Unary(u) => {
                self.visit_unary(u);
            }
            Expression::Assignment(a) => {
                self.visit_assignment(a);
            }
            Expression::Call(c) => {
                self.visit_call(c);
            }
            Expression::StructAccess(sa) => {
                self.visit_struct_access(sa);
            }
            Expression::Grouped(g) => {
                self.visit_grouped(g);
            }
        }
    }

    fn visit_assignment(&mut self, assignment: &AssignmentExpr) {
        match assignment.target.as_ref() {
            Expression::Literal(l) => {
                self.visit_literal(l);
            }
            Expression::Identifier(i) => {
                self.visit_identifier(i);
            }
            Expression::Binary(b) => {
                self.visit_binary(b);
            }
            Expression::Unary(u) => {
                self.visit_unary(u);
            }
            Expression::Assignment(a) => {
                self.visit_assignment(a);
            }
            Expression::Call(c) => {
                self.visit_call(c);
            }
            Expression::StructAccess(sa) => {
                self.visit_struct_access(sa);
            }
            Expression::Grouped(g) => {
                self.visit_grouped(g);
            }
        }
        match assignment.value.as_ref() {
            Expression::Literal(l) => {
                self.visit_literal(l);
            }
            Expression::Identifier(i) => {
                self.visit_identifier(i);
            }
            Expression::Binary(b) => {
                self.visit_binary(b);
            }
            Expression::Unary(u) => {
                self.visit_unary(u);
            }
            Expression::Assignment(a) => {
                self.visit_assignment(a);
            }
            Expression::Call(c) => {
                self.visit_call(c);
            }
            Expression::StructAccess(sa) => {
                self.visit_struct_access(sa);
            }
            Expression::Grouped(g) => {
                self.visit_grouped(g);
            }
        }
    }

    fn visit_call(&mut self, call: &CallExpr) {
        match call.callee.as_ref() {
            Expression::Literal(l) => {
                self.visit_literal(l);
            }
            Expression::Identifier(i) => {
                self.visit_identifier(i);
            }
            Expression::Binary(b) => {
                self.visit_binary(b);
            }
            Expression::Unary(u) => {
                self.visit_unary(u);
            }
            Expression::Assignment(a) => {
                self.visit_assignment(a);
            }
            Expression::Call(c) => {
                self.visit_call(c);
            }
            Expression::StructAccess(sa) => {
                self.visit_struct_access(sa);
            }
            Expression::Grouped(g) => {
                self.visit_grouped(g);
            }
        }
        for arg in &call.arguments {
            match arg {
                Expression::Literal(l) => {
                    self.visit_literal(l);
                }
                Expression::Identifier(i) => {
                    self.visit_identifier(i);
                }
                Expression::Binary(b) => {
                    self.visit_binary(b);
                }
                Expression::Unary(u) => {
                    self.visit_unary(u);
                }
                Expression::Assignment(a) => {
                    self.visit_assignment(a);
                }
                Expression::Call(c) => {
                    self.visit_call(c);
                }
                Expression::StructAccess(sa) => {
                    self.visit_struct_access(sa);
                }
                Expression::Grouped(g) => {
                    self.visit_grouped(g);
                }
            }
        }
    }

    fn visit_struct_access(&mut self, access: &StructAccessExpr) {
        match access.object.as_ref() {
            Expression::Literal(l) => {
                self.visit_literal(l);
            }
            Expression::Identifier(i) => {
                self.visit_identifier(i);
            }
            Expression::Binary(b) => {
                self.visit_binary(b);
            }
            Expression::Unary(u) => {
                self.visit_unary(u);
            }
            Expression::Assignment(a) => {
                self.visit_assignment(a);
            }
            Expression::Call(c) => {
                self.visit_call(c);
            }
            Expression::StructAccess(sa) => {
                self.visit_struct_access(sa);
            }
            Expression::Grouped(g) => {
                self.visit_grouped(g);
            }
        }
    }

    fn visit_grouped(&mut self, grouped: &GroupedExpr) {
        match grouped.expr.as_ref() {
            Expression::Literal(l) => {
                self.visit_literal(l);
            }
            Expression::Identifier(i) => {
                self.visit_identifier(i);
            }
            Expression::Binary(b) => {
                self.visit_binary(b);
            }
            Expression::Unary(u) => {
                self.visit_unary(u);
            }
            Expression::Assignment(a) => {
                self.visit_assignment(a);
            }
            Expression::Call(c) => {
                self.visit_call(c);
            }
            Expression::StructAccess(sa) => {
                self.visit_struct_access(sa);
            }
            Expression::Grouped(g) => {
                self.visit_grouped(g);
            }
        }
    }
}
