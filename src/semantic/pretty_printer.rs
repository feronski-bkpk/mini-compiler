//! Pretty printer для декорированного AST

use crate::parser::ast::*;
use crate::semantic::symbol_table::SymbolTable;

/// Форматтер для декорированного AST
pub struct DecoratedAstPrinter {
    indent_level: usize,
    show_types: bool,
    show_symbols: bool,
}

impl DecoratedAstPrinter {
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            show_types: true,
            show_symbols: false,
        }
    }

    pub fn with_types(mut self, show: bool) -> Self {
        self.show_types = show;
        self
    }
    pub fn with_symbols(mut self, show: bool) -> Self {
        self.show_symbols = show;
        self
    }

    pub fn format_program(&mut self, program: &Program, symbol_table: &SymbolTable) -> String {
        let mut output = String::new();
        output.push_str("Program [global scope]:\n");
        self.indent_level += 1;
        output.push_str(&self.format_indent());
        output.push_str("Symbol Table:\n");
        self.indent_level += 1;
        for symbol in symbol_table.global_symbols() {
            output.push_str(&self.format_symbol(symbol));
        }
        self.indent_level -= 1;
        output.push('\n');
        for decl in &program.declarations {
            output.push_str(&self.format_declaration(decl, symbol_table));
        }
        self.indent_level -= 1;
        output
    }

    fn format_symbol(&self, symbol: &crate::semantic::symbol_table::Symbol) -> String {
        let mut output = String::new();
        output.push_str(&self.format_indent());
        output.push_str(&format!("{}: ", symbol.name));
        match symbol.kind {
            crate::semantic::symbol_table::SymbolKind::Variable => {
                output.push_str(&format!("{} переменная", symbol.typ))
            }
            crate::semantic::symbol_table::SymbolKind::Parameter => {
                output.push_str(&format!("{} параметр", symbol.typ))
            }
            crate::semantic::symbol_table::SymbolKind::Function => {
                output.push_str(&format!("{} функция", symbol.typ))
            }
            crate::semantic::symbol_table::SymbolKind::Struct => {
                output.push_str(&format!("struct {}", symbol.name));
                if let Some(fields) = &symbol.fields {
                    output.push_str(" {\n");
                    for (fnm, fty) in fields {
                        output.push_str(&self.format_indent());
                        output.push_str(&format!("    {}: {}\n", fnm, fty));
                    }
                    output.push_str(&self.format_indent());
                    output.push_str("  }");
                }
            }
            crate::semantic::symbol_table::SymbolKind::Field => {
                output.push_str(&format!("{} поле", symbol.typ))
            }
        }
        output.push('\n');
        output
    }

    fn format_declaration(&mut self, decl: &Declaration, symbol_table: &SymbolTable) -> String {
        match decl {
            Declaration::Function(func) => self.format_function(func, symbol_table),
            Declaration::Struct(sd) => self.format_struct(sd),
            Declaration::Variable(var) => self.format_variable(var, true, symbol_table),
        }
    }

    fn format_function(&mut self, func: &FunctionDecl, symbol_table: &SymbolTable) -> String {
        let mut output = String::new();
        output.push_str(&self.format_indent());
        output.push_str(&format!(
            "FunctionDecl: {} -> {} [line {}]:\n",
            func.name, func.return_type, func.node.line
        ));
        self.indent_level += 1;
        if !func.parameters.is_empty() {
            output.push_str(&self.format_indent());
            output.push_str("Parameters:\n");
            self.indent_level += 1;
            for param in &func.parameters {
                output.push_str(&self.format_indent());
                output.push_str(&format!("- {}: {}\n", param.name, param.param_type));
            }
            self.indent_level -= 1;
        }
        output.push_str(&self.format_indent());
        output.push_str("Body [type checked]:\n");
        self.indent_level += 1;
        output.push_str(&self.format_block(&func.body, symbol_table));
        self.indent_level -= 2;
        output
    }

    fn format_struct(&mut self, struct_decl: &StructDecl) -> String {
        let mut output = String::new();
        output.push_str(&self.format_indent());
        output.push_str(&format!(
            "StructDecl: {} [line {}]:\n",
            struct_decl.name, struct_decl.node.line
        ));
        self.indent_level += 1;
        output.push_str(&self.format_indent());
        output.push_str("Fields:\n");
        self.indent_level += 1;
        for field in &struct_decl.fields {
            output.push_str(&self.format_indent());
            output.push_str(&format!("- {}: {}\n", field.name, field.var_type));
        }
        self.indent_level -= 2;
        output
    }

    fn format_block(&mut self, block: &BlockStmt, symbol_table: &SymbolTable) -> String {
        let mut output = String::new();
        output.push_str(&self.format_indent());
        output.push_str(&format!("Block [line {}]:\n", block.node.line));
        self.indent_level += 1;
        for stmt in &block.statements {
            output.push_str(&self.format_statement(stmt, symbol_table));
        }
        self.indent_level -= 1;
        output
    }

    fn format_statement(&mut self, stmt: &Statement, symbol_table: &SymbolTable) -> String {
        match stmt {
            Statement::VariableDecl(var) => self.format_variable(var, false, symbol_table),
            Statement::Expression(es) => self.format_expression_stmt(es),
            Statement::If(is) => self.format_if(is, symbol_table),
            Statement::While(ws) => self.format_while(ws, symbol_table),
            Statement::For(fs) => self.format_for(fs, symbol_table),
            Statement::Return(rs) => self.format_return(rs),
            Statement::Block(bs) => self.format_block(bs, symbol_table),
            Statement::Empty(_) => format!("{}EmptyStmt\n", self.format_indent()),
            Statement::Break(_) => format!("{}Break\n", self.format_indent()),
            Statement::Continue(_) => format!("{}Continue\n", self.format_indent()),
            Statement::Switch(ss) => {
                let mut out = String::new();
                out.push_str(&self.format_indent());
                out.push_str(&format!("SwitchStmt [line {}]:\n", ss.node.line));
                self.indent_level += 1;
                out.push_str(&self.format_indent());
                out.push_str(&format!(
                    "Expression: {}\n",
                    self.format_expression(&ss.expression)
                ));
                for case in &ss.cases {
                    out.push_str(&self.format_indent());
                    out.push_str(&format!("Case {}:\n", case.value.value));
                    self.indent_level += 1;
                    out.push_str(&self.format_statement(&case.body, symbol_table));
                    self.indent_level -= 1;
                }
                if let Some(default) = &ss.default {
                    out.push_str(&self.format_indent());
                    out.push_str("Default:\n");
                    self.indent_level += 1;
                    out.push_str(&self.format_statement(default, symbol_table));
                    self.indent_level -= 1;
                }
                self.indent_level -= 1;
                out
            }
        }
    }

    fn format_variable(
        &mut self,
        var: &VarDecl,
        _is_global: bool,
        symbol_table: &SymbolTable,
    ) -> String {
        let mut output = String::new();
        output.push_str(&self.format_indent());
        let inferred_type = symbol_table.lookup(&var.name).map(|s| &s.typ);
        match &var.var_type {
            crate::parser::ast::Type::Inferred => {
                if let Some(typ) = inferred_type {
                    output.push_str(&format!("VarDecl: var {} = ", var.name));
                    if let Some(init) = &var.initializer {
                        output.push_str(&self.format_expression(init));
                    }
                    if self.show_types {
                        output.push_str(&format!(" [выведен: {}]", typ));
                    }
                } else {
                    output.push_str(&format!("VarDecl: var {}", var.name));
                    if let Some(init) = &var.initializer {
                        output.push_str(" = ");
                        output.push_str(&self.format_expression(init));
                    }
                    if self.show_types {
                        output.push_str(" [type: var]");
                    }
                }
            }
            _ => {
                output.push_str(&format!("VarDecl: {} {}", var.var_type, var.name));
                if let Some(init) = &var.initializer {
                    output.push_str(" = ");
                    output.push_str(&self.format_expression(init));
                }
                if self.show_types {
                    output.push_str(&format!(" [type: {}]", var.var_type));
                }
            }
        }
        output.push('\n');
        output
    }

    fn format_expression_stmt(&mut self, expr_stmt: &ExprStmt) -> String {
        format!(
            "{}Expr: {}\n",
            self.format_indent(),
            self.format_expression(&expr_stmt.expr)
        )
    }

    fn format_if(&mut self, if_stmt: &IfStmt, symbol_table: &SymbolTable) -> String {
        let mut output = String::new();
        output.push_str(&self.format_indent());
        output.push_str(&format!("IfStmt [line {}]:\n", if_stmt.node.line));
        self.indent_level += 1;
        output.push_str(&self.format_indent());
        output.push_str(&format!(
            "Condition: {}\n",
            self.format_expression(&if_stmt.condition)
        ));
        output.push_str(&self.format_indent());
        output.push_str("Then branch:\n");
        self.indent_level += 1;
        output.push_str(&self.format_statement(&if_stmt.then_branch, symbol_table));
        self.indent_level -= 1;
        if let Some(eb) = &if_stmt.else_branch {
            output.push_str(&self.format_indent());
            output.push_str("Else branch:\n");
            self.indent_level += 1;
            output.push_str(&self.format_statement(eb, symbol_table));
            self.indent_level -= 1;
        }
        self.indent_level -= 1;
        output
    }

    fn format_while(&mut self, while_stmt: &WhileStmt, symbol_table: &SymbolTable) -> String {
        let mut output = String::new();
        output.push_str(&self.format_indent());
        output.push_str(&format!("WhileStmt [line {}]:\n", while_stmt.node.line));
        self.indent_level += 1;
        output.push_str(&self.format_indent());
        output.push_str(&format!(
            "Condition: {}\n",
            self.format_expression(&while_stmt.condition)
        ));
        output.push_str(&self.format_indent());
        output.push_str("Body:\n");
        self.indent_level += 1;
        output.push_str(&self.format_statement(&while_stmt.body, symbol_table));
        self.indent_level -= 2;
        output
    }

    fn format_for(&mut self, for_stmt: &ForStmt, symbol_table: &SymbolTable) -> String {
        let mut output = String::new();
        output.push_str(&self.format_indent());
        output.push_str(&format!("ForStmt [line {}]:\n", for_stmt.node.line));
        self.indent_level += 1;
        if let Some(init) = &for_stmt.init {
            output.push_str(&self.format_indent());
            output.push_str(&format!(
                "Init: {}",
                self.format_statement(init, symbol_table)
            ));
        }
        if let Some(cond) = &for_stmt.condition {
            output.push_str(&self.format_indent());
            output.push_str(&format!("Condition: {}\n", self.format_expression(cond)));
        }
        if let Some(upd) = &for_stmt.update {
            output.push_str(&self.format_indent());
            output.push_str(&format!("Update: {}\n", self.format_expression(upd)));
        }
        output.push_str(&self.format_indent());
        output.push_str("Body:\n");
        self.indent_level += 1;
        output.push_str(&self.format_statement(&for_stmt.body, symbol_table));
        self.indent_level -= 2;
        output
    }

    fn format_return(&mut self, return_stmt: &ReturnStmt) -> String {
        let mut output = String::new();
        output.push_str(&self.format_indent());
        if let Some(val) = &return_stmt.value {
            output.push_str(&format!("Return: {}\n", self.format_expression(val)));
        } else {
            output.push_str("Return\n");
        }
        output
    }

    fn format_expression(&mut self, expr: &Expression) -> String {
        match expr {
            Expression::Literal(lit) => format!("{}", lit.value),
            Expression::Identifier(ident) => ident.name.clone(),
            Expression::Binary(bin) => format!(
                "({} {} {})",
                self.format_expression(&bin.left),
                bin.operator,
                self.format_expression(&bin.right)
            ),
            Expression::Unary(un) => {
                format!("({}{})", un.operator, self.format_expression(&un.operand))
            }
            Expression::Assignment(a) => format!(
                "({} {} {})",
                self.format_expression(&a.target),
                a.operator,
                self.format_expression(&a.value)
            ),
            Expression::Call(c) => {
                let args: Vec<String> = c
                    .arguments
                    .iter()
                    .map(|a| self.format_expression(a))
                    .collect();
                format!("{}({})", self.format_expression(&c.callee), args.join(", "))
            }
            Expression::StructAccess(sa) => {
                format!("{}.{}", self.format_expression(&sa.object), sa.field)
            }
            Expression::ArrayAccess(aa) => format!(
                "{}[{}]",
                self.format_expression(&aa.array),
                self.format_expression(&aa.index)
            ),
            Expression::Grouped(g) => format!("({})", self.format_expression(&g.expr)),
        }
    }

    fn format_indent(&self) -> String {
        "  ".repeat(self.indent_level)
    }
}

impl Default for DecoratedAstPrinter {
    fn default() -> Self {
        Self::new()
    }
}
