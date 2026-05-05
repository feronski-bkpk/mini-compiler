//! Генератор JSON для AST
//!
//! Создает машиночитаемое представление AST в формате JSON.

use crate::parser::ast::*;
use crate::parser::visitor::Visitor;
use serde_json::{Value, json};

/// Генератор JSON для AST
pub struct JsonGenerator;

impl JsonGenerator {
    /// Создает новый генератор JSON
    pub fn new() -> Self {
        Self
    }

    /// Генерирует JSON для программы
    pub fn generate(&mut self, program: &Program) -> Value {
        self.visit_program(program)
    }

    /// Генерирует JSON строку для программы
    pub fn to_string_pretty(&mut self, program: &Program) -> String {
        let value = self.generate(program);
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string())
    }

    /// Генерирует компактную JSON строку
    pub fn to_string_compact(&mut self, program: &Program) -> String {
        let value = self.generate(program);
        serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string())
    }
}

impl Visitor<Value> for JsonGenerator {
    fn visit_program(&mut self, program: &Program) -> Value {
        let mut declarations = Vec::new();
        for decl in &program.declarations {
            declarations.push(match decl {
                Declaration::Function(f) => self.visit_function_decl(f),
                Declaration::Struct(s) => self.visit_struct_decl(s),
                Declaration::Variable(v) => self.visit_var_decl(v),
            });
        }
        json!({ "type": "Program", "line": program.node.line, "column": program.node.column, "declarations": declarations })
    }

    fn visit_function_decl(&mut self, func: &FunctionDecl) -> Value {
        let mut params = Vec::new();
        for param in &func.parameters {
            params.push(self.visit_param(param));
        }
        json!({ "type": "FunctionDecl", "line": func.node.line, "column": func.node.column, "name": func.name, "return_type": func.return_type.to_string(), "parameters": params, "body": self.visit_block(&func.body) })
    }

    fn visit_struct_decl(&mut self, struct_decl: &StructDecl) -> Value {
        let mut fields = Vec::new();
        for field in &struct_decl.fields {
            fields.push(self.visit_var_decl(field));
        }
        json!({ "type": "StructDecl", "line": struct_decl.node.line, "column": struct_decl.node.column, "name": struct_decl.name, "fields": fields })
    }

    fn visit_var_decl(&mut self, var_decl: &VarDecl) -> Value {
        json!({ "type": "VarDecl", "line": var_decl.node.line, "column": var_decl.node.column, "var_type": var_decl.var_type.to_string(), "name": var_decl.name, "initializer": var_decl.initializer.as_ref().map(|init| self.visit_expression(init)) })
    }

    fn visit_param(&mut self, param: &Param) -> Value {
        json!({ "type": "Param", "line": param.node.line, "column": param.node.column, "param_type": param.param_type.to_string(), "name": param.name })
    }

    fn visit_block(&mut self, block: &BlockStmt) -> Value {
        let mut statements = Vec::new();
        for stmt in &block.statements {
            statements.push(match stmt {
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
            });
        }
        json!({ "type": "Block", "line": block.node.line, "column": block.node.column, "statements": statements })
    }

    fn visit_if_stmt(&mut self, if_stmt: &IfStmt) -> Value {
        json!({ "type": "IfStmt", "line": if_stmt.node.line, "column": if_stmt.node.column, "condition": self.visit_expression(&if_stmt.condition), "then_branch": self.visit_statement(&if_stmt.then_branch), "else_branch": if_stmt.else_branch.as_ref().map(|eb| self.visit_statement(eb)) })
    }

    fn visit_switch_stmt(&mut self, switch_stmt: &SwitchStmt) -> Value {
        let mut cases = Vec::new();
        for case in &switch_stmt.cases {
            cases.push(self.visit_case_stmt(case));
        }
        json!({ "type": "SwitchStmt", "line": switch_stmt.node.line, "column": switch_stmt.node.column, "expression": self.visit_expression(&switch_stmt.expression), "cases": cases, "default": switch_stmt.default.as_ref().map(|d| self.visit_statement(d)) })
    }

    fn visit_case_stmt(&mut self, case_stmt: &CaseStmt) -> Value {
        json!({ "type": "CaseStmt", "line": case_stmt.node.line, "column": case_stmt.node.column, "value": self.visit_literal(&case_stmt.value), "body": self.visit_statement(&case_stmt.body) })
    }

    fn visit_while_stmt(&mut self, while_stmt: &WhileStmt) -> Value {
        json!({ "type": "WhileStmt", "line": while_stmt.node.line, "column": while_stmt.node.column, "condition": self.visit_expression(&while_stmt.condition), "body": self.visit_statement(&while_stmt.body) })
    }

    fn visit_for_stmt(&mut self, for_stmt: &ForStmt) -> Value {
        json!({ "type": "ForStmt", "line": for_stmt.node.line, "column": for_stmt.node.column, "init": for_stmt.init.as_ref().map(|i| self.visit_statement(i)), "condition": for_stmt.condition.as_ref().map(|c| self.visit_expression(c)), "update": for_stmt.update.as_ref().map(|u| self.visit_expression(u)), "body": self.visit_statement(&for_stmt.body) })
    }

    fn visit_return_stmt(&mut self, return_stmt: &ReturnStmt) -> Value {
        json!({ "type": "ReturnStmt", "line": return_stmt.node.line, "column": return_stmt.node.column, "value": return_stmt.value.as_ref().map(|v| self.visit_expression(v)) })
    }

    fn visit_expr_stmt(&mut self, expr_stmt: &ExprStmt) -> Value {
        json!({ "type": "ExprStmt", "line": expr_stmt.node.line, "column": expr_stmt.node.column, "expression": self.visit_expression(&expr_stmt.expr) })
    }

    fn visit_empty_stmt(&mut self, empty_stmt: &EmptyStmt) -> Value {
        json!({ "type": "EmptyStmt", "line": empty_stmt.node.line, "column": empty_stmt.node.column })
    }

    fn visit_break_stmt(&mut self, break_stmt: &BreakStmt) -> Value {
        json!({ "type": "BreakStmt", "line": break_stmt.node.line, "column": break_stmt.node.column })
    }

    fn visit_continue_stmt(&mut self, continue_stmt: &ContinueStmt) -> Value {
        json!({ "type": "ContinueStmt", "line": continue_stmt.node.line, "column": continue_stmt.node.column })
    }

    fn visit_literal(&mut self, literal: &Literal) -> Value {
        let value = match &literal.value {
            LiteralValue::Int(i) => json!({ "int": i }),
            LiteralValue::Float(f) => json!({ "float": f }),
            LiteralValue::Bool(b) => json!({ "bool": b }),
            LiteralValue::String(s) => json!({ "string": s }),
        };
        json!({ "type": "Literal", "line": literal.node.line, "column": literal.node.column, "value": value })
    }

    fn visit_identifier(&mut self, identifier: &IdentifierExpr) -> Value {
        json!({ "type": "Identifier", "line": identifier.node.line, "column": identifier.node.column, "name": identifier.name })
    }

    fn visit_binary(&mut self, binary: &BinaryExpr) -> Value {
        json!({ "type": "BinaryExpr", "line": binary.node.line, "column": binary.node.column, "operator": binary.operator.to_string(), "left": self.visit_expression(&binary.left), "right": self.visit_expression(&binary.right) })
    }

    fn visit_unary(&mut self, unary: &UnaryExpr) -> Value {
        json!({ "type": "UnaryExpr", "line": unary.node.line, "column": unary.node.column, "operator": unary.operator.to_string(), "operand": self.visit_expression(&unary.operand) })
    }

    fn visit_assignment(&mut self, assignment: &AssignmentExpr) -> Value {
        json!({ "type": "AssignmentExpr", "line": assignment.node.line, "column": assignment.node.column, "operator": assignment.operator.to_string(), "target": self.visit_expression(&assignment.target), "value": self.visit_expression(&assignment.value) })
    }

    fn visit_call(&mut self, call: &CallExpr) -> Value {
        let mut arguments = Vec::new();
        for arg in &call.arguments {
            arguments.push(self.visit_expression(arg));
        }
        json!({ "type": "CallExpr", "line": call.node.line, "column": call.node.column, "callee": self.visit_expression(&call.callee), "arguments": arguments })
    }

    fn visit_struct_access(&mut self, access: &StructAccessExpr) -> Value {
        json!({ "type": "StructAccessExpr", "line": access.node.line, "column": access.node.column, "object": self.visit_expression(&access.object), "field": access.field })
    }

    fn visit_grouped(&mut self, grouped: &GroupedExpr) -> Value {
        json!({ "type": "GroupedExpr", "line": grouped.node.line, "column": grouped.node.column, "expression": self.visit_expression(&grouped.expr) })
    }

    fn visit_array_access(&mut self, access: &ArrayAccessExpr) -> Value {
        json!({
            "type": "ArrayAccess",
            "line": access.node.line,
            "column": access.node.column,
            "array": self.visit_expression(&access.array),
            "index": self.visit_expression(&access.index)
        })
    }
}

impl JsonGenerator {
    fn visit_expression(&mut self, expr: &Expression) -> Value {
        match expr {
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Identifier(i) => self.visit_identifier(i),
            Expression::Binary(b) => self.visit_binary(b),
            Expression::Unary(u) => self.visit_unary(u),
            Expression::Assignment(a) => self.visit_assignment(a),
            Expression::Call(c) => self.visit_call(c),
            Expression::StructAccess(sa) => self.visit_struct_access(sa),
            Expression::ArrayAccess(aa) => {
                json!({ "type": "ArrayAccess", "array": self.visit_expression(&aa.array), "index": self.visit_expression(&aa.index) })
            }
            Expression::Grouped(g) => self.visit_grouped(g),
        }
    }

    fn visit_statement(&mut self, stmt: &Statement) -> Value {
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
