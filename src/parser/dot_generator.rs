//! Генератор DOT-графов для визуализации AST
//!
//! Создает файлы в формате Graphviz DOT для визуализации синтаксического дерева.

use crate::parser::ast::*;
use crate::parser::visitor::Visitor;

/// Генератор DOT-графов
pub struct DotGenerator {
    output: String,
    node_counter: usize,
    /// Цветовая схема для разных типов узлов
    colors: ColorScheme,
}

/// Цветовая схема для узлов AST
#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub program: String,
    pub function: String,
    pub struct_node: String,
    pub variable: String,
    pub statement: String,
    pub expression: String,
    pub literal: String,
    pub identifier: String,
    pub operator: String,
    pub call: String,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            program: "lightblue".to_string(),
            function: "lightgreen".to_string(),
            struct_node: "lightcoral".to_string(),
            variable: "lightyellow".to_string(),
            statement: "lightgray".to_string(),
            expression: "lightsalmon".to_string(),
            literal: "lightpink".to_string(),
            identifier: "lightcyan".to_string(),
            operator: "lightgoldenrodyellow".to_string(),
            call: "lightseagreen".to_string(),
        }
    }
}

impl DotGenerator {
    /// Создает новый генератор DOT с цветовой схемой по умолчанию
    pub fn new() -> Self {
        Self {
            output: String::new(),
            node_counter: 0,
            colors: ColorScheme::default(),
        }
    }

    /// Создает новый генератор DOT с пользовательской цветовой схемой
    pub fn with_colors(colors: ColorScheme) -> Self {
        Self {
            output: String::new(),
            node_counter: 0,
            colors,
        }
    }

    /// Генерирует уникальный идентификатор для узла
    fn next_node_id(&mut self) -> usize {
        self.node_counter += 1;
        self.node_counter
    }

    /// Форматирует узел в DOT-формате с экранированием метки
    fn format_node(&self, id: usize, label: &str, color: &str) -> String {
        let escaped_label = self.escape_label(label);
        format!(
            "    node{} [label=\"{}\", shape=box, style=\"filled,rounded\", fillcolor={}];\n",
            id, escaped_label, color
        )
    }

    /// Форматирует ребро между узлами
    fn format_edge(&self, from: usize, to: usize, label: Option<&str>) -> String {
        if let Some(label) = label {
            let escaped_label = self.escape_label(label);
            format!(
                "    node{} -> node{} [label=\"{}\"];\n",
                from, to, escaped_label
            )
        } else {
            format!("    node{} -> node{};\n", from, to)
        }
    }

    /// Экранирует специальные символы в метках для DOT формата
    fn escape_label(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len() * 2);

        for c in text.chars() {
            match c {
                '"' => result.push_str("\\\""),
                '\\' => result.push_str("\\\\"),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                '<' => result.push_str("\\<"),
                '>' => result.push_str("\\>"),
                '{' => result.push_str("\\{"),
                '}' => result.push_str("\\}"),
                '|' => result.push_str("\\|"),
                '[' => result.push_str("\\["),
                ']' => result.push_str("\\]"),
                _ => result.push(c),
            }
        }

        result
    }

    /// Генерирует DOT-граф для программы
    pub fn generate(&mut self, program: &Program) -> String {
        self.output.clear();
        self.node_counter = 0;

        self.output.push_str("digraph AST {\n");
        self.output
            .push_str("    graph [rankdir=TB, splines=ortho];\n");
        self.output
            .push_str("    node [fontname=\"Courier New\"];\n");
        self.output
            .push_str("    edge [fontname=\"Courier New\"];\n\n");

        let _root_id = self.visit_program(program);

        self.output.push_str("}\n");
        self.output.clone()
    }

    /// Сохраняет граф в файл
    pub fn save_to_file(&mut self, program: &Program, filename: &str) -> std::io::Result<()> {
        let dot = self.generate(program);
        std::fs::write(filename, dot)
    }
}

impl Visitor<usize> for DotGenerator {
    fn visit_program(&mut self, program: &Program) -> usize {
        let node_id = self.next_node_id();
        let label = format!("Program [line {}]", program.node.line);
        let node_str = self.format_node(node_id, &label, &self.colors.program);
        self.output.push_str(&node_str);

        for decl in &program.declarations {
            let child_id = match decl {
                Declaration::Function(f) => self.visit_function_decl(f),
                Declaration::Struct(s) => self.visit_struct_decl(s),
                Declaration::Variable(v) => self.visit_var_decl(v),
            };
            self.output
                .push_str(&self.format_edge(node_id, child_id, None));
        }

        node_id
    }

    fn visit_function_decl(&mut self, func: &FunctionDecl) -> usize {
        let node_id = self.next_node_id();
        let label = format!(
            "Function {} [line {}]\\nreturns {}",
            func.name, func.node.line, func.return_type
        );
        let node_str = self.format_node(node_id, &label, &self.colors.function);
        self.output.push_str(&node_str);

        if !func.parameters.is_empty() {
            let params_id = self.next_node_id();
            let params_label = format!("Parameters ({})", func.parameters.len());
            let params_node_str =
                self.format_node(params_id, &params_label, &self.colors.statement);
            self.output.push_str(&params_node_str);
            self.output
                .push_str(&self.format_edge(node_id, params_id, Some("params")));

            for param in &func.parameters {
                let param_id = self.visit_param(param);
                self.output
                    .push_str(&self.format_edge(params_id, param_id, None));
            }
        }

        let body_id = self.visit_block(&func.body);
        self.output
            .push_str(&self.format_edge(node_id, body_id, Some("body")));

        node_id
    }

    fn visit_struct_decl(&mut self, struct_decl: &StructDecl) -> usize {
        let node_id = self.next_node_id();
        let label = format!(
            "Struct {} [line {}]",
            struct_decl.name, struct_decl.node.line
        );
        let node_str = self.format_node(node_id, &label, &self.colors.struct_node);
        self.output.push_str(&node_str);

        if !struct_decl.fields.is_empty() {
            let fields_id = self.next_node_id();
            let fields_label = format!("Fields ({})", struct_decl.fields.len());
            let fields_node_str =
                self.format_node(fields_id, &fields_label, &self.colors.statement);
            self.output.push_str(&fields_node_str);
            self.output
                .push_str(&self.format_edge(node_id, fields_id, Some("fields")));

            for field in &struct_decl.fields {
                let field_id = self.visit_var_decl(field);
                self.output
                    .push_str(&self.format_edge(fields_id, field_id, None));
            }
        }

        node_id
    }

    fn visit_var_decl(&mut self, var_decl: &VarDecl) -> usize {
        let node_id = self.next_node_id();
        let label = format!(
            "VarDecl\\n{} {}{}",
            var_decl.var_type,
            var_decl.name,
            if var_decl.initializer.is_some() {
                " = ..."
            } else {
                ""
            }
        );
        let node_str = self.format_node(node_id, &label, &self.colors.variable);
        self.output.push_str(&node_str);

        if let Some(init) = &var_decl.initializer {
            let init_id = match init.as_ref() {
                Expression::Literal(l) => self.visit_literal(l),
                Expression::Identifier(i) => self.visit_identifier(i),
                Expression::Binary(b) => self.visit_binary(b),
                Expression::Unary(u) => self.visit_unary(u),
                Expression::Assignment(a) => self.visit_assignment(a),
                Expression::Call(c) => self.visit_call(c),
                Expression::StructAccess(sa) => self.visit_struct_access(sa),
                Expression::Grouped(g) => self.visit_grouped(g),
            };
            self.output
                .push_str(&self.format_edge(node_id, init_id, Some("init")));
        }

        node_id
    }

    fn visit_param(&mut self, param: &Param) -> usize {
        let node_id = self.next_node_id();
        let label = format!("Param\\n{}: {}", param.name, param.param_type);
        let node_str = self.format_node(node_id, &label, &self.colors.variable);
        self.output.push_str(&node_str);
        node_id
    }

    fn visit_block(&mut self, block: &BlockStmt) -> usize {
        let node_id = self.next_node_id();
        let label = format!(
            "Block [line {}]\\n{} stmts",
            block.node.line,
            block.statements.len()
        );
        let node_str = self.format_node(node_id, &label, &self.colors.statement);
        self.output.push_str(&node_str);

        for stmt in &block.statements {
            let stmt_id = match stmt {
                Statement::VariableDecl(v) => self.visit_var_decl(v),
                Statement::Expression(e) => self.visit_expr_stmt(e),
                Statement::If(i) => self.visit_if_stmt(i),
                Statement::While(w) => self.visit_while_stmt(w),
                Statement::For(f) => self.visit_for_stmt(f),
                Statement::Return(r) => self.visit_return_stmt(r),
                Statement::Block(b) => self.visit_block(b),
                Statement::Empty(e) => self.visit_empty_stmt(e),
            };
            self.output
                .push_str(&self.format_edge(node_id, stmt_id, None));
        }

        node_id
    }

    fn visit_if_stmt(&mut self, if_stmt: &IfStmt) -> usize {
        let node_id = self.next_node_id();
        let label = format!("IfStmt [line {}]", if_stmt.node.line);
        let node_str = self.format_node(node_id, &label, &self.colors.statement);
        self.output.push_str(&node_str);

        let cond_id = match if_stmt.condition.as_ref() {
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Identifier(i) => self.visit_identifier(i),
            Expression::Binary(b) => self.visit_binary(b),
            Expression::Unary(u) => self.visit_unary(u),
            Expression::Assignment(a) => self.visit_assignment(a),
            Expression::Call(c) => self.visit_call(c),
            Expression::StructAccess(sa) => self.visit_struct_access(sa),
            Expression::Grouped(g) => self.visit_grouped(g),
        };
        self.output
            .push_str(&self.format_edge(node_id, cond_id, Some("cond")));

        let then_id = match if_stmt.then_branch.as_ref() {
            Statement::VariableDecl(v) => self.visit_var_decl(v),
            Statement::Expression(e) => self.visit_expr_stmt(e),
            Statement::If(i) => self.visit_if_stmt(i),
            Statement::While(w) => self.visit_while_stmt(w),
            Statement::For(f) => self.visit_for_stmt(f),
            Statement::Return(r) => self.visit_return_stmt(r),
            Statement::Block(b) => self.visit_block(b),
            Statement::Empty(e) => self.visit_empty_stmt(e),
        };
        self.output
            .push_str(&self.format_edge(node_id, then_id, Some("then")));

        if let Some(else_branch) = &if_stmt.else_branch {
            let else_id = match else_branch.as_ref() {
                Statement::VariableDecl(v) => self.visit_var_decl(v),
                Statement::Expression(e) => self.visit_expr_stmt(e),
                Statement::If(i) => self.visit_if_stmt(i),
                Statement::While(w) => self.visit_while_stmt(w),
                Statement::For(f) => self.visit_for_stmt(f),
                Statement::Return(r) => self.visit_return_stmt(r),
                Statement::Block(b) => self.visit_block(b),
                Statement::Empty(e) => self.visit_empty_stmt(e),
            };
            self.output
                .push_str(&self.format_edge(node_id, else_id, Some("else")));
        }

        node_id
    }

    fn visit_while_stmt(&mut self, while_stmt: &WhileStmt) -> usize {
        let node_id = self.next_node_id();
        let label = format!("WhileStmt [line {}]", while_stmt.node.line);
        let node_str = self.format_node(node_id, &label, &self.colors.statement);
        self.output.push_str(&node_str);

        let cond_id = match while_stmt.condition.as_ref() {
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Identifier(i) => self.visit_identifier(i),
            Expression::Binary(b) => self.visit_binary(b),
            Expression::Unary(u) => self.visit_unary(u),
            Expression::Assignment(a) => self.visit_assignment(a),
            Expression::Call(c) => self.visit_call(c),
            Expression::StructAccess(sa) => self.visit_struct_access(sa),
            Expression::Grouped(g) => self.visit_grouped(g),
        };
        self.output
            .push_str(&self.format_edge(node_id, cond_id, Some("cond")));

        let body_id = match while_stmt.body.as_ref() {
            Statement::VariableDecl(v) => self.visit_var_decl(v),
            Statement::Expression(e) => self.visit_expr_stmt(e),
            Statement::If(i) => self.visit_if_stmt(i),
            Statement::While(w) => self.visit_while_stmt(w),
            Statement::For(f) => self.visit_for_stmt(f),
            Statement::Return(r) => self.visit_return_stmt(r),
            Statement::Block(b) => self.visit_block(b),
            Statement::Empty(e) => self.visit_empty_stmt(e),
        };
        self.output
            .push_str(&self.format_edge(node_id, body_id, Some("body")));

        node_id
    }

    fn visit_for_stmt(&mut self, for_stmt: &ForStmt) -> usize {
        let node_id = self.next_node_id();
        let label = format!("ForStmt [line {}]", for_stmt.node.line);
        let node_str = self.format_node(node_id, &label, &self.colors.statement);
        self.output.push_str(&node_str);

        if let Some(init) = &for_stmt.init {
            let init_id = match init.as_ref() {
                Statement::VariableDecl(v) => self.visit_var_decl(v),
                Statement::Expression(e) => self.visit_expr_stmt(e),
                Statement::If(i) => self.visit_if_stmt(i),
                Statement::While(w) => self.visit_while_stmt(w),
                Statement::For(f) => self.visit_for_stmt(f),
                Statement::Return(r) => self.visit_return_stmt(r),
                Statement::Block(b) => self.visit_block(b),
                Statement::Empty(e) => self.visit_empty_stmt(e),
            };
            self.output
                .push_str(&self.format_edge(node_id, init_id, Some("init")));
        }

        if let Some(cond) = &for_stmt.condition {
            let cond_id = match cond.as_ref() {
                Expression::Literal(l) => self.visit_literal(l),
                Expression::Identifier(i) => self.visit_identifier(i),
                Expression::Binary(b) => self.visit_binary(b),
                Expression::Unary(u) => self.visit_unary(u),
                Expression::Assignment(a) => self.visit_assignment(a),
                Expression::Call(c) => self.visit_call(c),
                Expression::StructAccess(sa) => self.visit_struct_access(sa),
                Expression::Grouped(g) => self.visit_grouped(g),
            };
            self.output
                .push_str(&self.format_edge(node_id, cond_id, Some("cond")));
        }

        if let Some(update) = &for_stmt.update {
            let update_id = match update.as_ref() {
                Expression::Literal(l) => self.visit_literal(l),
                Expression::Identifier(i) => self.visit_identifier(i),
                Expression::Binary(b) => self.visit_binary(b),
                Expression::Unary(u) => self.visit_unary(u),
                Expression::Assignment(a) => self.visit_assignment(a),
                Expression::Call(c) => self.visit_call(c),
                Expression::StructAccess(sa) => self.visit_struct_access(sa),
                Expression::Grouped(g) => self.visit_grouped(g),
            };
            self.output
                .push_str(&self.format_edge(node_id, update_id, Some("update")));
        }

        let body_id = match for_stmt.body.as_ref() {
            Statement::VariableDecl(v) => self.visit_var_decl(v),
            Statement::Expression(e) => self.visit_expr_stmt(e),
            Statement::If(i) => self.visit_if_stmt(i),
            Statement::While(w) => self.visit_while_stmt(w),
            Statement::For(f) => self.visit_for_stmt(f),
            Statement::Return(r) => self.visit_return_stmt(r),
            Statement::Block(b) => self.visit_block(b),
            Statement::Empty(e) => self.visit_empty_stmt(e),
        };
        self.output
            .push_str(&self.format_edge(node_id, body_id, Some("body")));

        node_id
    }

    fn visit_return_stmt(&mut self, return_stmt: &ReturnStmt) -> usize {
        let node_id = self.next_node_id();
        let label = if return_stmt.value.is_some() {
            "ReturnStmt with value".to_string()
        } else {
            "ReturnStmt void".to_string()
        };
        let node_str = self.format_node(node_id, &label, &self.colors.statement);
        self.output.push_str(&node_str);

        if let Some(value) = &return_stmt.value {
            let value_id = match value.as_ref() {
                Expression::Literal(l) => self.visit_literal(l),
                Expression::Identifier(i) => self.visit_identifier(i),
                Expression::Binary(b) => self.visit_binary(b),
                Expression::Unary(u) => self.visit_unary(u),
                Expression::Assignment(a) => self.visit_assignment(a),
                Expression::Call(c) => self.visit_call(c),
                Expression::StructAccess(sa) => self.visit_struct_access(sa),
                Expression::Grouped(g) => self.visit_grouped(g),
            };
            self.output
                .push_str(&self.format_edge(node_id, value_id, Some("value")));
        }

        node_id
    }

    fn visit_expr_stmt(&mut self, expr_stmt: &ExprStmt) -> usize {
        let node_id = self.next_node_id();
        let node_str = self.format_node(node_id, "ExprStmt", &self.colors.statement);
        self.output.push_str(&node_str);

        let expr_id = match expr_stmt.expr.as_ref() {
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Identifier(i) => self.visit_identifier(i),
            Expression::Binary(b) => self.visit_binary(b),
            Expression::Unary(u) => self.visit_unary(u),
            Expression::Assignment(a) => self.visit_assignment(a),
            Expression::Call(c) => self.visit_call(c),
            Expression::StructAccess(sa) => self.visit_struct_access(sa),
            Expression::Grouped(g) => self.visit_grouped(g),
        };
        self.output
            .push_str(&self.format_edge(node_id, expr_id, None));

        node_id
    }

    fn visit_empty_stmt(&mut self, empty_stmt: &EmptyStmt) -> usize {
        let node_id = self.next_node_id();
        let label = format!("EmptyStmt [line {}]", empty_stmt.node.line);
        let node_str = self.format_node(node_id, &label, &self.colors.statement);
        self.output.push_str(&node_str);
        node_id
    }

    fn visit_literal(&mut self, literal: &Literal) -> usize {
        let node_id = self.next_node_id();

        let value_str = match &literal.value {
            LiteralValue::String(s) => {
                format!("\\\"{}\\\"", self.escape_label(s))
            }
            LiteralValue::Int(i) => format!("{}", i),
            LiteralValue::Float(f) => format!("{}", f),
            LiteralValue::Bool(b) => format!("{}", b),
        };

        let label = format!("Literal\\n{}", value_str);
        let node_str = self.format_node(node_id, &label, &self.colors.literal);
        self.output.push_str(&node_str);
        node_id
    }

    fn visit_identifier(&mut self, identifier: &IdentifierExpr) -> usize {
        let node_id = self.next_node_id();
        let label = format!("Identifier\\n{}", identifier.name);
        let node_str = self.format_node(node_id, &label, &self.colors.identifier);
        self.output.push_str(&node_str);
        node_id
    }

    fn visit_binary(&mut self, binary: &BinaryExpr) -> usize {
        let node_id = self.next_node_id();
        let label = format!("BinaryOp\\n{}", binary.operator);
        let node_str = self.format_node(node_id, &label, &self.colors.operator);
        self.output.push_str(&node_str);

        let left_id = match binary.left.as_ref() {
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Identifier(i) => self.visit_identifier(i),
            Expression::Binary(b) => self.visit_binary(b),
            Expression::Unary(u) => self.visit_unary(u),
            Expression::Assignment(a) => self.visit_assignment(a),
            Expression::Call(c) => self.visit_call(c),
            Expression::StructAccess(sa) => self.visit_struct_access(sa),
            Expression::Grouped(g) => self.visit_grouped(g),
        };
        self.output
            .push_str(&self.format_edge(node_id, left_id, Some("left")));

        let right_id = match binary.right.as_ref() {
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Identifier(i) => self.visit_identifier(i),
            Expression::Binary(b) => self.visit_binary(b),
            Expression::Unary(u) => self.visit_unary(u),
            Expression::Assignment(a) => self.visit_assignment(a),
            Expression::Call(c) => self.visit_call(c),
            Expression::StructAccess(sa) => self.visit_struct_access(sa),
            Expression::Grouped(g) => self.visit_grouped(g),
        };
        self.output
            .push_str(&self.format_edge(node_id, right_id, Some("right")));

        node_id
    }

    fn visit_unary(&mut self, unary: &UnaryExpr) -> usize {
        let node_id = self.next_node_id();
        let label = format!("UnaryOp\\n{}", unary.operator);
        let node_str = self.format_node(node_id, &label, &self.colors.operator);
        self.output.push_str(&node_str);

        let operand_id = match unary.operand.as_ref() {
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Identifier(i) => self.visit_identifier(i),
            Expression::Binary(b) => self.visit_binary(b),
            Expression::Unary(u) => self.visit_unary(u),
            Expression::Assignment(a) => self.visit_assignment(a),
            Expression::Call(c) => self.visit_call(c),
            Expression::StructAccess(sa) => self.visit_struct_access(sa),
            Expression::Grouped(g) => self.visit_grouped(g),
        };
        self.output
            .push_str(&self.format_edge(node_id, operand_id, Some("operand")));

        node_id
    }

    fn visit_assignment(&mut self, assignment: &AssignmentExpr) -> usize {
        let node_id = self.next_node_id();
        let label = format!("Assignment\\n{}", assignment.operator);
        let node_str = self.format_node(node_id, &label, &self.colors.operator);
        self.output.push_str(&node_str);

        let target_id = match assignment.target.as_ref() {
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Identifier(i) => self.visit_identifier(i),
            Expression::Binary(b) => self.visit_binary(b),
            Expression::Unary(u) => self.visit_unary(u),
            Expression::Assignment(a) => self.visit_assignment(a),
            Expression::Call(c) => self.visit_call(c),
            Expression::StructAccess(sa) => self.visit_struct_access(sa),
            Expression::Grouped(g) => self.visit_grouped(g),
        };
        self.output
            .push_str(&self.format_edge(node_id, target_id, Some("target")));

        let value_id = match assignment.value.as_ref() {
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Identifier(i) => self.visit_identifier(i),
            Expression::Binary(b) => self.visit_binary(b),
            Expression::Unary(u) => self.visit_unary(u),
            Expression::Assignment(a) => self.visit_assignment(a),
            Expression::Call(c) => self.visit_call(c),
            Expression::StructAccess(sa) => self.visit_struct_access(sa),
            Expression::Grouped(g) => self.visit_grouped(g),
        };
        self.output
            .push_str(&self.format_edge(node_id, value_id, Some("value")));

        node_id
    }

    fn visit_call(&mut self, call: &CallExpr) -> usize {
        let node_id = self.next_node_id();
        let label = format!("Call\\n({} args)", call.arguments.len());
        let node_str = self.format_node(node_id, &label, &self.colors.call);
        self.output.push_str(&node_str);

        let callee_id = match call.callee.as_ref() {
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Identifier(i) => self.visit_identifier(i),
            Expression::Binary(b) => self.visit_binary(b),
            Expression::Unary(u) => self.visit_unary(u),
            Expression::Assignment(a) => self.visit_assignment(a),
            Expression::Call(c) => self.visit_call(c),
            Expression::StructAccess(sa) => self.visit_struct_access(sa),
            Expression::Grouped(g) => self.visit_grouped(g),
        };
        self.output
            .push_str(&self.format_edge(node_id, callee_id, Some("callee")));

        if !call.arguments.is_empty() {
            let args_id = self.next_node_id();
            let args_label = format!("Arguments ({})", call.arguments.len());
            let args_node_str = self.format_node(args_id, &args_label, &self.colors.expression);
            self.output.push_str(&args_node_str);
            self.output
                .push_str(&self.format_edge(node_id, args_id, Some("args")));

            for arg in &call.arguments {
                let arg_id = match arg {
                    Expression::Literal(l) => self.visit_literal(l),
                    Expression::Identifier(i) => self.visit_identifier(i),
                    Expression::Binary(b) => self.visit_binary(b),
                    Expression::Unary(u) => self.visit_unary(u),
                    Expression::Assignment(a) => self.visit_assignment(a),
                    Expression::Call(c) => self.visit_call(c),
                    Expression::StructAccess(sa) => self.visit_struct_access(sa),
                    Expression::Grouped(g) => self.visit_grouped(g),
                };
                self.output
                    .push_str(&self.format_edge(args_id, arg_id, None));
            }
        }

        node_id
    }

    fn visit_struct_access(&mut self, access: &StructAccessExpr) -> usize {
        let node_id = self.next_node_id();
        let label = format!("StructAccess\\n.{}", access.field);
        let node_str = self.format_node(node_id, &label, &self.colors.expression);
        self.output.push_str(&node_str);

        let object_id = match access.object.as_ref() {
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Identifier(i) => self.visit_identifier(i),
            Expression::Binary(b) => self.visit_binary(b),
            Expression::Unary(u) => self.visit_unary(u),
            Expression::Assignment(a) => self.visit_assignment(a),
            Expression::Call(c) => self.visit_call(c),
            Expression::StructAccess(sa) => self.visit_struct_access(sa),
            Expression::Grouped(g) => self.visit_grouped(g),
        };
        self.output
            .push_str(&self.format_edge(node_id, object_id, Some("object")));

        node_id
    }

    fn visit_grouped(&mut self, grouped: &GroupedExpr) -> usize {
        let node_id = self.next_node_id();
        let node_str = self.format_node(node_id, "Grouped", &self.colors.expression);
        self.output.push_str(&node_str);

        let expr_id = match grouped.expr.as_ref() {
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Identifier(i) => self.visit_identifier(i),
            Expression::Binary(b) => self.visit_binary(b),
            Expression::Unary(u) => self.visit_unary(u),
            Expression::Assignment(a) => self.visit_assignment(a),
            Expression::Call(c) => self.visit_call(c),
            Expression::StructAccess(sa) => self.visit_struct_access(sa),
            Expression::Grouped(g) => self.visit_grouped(g),
        };
        self.output
            .push_str(&self.format_edge(node_id, expr_id, Some("expr")));

        node_id
    }
}
