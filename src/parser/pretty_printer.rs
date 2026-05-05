//! Pretty printer для AST - выводит AST в человекочитаемом формате

use crate::parser::ast::*;
use crate::parser::visitor::Visitor;

/// Pretty printer для AST
pub struct PrettyPrinter {
    output: String,
    indent_level: usize,
    indent_size: usize,
}

impl PrettyPrinter {
    /// Создает новый pretty printer
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            indent_size: 2,
        }
    }

    /// Возвращает результат форматирования
    pub fn into_string(self) -> String {
        self.output
    }

    /// Добавляет отступ
    fn indent(&mut self) {
        self.indent_level += 1;
    }

    /// Убирает отступ
    fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    /// Выводит текущий отступ
    fn write_indent(&mut self) {
        self.output
            .push_str(&" ".repeat(self.indent_level * self.indent_size));
    }

    /// Выводит строку с отступом
    fn writeln(&mut self, s: &str) {
        self.write_indent();
        self.output.push_str(s);
        self.output.push('\n');
    }

    /// Форматирует программу
    pub fn format_program(&mut self, program: &Program) -> String {
        self.visit_program(program);
        self.output.clone()
    }
}

impl Visitor<()> for PrettyPrinter {
    fn visit_program(&mut self, program: &Program) {
        self.writeln(&format!("Program [line {}]:", program.node.line));
        self.indent();
        if program.declarations.is_empty() {
            self.writeln("(empty program)");
        } else {
            for decl in &program.declarations {
                match decl {
                    Declaration::Function(f) => self.visit_function_decl(f),
                    Declaration::Struct(s) => self.visit_struct_decl(s),
                    Declaration::Variable(v) => self.visit_var_decl(v),
                }
            }
        }
        self.dedent();
    }

    fn visit_function_decl(&mut self, func: &FunctionDecl) {
        self.writeln(&format!(
            "FunctionDecl: {} -> {} [line {}]:",
            func.name, func.return_type, func.node.line
        ));
        self.indent();
        self.write_indent();
        self.output.push_str("Parameters: [");
        for (i, param) in func.parameters.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.output
                .push_str(&format!("{}: {}", param.name, param.param_type));
        }
        self.output.push_str("]\n");
        self.visit_block(&func.body);
        self.dedent();
    }

    fn visit_struct_decl(&mut self, struct_decl: &StructDecl) {
        self.writeln(&format!(
            "StructDecl: {} [line {}]:",
            struct_decl.name, struct_decl.node.line
        ));
        self.indent();
        self.writeln("Fields:");
        self.indent();
        for field in &struct_decl.fields {
            self.visit_var_decl(field);
        }
        self.dedent();
        self.dedent();
    }

    fn visit_var_decl(&mut self, var_decl: &VarDecl) {
        self.write_indent();
        self.output
            .push_str(&format!("VarDecl: {} {}", var_decl.var_type, var_decl.name));
        if let Some(init) = &var_decl.initializer {
            self.output.push_str(" = ");
            match init.as_ref() {
                Expression::Literal(l) => {
                    self.output.push_str(&format!("{}", l.value));
                }
                Expression::Identifier(i) => {
                    self.output.push_str(&i.name);
                }
                Expression::Binary(b) => {
                    self.output.push('(');
                    self.visit_binary(b);
                    self.output.push(')');
                }
                Expression::Unary(u) => {
                    self.output.push('(');
                    self.visit_unary(u);
                    self.output.push(')');
                }
                Expression::Assignment(a) => {
                    self.output.push('(');
                    self.visit_assignment(a);
                    self.output.push(')');
                }
                Expression::Call(c) => {
                    self.visit_call(c);
                }
                Expression::StructAccess(sa) => {
                    self.visit_struct_access(sa);
                }
                Expression::Grouped(g) => {
                    self.output.push('(');
                    self.visit_grouped(g);
                    self.output.push(')');
                }
                Expression::ArrayAccess(_aa) => {
                    self.output.push_str("Arr[...]");
                }
            }
        }
        self.output.push('\n');
    }

    fn visit_param(&mut self, _param: &Param) {}

    fn visit_block(&mut self, block: &BlockStmt) {
        self.writeln(&format!("Block [line {}]:", block.node.line));
        self.indent();
        if block.statements.is_empty() {
            self.writeln("(empty block)");
        } else {
            for stmt in &block.statements {
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
        self.dedent();
    }

    fn visit_if_stmt(&mut self, if_stmt: &IfStmt) {
        self.writeln(&format!("IfStmt [line {}]:", if_stmt.node.line));
        self.indent();
        self.write_indent();
        self.output.push_str("Condition: ");
        match if_stmt.condition.as_ref() {
            Expression::Literal(l) => {
                self.output.push_str(&format!("{}", l.value));
            }
            Expression::Identifier(i) => {
                self.output.push_str(&i.name);
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
            Expression::ArrayAccess(_) => {
                self.output.push_str("Arr[...]");
            }
        }
        self.output.push('\n');
        self.writeln("Then:");
        self.indent();
        match if_stmt.then_branch.as_ref() {
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
        self.dedent();
        if let Some(else_branch) = &if_stmt.else_branch {
            self.writeln("Else:");
            self.indent();
            match else_branch.as_ref() {
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
            self.dedent();
        }
        self.dedent();
    }

    fn visit_switch_stmt(&mut self, switch_stmt: &SwitchStmt) {
        self.writeln(&format!("SwitchStmt [line {}]:", switch_stmt.node.line));
        self.indent();
        self.write_indent();
        self.output.push_str("Expression: ");
        match switch_stmt.expression.as_ref() {
            Expression::Literal(l) => {
                self.output.push_str(&format!("{}", l.value));
            }
            Expression::Identifier(i) => {
                self.output.push_str(&i.name);
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
            Expression::ArrayAccess(_) => {
                self.output.push_str("Arr[...]");
            }
        }
        self.output.push('\n');
        for case in &switch_stmt.cases {
            self.visit_case_stmt(case);
        }
        if let Some(default) = &switch_stmt.default {
            self.writeln("Default:");
            self.indent();
            match default.as_ref() {
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
            self.dedent();
        }
        self.dedent();
    }

    fn visit_case_stmt(&mut self, case_stmt: &CaseStmt) {
        self.writeln(&format!("Case {}:", case_stmt.value.value));
        self.indent();
        match case_stmt.body.as_ref() {
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
        self.dedent();
    }

    fn visit_while_stmt(&mut self, while_stmt: &WhileStmt) {
        self.writeln(&format!("WhileStmt [line {}]:", while_stmt.node.line));
        self.indent();
        self.write_indent();
        self.output.push_str("Condition: ");
        match while_stmt.condition.as_ref() {
            Expression::Literal(l) => {
                self.output.push_str(&format!("{}", l.value));
            }
            Expression::Identifier(i) => {
                self.output.push_str(&i.name);
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
            Expression::ArrayAccess(_) => {
                self.output.push_str("Arr[...]");
            }
        }
        self.output.push('\n');
        self.writeln("Body:");
        self.indent();
        match while_stmt.body.as_ref() {
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
        self.dedent();
        self.dedent();
    }

    fn visit_for_stmt(&mut self, for_stmt: &ForStmt) {
        self.writeln(&format!("ForStmt [line {}]:", for_stmt.node.line));
        self.indent();
        if let Some(init) = &for_stmt.init {
            self.writeln("Init:");
            self.indent();
            match init.as_ref() {
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
            self.dedent();
        }
        if let Some(condition) = &for_stmt.condition {
            self.write_indent();
            self.output.push_str("Condition: ");
            match condition.as_ref() {
                Expression::Literal(l) => {
                    self.output.push_str(&format!("{}", l.value));
                }
                Expression::Identifier(i) => {
                    self.output.push_str(&i.name);
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
                Expression::ArrayAccess(_) => {
                    self.output.push_str("Arr[...]");
                }
            }
            self.output.push('\n');
        }
        if let Some(update) = &for_stmt.update {
            self.write_indent();
            self.output.push_str("Update: ");
            match update.as_ref() {
                Expression::Literal(l) => {
                    self.output.push_str(&format!("{}", l.value));
                }
                Expression::Identifier(i) => {
                    self.output.push_str(&i.name);
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
                Expression::ArrayAccess(_) => {
                    self.output.push_str("Arr[...]");
                }
            }
            self.output.push('\n');
        }
        self.writeln("Body:");
        self.indent();
        match for_stmt.body.as_ref() {
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
        self.dedent();
        self.dedent();
    }

    fn visit_return_stmt(&mut self, return_stmt: &ReturnStmt) {
        self.write_indent();
        self.output.push_str("Return");
        if let Some(value) = &return_stmt.value {
            self.output.push_str(": ");
            match value.as_ref() {
                Expression::Literal(l) => {
                    self.output.push_str(&format!("{}", l.value));
                }
                Expression::Identifier(i) => {
                    self.output.push_str(&i.name);
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
                Expression::ArrayAccess(_) => {
                    self.output.push_str("Expr: Arr[...]");
                }
            }
        }
        self.output.push('\n');
    }

    fn visit_expr_stmt(&mut self, expr_stmt: &ExprStmt) {
        self.write_indent();
        match expr_stmt.expr.as_ref() {
            Expression::Literal(l) => {
                self.output.push_str(&format!("Expr: {}", l.value));
            }
            Expression::Identifier(i) => {
                self.output.push_str(&format!("Expr: {}", i.name));
            }
            Expression::Binary(b) => {
                self.output.push_str("Expr: ");
                self.visit_binary(b);
            }
            Expression::Unary(u) => {
                self.output.push_str("Expr: ");
                self.visit_unary(u);
            }
            Expression::Assignment(a) => {
                self.output.push_str("Expr: ");
                self.visit_assignment(a);
            }
            Expression::Call(c) => {
                self.output.push_str("Expr: ");
                self.visit_call(c);
            }
            Expression::StructAccess(sa) => {
                self.output.push_str("Expr: ");
                self.visit_struct_access(sa);
            }
            Expression::Grouped(g) => {
                self.output.push_str("Expr: ");
                self.visit_grouped(g);
            }
            Expression::ArrayAccess(_) => {
                self.output.push_str("Arr[...]");
            }
        }
        self.output.push('\n');
    }

    fn visit_empty_stmt(&mut self, _empty_stmt: &EmptyStmt) {
        self.writeln("EmptyStmt: ;");
    }
    fn visit_break_stmt(&mut self, _break_stmt: &BreakStmt) {
        self.writeln("Break");
    }
    fn visit_continue_stmt(&mut self, _continue_stmt: &ContinueStmt) {
        self.writeln("Continue");
    }

    fn visit_literal(&mut self, literal: &Literal) {
        self.output.push_str(&format!("{}", literal.value));
    }
    fn visit_identifier(&mut self, identifier: &IdentifierExpr) {
        self.output.push_str(&identifier.name);
    }

    fn visit_binary(&mut self, binary: &BinaryExpr) {
        self.output.push('(');
        match binary.left.as_ref() {
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Identifier(i) => self.visit_identifier(i),
            Expression::Binary(b) => self.visit_binary(b),
            Expression::Unary(u) => self.visit_unary(u),
            Expression::Assignment(a) => self.visit_assignment(a),
            Expression::Call(c) => self.visit_call(c),
            Expression::StructAccess(sa) => self.visit_struct_access(sa),
            Expression::Grouped(g) => self.visit_grouped(g),
            Expression::ArrayAccess(_) => {
                self.output.push_str("Arr[...]");
            }
        }
        self.output.push_str(&format!(" {} ", binary.operator));
        match binary.right.as_ref() {
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Identifier(i) => self.visit_identifier(i),
            Expression::Binary(b) => self.visit_binary(b),
            Expression::Unary(u) => self.visit_unary(u),
            Expression::Assignment(a) => self.visit_assignment(a),
            Expression::Call(c) => self.visit_call(c),
            Expression::StructAccess(sa) => self.visit_struct_access(sa),
            Expression::Grouped(g) => self.visit_grouped(g),
            Expression::ArrayAccess(_) => {
                self.output.push_str("Arr[...]");
            }
        }
        self.output.push(')');
    }

    fn visit_unary(&mut self, unary: &UnaryExpr) {
        self.output.push_str(&format!("{}", unary.operator));
        match unary.operand.as_ref() {
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Identifier(i) => self.visit_identifier(i),
            Expression::Binary(b) => {
                self.output.push('(');
                self.visit_binary(b);
                self.output.push(')');
            }
            Expression::Unary(u) => self.visit_unary(u),
            Expression::Assignment(a) => {
                self.output.push('(');
                self.visit_assignment(a);
                self.output.push(')');
            }
            Expression::Call(c) => self.visit_call(c),
            Expression::StructAccess(sa) => self.visit_struct_access(sa),
            Expression::Grouped(g) => {
                self.output.push('(');
                self.visit_grouped(g);
                self.output.push(')');
            }
            Expression::ArrayAccess(_) => {
                self.output.push_str("Arr[...]");
            }
        }
    }

    fn visit_assignment(&mut self, assignment: &AssignmentExpr) {
        match assignment.target.as_ref() {
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Identifier(i) => self.visit_identifier(i),
            Expression::Binary(b) => self.visit_binary(b),
            Expression::Unary(u) => self.visit_unary(u),
            Expression::Assignment(a) => self.visit_assignment(a),
            Expression::Call(c) => self.visit_call(c),
            Expression::StructAccess(sa) => self.visit_struct_access(sa),
            Expression::Grouped(g) => self.visit_grouped(g),
            Expression::ArrayAccess(_) => {
                self.output.push_str("Arr[...]");
            }
        }
        self.output.push_str(&format!(" {} ", assignment.operator));
        match assignment.value.as_ref() {
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Identifier(i) => self.visit_identifier(i),
            Expression::Binary(b) => self.visit_binary(b),
            Expression::Unary(u) => self.visit_unary(u),
            Expression::Assignment(a) => self.visit_assignment(a),
            Expression::Call(c) => self.visit_call(c),
            Expression::StructAccess(sa) => self.visit_struct_access(sa),
            Expression::Grouped(g) => self.visit_grouped(g),
            Expression::ArrayAccess(_) => {
                self.output.push_str("Arr[...]");
            }
        }
    }

    fn visit_call(&mut self, call: &CallExpr) {
        match call.callee.as_ref() {
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Identifier(i) => self.visit_identifier(i),
            Expression::Binary(b) => self.visit_binary(b),
            Expression::Unary(u) => self.visit_unary(u),
            Expression::Assignment(a) => self.visit_assignment(a),
            Expression::Call(c) => self.visit_call(c),
            Expression::StructAccess(sa) => self.visit_struct_access(sa),
            Expression::Grouped(g) => self.visit_grouped(g),
            Expression::ArrayAccess(_) => {
                self.output.push_str("Arr[...]");
            }
        }
        self.output.push('(');
        for (i, arg) in call.arguments.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            match arg {
                Expression::Literal(l) => self.visit_literal(l),
                Expression::Identifier(i) => self.visit_identifier(i),
                Expression::Binary(b) => self.visit_binary(b),
                Expression::Unary(u) => self.visit_unary(u),
                Expression::Assignment(a) => self.visit_assignment(a),
                Expression::Call(c) => self.visit_call(c),
                Expression::StructAccess(sa) => self.visit_struct_access(sa),
                Expression::Grouped(g) => self.visit_grouped(g),
                Expression::ArrayAccess(_) => {
                    self.output.push_str("Arr[...]");
                }
            }
        }
        self.output.push(')');
    }

    fn visit_struct_access(&mut self, access: &StructAccessExpr) {
        match access.object.as_ref() {
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Identifier(i) => self.visit_identifier(i),
            Expression::Binary(b) => self.visit_binary(b),
            Expression::Unary(u) => self.visit_unary(u),
            Expression::Assignment(a) => self.visit_assignment(a),
            Expression::Call(c) => self.visit_call(c),
            Expression::StructAccess(sa) => self.visit_struct_access(sa),
            Expression::Grouped(g) => self.visit_grouped(g),
            Expression::ArrayAccess(_) => {
                self.output.push_str("Arr[...]");
            }
        }
        self.output.push_str(&format!(".{}", access.field));
    }

    fn visit_grouped(&mut self, grouped: &GroupedExpr) {
        self.output.push('(');
        match grouped.expr.as_ref() {
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Identifier(i) => self.visit_identifier(i),
            Expression::Binary(b) => self.visit_binary(b),
            Expression::Unary(u) => self.visit_unary(u),
            Expression::Assignment(a) => self.visit_assignment(a),
            Expression::Call(c) => self.visit_call(c),
            Expression::StructAccess(sa) => self.visit_struct_access(sa),
            Expression::Grouped(g) => self.visit_grouped(g),
            Expression::ArrayAccess(_) => {
                self.output.push_str("Arr[...]");
            }
        }
        self.output.push(')');
    }

    fn visit_array_access(&mut self, _access: &ArrayAccessExpr) {
        self.writeln("ArrayAccess");
    }
}
