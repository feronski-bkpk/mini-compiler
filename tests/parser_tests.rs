//! Тесты для парсера языка MiniC

#[cfg(test)]
mod tests {
    use minic::compiler;
    use minic::parser::{ParseOutput, Parser, PrettyPrinter, Visitor};

    /// Вспомогательная функция для парсинга строки
    fn parse_string(source: &str) -> ParseOutput {
        println!("DEBUG: Parsing source:\n{}", source);
        let (tokens, lex_errors) = compiler::lexical_analysis(source);

        println!(
            "DEBUG: Tokens count: {}, Lex errors: {}",
            tokens.len(),
            lex_errors.len()
        );
        for (i, token) in tokens.iter().enumerate() {
            if !token.is_eof() {
                println!(
                    "DEBUG: Token {}: {:?} '{}' at {}:{}",
                    i, token.kind, token.lexeme, token.position.line, token.position.column
                );
            }
        }

        if !lex_errors.is_empty() {
            let mut errors = minic::parser::ParseErrors::new();
            for lex_error in lex_errors {
                errors.add(minic::parser::ParseError::from_lexer_error(lex_error));
            }
            return ParseOutput::new(None, errors);
        }

        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        ParseOutput::new(ast, parser.errors().clone())
    }

    /// Проверяет, что парсинг прошел успешно
    fn assert_parses(source: &str) {
        let output = parse_string(source);
        assert!(
            output.is_valid(),
            "Ожидался успешный парсинг, но получены ошибки: {:?}",
            output.errors
        );
    }

    /// Проверяет, что парсинг завершился с ошибкой
    fn assert_parse_error(source: &str) {
        let output = parse_string(source);
        assert!(
            output.has_errors(),
            "Ожидалась ошибка парсинга, но парсинг прошел успешно"
        );
        assert!(output.errors.len() > 0, "Должна быть хотя бы одна ошибка");
    }

    #[test]
    fn test_empty_program() {
        assert_parses("");
    }

    #[test]
    fn test_simple_function() {
        assert_parses("fn main() { return 42; }");
    }

    #[test]
    fn test_function_with_params() {
        assert_parses("fn add(int a, int b) -> int { return a + b; }");
    }

    #[test]
    fn test_function_without_return_type() {
        assert_parses("fn main() { return; }");
    }

    #[test]
    fn test_struct_declaration() {
        assert_parses("struct Point { int x; int y; }");
    }

    #[test]
    fn test_variable_declarations() {
        assert_parses(
            r#"
            fn main() {
                int x = 42;
                float pi = 3.14;
                bool flag = true;
                string msg = "hello";
                int y;
            }
        "#,
        );
    }

    #[test]
    fn test_if_statement() {
        assert_parses(
            r#"
            fn main() {
                if (x > 0) {
                    return 1;
                } else {
                    return 0;
                }
            }
        "#,
        );
    }

    #[test]
    fn test_nested_if() {
        assert_parses(
            r#"
            fn main() {
                if (x > 0) {
                    if (y > 0) {
                        return 1;
                    }
                } else {
                    return 0;
                }
            }
        "#,
        );
    }

    #[test]
    fn test_while_loop() {
        assert_parses(
            r#"
            fn main() {
                int i = 0;
                while (i < 10) {
                    i = i + 1;
                }
            }
        "#,
        );
    }

    #[test]
    fn test_for_loop() {
        assert_parses(
            r#"
            fn main() {
                for (int i = 0; i < 10; i = i + 1) {
                    print(i);
                }
            }
        "#,
        );
    }

    #[test]
    fn test_for_loop_without_init() {
        assert_parses(
            r#"
            fn main() {
                int i = 0;
                for (; i < 10; i = i + 1) {
                    print(i);
                }
            }
        "#,
        );
    }

    #[test]
    fn test_for_loop_without_condition() {
        assert_parses(
            r#"
            fn main() {
                for (int i = 0;; i = i + 1) {
                    if (i > 10) { break; }
                }
            }
        "#,
        );
    }

    #[test]
    fn test_expression_precedence() {
        assert_parses(
            r#"
            fn main() {
                int a = 1 + 2 * 3;
                int b = (1 + 2) * 3;
                bool c = x > 0 && y < 10;
                int d = a = b = 5;
            }
        "#,
        );
    }

    #[test]
    fn test_function_call() {
        assert_parses(
            r#"
            fn main() {
                int result = add(5, 3);
                print("result =", result);
                int x = foo();
            }
        "#,
        );
    }

    #[test]
    fn test_struct_access() {
        assert_parses(
            r#"
            struct Point { int x; int y; }

            fn main() {
                struct Point p;
                p.x = 10;
                p.y = p.x + 5;
            }
        "#,
        );
    }

    #[test]
    fn test_complex_expression() {
        assert_parses(
            r#"
            fn main() {
                int result = (x + y) * (z - 1) / 2 + foo(bar()) - !flag;
            }
        "#,
        );
    }

    // === Тесты на ошибки ===

    #[test]
    fn test_missing_semicolon() {
        assert_parse_error("fn main() { return 42 }");
    }

    #[test]
    fn test_invalid_function_params() {
        assert_parse_error("fn main(int x) { return 42; }");
    }

    #[test]
    fn test_missing_paren() {
        assert_parse_error("fn main() { if x > 0 { return 1; } }");
    }

    #[test]
    fn test_missing_brace() {
        assert_parse_error("fn main() { return 42; ");
    }

    #[test]
    fn test_invalid_expression() {
        assert_parse_error("fn main() { int x = 1 + * 2; }");
    }

    #[test]
    fn test_invalid_assignment() {
        assert_parse_error("fn main() { 5 = x; }");
    }

    #[test]
    fn test_dangling_else() {
        assert_parses(
            r#"
            fn main() {
                if (x > 0)
                    if (y > 0)
                        return 1;
                    else
                        return 0;
            }
        "#,
        );
    }

    #[test]
    fn test_golden_factorial() {
        let source = r#"
            fn factorial(int n) -> int {
                if (n <= 1) {
                    return 1;
                }
                return n * factorial(n - 1);
            }

            fn main() {
                int result = factorial(5);
                return result;
            }
        "#;

        let output = parse_string(source);
        assert!(output.is_valid(), "Ожидался успешный парсинг factorial");

        if let Some(ast) = output.ast {
            let mut printer = PrettyPrinter::new();
            let ast_text = printer.format_program(&ast);
            assert!(!ast_text.is_empty(), "AST не должно быть пустым");
            println!("AST:\n{}", ast_text);
        } else {
            panic!("AST не был построен");
        }
    }

    #[test]
    fn test_visitor_pattern() {
        let source = r#"
            struct Point { int x; int y; }

            fn main() {
                struct Point p;
                p.x = 10;
                p.y = 20;
                return p.x + p.y;
            }
        "#;

        let output = parse_string(source);
        assert!(output.is_valid(), "Ожидался успешный парсинг для visitor");

        let ast = output.ast.expect("AST должен быть построен");

        struct CountingVisitor {
            node_count: usize,
        }

        impl CountingVisitor {
            fn new() -> Self {
                Self { node_count: 0 }
            }
        }

        impl Visitor<()> for CountingVisitor {
            fn visit_program(&mut self, program: &minic::parser::Program) {
                self.node_count += 1;
                for decl in &program.declarations {
                    match decl {
                        minic::parser::Declaration::Function(f) => self.visit_function_decl(f),
                        minic::parser::Declaration::Struct(s) => self.visit_struct_decl(s),
                        minic::parser::Declaration::Variable(v) => self.visit_var_decl(v),
                    }
                }
            }

            fn visit_function_decl(&mut self, func: &minic::parser::FunctionDecl) {
                self.node_count += 1;
                for param in &func.parameters {
                    self.visit_param(param);
                }
                self.visit_block(&func.body);
            }

            fn visit_struct_decl(&mut self, struct_decl: &minic::parser::StructDecl) {
                self.node_count += 1;
                for field in &struct_decl.fields {
                    self.visit_var_decl(field);
                }
            }

            fn visit_var_decl(&mut self, var_decl: &minic::parser::VarDecl) {
                self.node_count += 1;
                if let Some(init) = &var_decl.initializer {
                    match init.as_ref() {
                        minic::parser::Expression::Literal(l) => self.visit_literal(l),
                        minic::parser::Expression::Identifier(i) => self.visit_identifier(i),
                        minic::parser::Expression::Binary(b) => self.visit_binary(b),
                        minic::parser::Expression::Unary(u) => self.visit_unary(u),
                        minic::parser::Expression::Assignment(a) => self.visit_assignment(a),
                        minic::parser::Expression::Call(c) => self.visit_call(c),
                        minic::parser::Expression::StructAccess(sa) => self.visit_struct_access(sa),
                        minic::parser::Expression::Grouped(g) => self.visit_grouped(g),
                    }
                }
            }

            fn visit_param(&mut self, _param: &minic::parser::Param) {
                self.node_count += 1;
            }

            fn visit_block(&mut self, block: &minic::parser::BlockStmt) {
                self.node_count += 1;
                for stmt in &block.statements {
                    match stmt {
                        minic::parser::Statement::VariableDecl(v) => self.visit_var_decl(v),
                        minic::parser::Statement::Expression(e) => self.visit_expr_stmt(e),
                        minic::parser::Statement::If(i) => self.visit_if_stmt(i),
                        minic::parser::Statement::While(w) => self.visit_while_stmt(w),
                        minic::parser::Statement::For(f) => self.visit_for_stmt(f),
                        minic::parser::Statement::Return(r) => self.visit_return_stmt(r),
                        minic::parser::Statement::Block(b) => self.visit_block(b),
                        minic::parser::Statement::Empty(e) => self.visit_empty_stmt(e),
                    }
                }
            }

            fn visit_if_stmt(&mut self, if_stmt: &minic::parser::IfStmt) {
                self.node_count += 1;
                match if_stmt.condition.as_ref() {
                    minic::parser::Expression::Literal(l) => self.visit_literal(l),
                    minic::parser::Expression::Identifier(i) => self.visit_identifier(i),
                    minic::parser::Expression::Binary(b) => self.visit_binary(b),
                    minic::parser::Expression::Unary(u) => self.visit_unary(u),
                    minic::parser::Expression::Assignment(a) => self.visit_assignment(a),
                    minic::parser::Expression::Call(c) => self.visit_call(c),
                    minic::parser::Expression::StructAccess(sa) => self.visit_struct_access(sa),
                    minic::parser::Expression::Grouped(g) => self.visit_grouped(g),
                }
                match if_stmt.then_branch.as_ref() {
                    minic::parser::Statement::VariableDecl(v) => self.visit_var_decl(v),
                    minic::parser::Statement::Expression(e) => self.visit_expr_stmt(e),
                    minic::parser::Statement::If(i) => self.visit_if_stmt(i),
                    minic::parser::Statement::While(w) => self.visit_while_stmt(w),
                    minic::parser::Statement::For(f) => self.visit_for_stmt(f),
                    minic::parser::Statement::Return(r) => self.visit_return_stmt(r),
                    minic::parser::Statement::Block(b) => self.visit_block(b),
                    minic::parser::Statement::Empty(e) => self.visit_empty_stmt(e),
                }
                if let Some(else_branch) = &if_stmt.else_branch {
                    match else_branch.as_ref() {
                        minic::parser::Statement::VariableDecl(v) => self.visit_var_decl(v),
                        minic::parser::Statement::Expression(e) => self.visit_expr_stmt(e),
                        minic::parser::Statement::If(i) => self.visit_if_stmt(i),
                        minic::parser::Statement::While(w) => self.visit_while_stmt(w),
                        minic::parser::Statement::For(f) => self.visit_for_stmt(f),
                        minic::parser::Statement::Return(r) => self.visit_return_stmt(r),
                        minic::parser::Statement::Block(b) => self.visit_block(b),
                        minic::parser::Statement::Empty(e) => self.visit_empty_stmt(e),
                    }
                }
            }

            fn visit_while_stmt(&mut self, while_stmt: &minic::parser::WhileStmt) {
                self.node_count += 1;
                match while_stmt.condition.as_ref() {
                    minic::parser::Expression::Literal(l) => self.visit_literal(l),
                    minic::parser::Expression::Identifier(i) => self.visit_identifier(i),
                    minic::parser::Expression::Binary(b) => self.visit_binary(b),
                    minic::parser::Expression::Unary(u) => self.visit_unary(u),
                    minic::parser::Expression::Assignment(a) => self.visit_assignment(a),
                    minic::parser::Expression::Call(c) => self.visit_call(c),
                    minic::parser::Expression::StructAccess(sa) => self.visit_struct_access(sa),
                    minic::parser::Expression::Grouped(g) => self.visit_grouped(g),
                }
                match while_stmt.body.as_ref() {
                    minic::parser::Statement::VariableDecl(v) => self.visit_var_decl(v),
                    minic::parser::Statement::Expression(e) => self.visit_expr_stmt(e),
                    minic::parser::Statement::If(i) => self.visit_if_stmt(i),
                    minic::parser::Statement::While(w) => self.visit_while_stmt(w),
                    minic::parser::Statement::For(f) => self.visit_for_stmt(f),
                    minic::parser::Statement::Return(r) => self.visit_return_stmt(r),
                    minic::parser::Statement::Block(b) => self.visit_block(b),
                    minic::parser::Statement::Empty(e) => self.visit_empty_stmt(e),
                }
            }

            fn visit_for_stmt(&mut self, for_stmt: &minic::parser::ForStmt) {
                self.node_count += 1;
                if let Some(init) = &for_stmt.init {
                    match init.as_ref() {
                        minic::parser::Statement::VariableDecl(v) => self.visit_var_decl(v),
                        minic::parser::Statement::Expression(e) => self.visit_expr_stmt(e),
                        minic::parser::Statement::If(i) => self.visit_if_stmt(i),
                        minic::parser::Statement::While(w) => self.visit_while_stmt(w),
                        minic::parser::Statement::For(f) => self.visit_for_stmt(f),
                        minic::parser::Statement::Return(r) => self.visit_return_stmt(r),
                        minic::parser::Statement::Block(b) => self.visit_block(b),
                        minic::parser::Statement::Empty(e) => self.visit_empty_stmt(e),
                    }
                }
                if let Some(condition) = &for_stmt.condition {
                    match condition.as_ref() {
                        minic::parser::Expression::Literal(l) => self.visit_literal(l),
                        minic::parser::Expression::Identifier(i) => self.visit_identifier(i),
                        minic::parser::Expression::Binary(b) => self.visit_binary(b),
                        minic::parser::Expression::Unary(u) => self.visit_unary(u),
                        minic::parser::Expression::Assignment(a) => self.visit_assignment(a),
                        minic::parser::Expression::Call(c) => self.visit_call(c),
                        minic::parser::Expression::StructAccess(sa) => self.visit_struct_access(sa),
                        minic::parser::Expression::Grouped(g) => self.visit_grouped(g),
                    }
                }
                if let Some(update) = &for_stmt.update {
                    match update.as_ref() {
                        minic::parser::Expression::Literal(l) => self.visit_literal(l),
                        minic::parser::Expression::Identifier(i) => self.visit_identifier(i),
                        minic::parser::Expression::Binary(b) => self.visit_binary(b),
                        minic::parser::Expression::Unary(u) => self.visit_unary(u),
                        minic::parser::Expression::Assignment(a) => self.visit_assignment(a),
                        minic::parser::Expression::Call(c) => self.visit_call(c),
                        minic::parser::Expression::StructAccess(sa) => self.visit_struct_access(sa),
                        minic::parser::Expression::Grouped(g) => self.visit_grouped(g),
                    }
                }
                match for_stmt.body.as_ref() {
                    minic::parser::Statement::VariableDecl(v) => self.visit_var_decl(v),
                    minic::parser::Statement::Expression(e) => self.visit_expr_stmt(e),
                    minic::parser::Statement::If(i) => self.visit_if_stmt(i),
                    minic::parser::Statement::While(w) => self.visit_while_stmt(w),
                    minic::parser::Statement::For(f) => self.visit_for_stmt(f),
                    minic::parser::Statement::Return(r) => self.visit_return_stmt(r),
                    minic::parser::Statement::Block(b) => self.visit_block(b),
                    minic::parser::Statement::Empty(e) => self.visit_empty_stmt(e),
                }
            }

            fn visit_return_stmt(&mut self, return_stmt: &minic::parser::ReturnStmt) {
                self.node_count += 1;
                if let Some(value) = &return_stmt.value {
                    match value.as_ref() {
                        minic::parser::Expression::Literal(l) => self.visit_literal(l),
                        minic::parser::Expression::Identifier(i) => self.visit_identifier(i),
                        minic::parser::Expression::Binary(b) => self.visit_binary(b),
                        minic::parser::Expression::Unary(u) => self.visit_unary(u),
                        minic::parser::Expression::Assignment(a) => self.visit_assignment(a),
                        minic::parser::Expression::Call(c) => self.visit_call(c),
                        minic::parser::Expression::StructAccess(sa) => self.visit_struct_access(sa),
                        minic::parser::Expression::Grouped(g) => self.visit_grouped(g),
                    }
                }
            }

            fn visit_expr_stmt(&mut self, expr_stmt: &minic::parser::ExprStmt) {
                self.node_count += 1;
                match expr_stmt.expr.as_ref() {
                    minic::parser::Expression::Literal(l) => self.visit_literal(l),
                    minic::parser::Expression::Identifier(i) => self.visit_identifier(i),
                    minic::parser::Expression::Binary(b) => self.visit_binary(b),
                    minic::parser::Expression::Unary(u) => self.visit_unary(u),
                    minic::parser::Expression::Assignment(a) => self.visit_assignment(a),
                    minic::parser::Expression::Call(c) => self.visit_call(c),
                    minic::parser::Expression::StructAccess(sa) => self.visit_struct_access(sa),
                    minic::parser::Expression::Grouped(g) => self.visit_grouped(g),
                }
            }

            fn visit_empty_stmt(&mut self, _empty_stmt: &minic::parser::EmptyStmt) {
                self.node_count += 1;
            }

            fn visit_literal(&mut self, _literal: &minic::parser::Literal) {
                self.node_count += 1;
            }

            fn visit_identifier(&mut self, _identifier: &minic::parser::IdentifierExpr) {
                self.node_count += 1;
            }

            fn visit_binary(&mut self, binary: &minic::parser::BinaryExpr) {
                self.node_count += 1;
                match binary.left.as_ref() {
                    minic::parser::Expression::Literal(l) => self.visit_literal(l),
                    minic::parser::Expression::Identifier(i) => self.visit_identifier(i),
                    minic::parser::Expression::Binary(b) => self.visit_binary(b),
                    minic::parser::Expression::Unary(u) => self.visit_unary(u),
                    minic::parser::Expression::Assignment(a) => self.visit_assignment(a),
                    minic::parser::Expression::Call(c) => self.visit_call(c),
                    minic::parser::Expression::StructAccess(sa) => self.visit_struct_access(sa),
                    minic::parser::Expression::Grouped(g) => self.visit_grouped(g),
                }
                match binary.right.as_ref() {
                    minic::parser::Expression::Literal(l) => self.visit_literal(l),
                    minic::parser::Expression::Identifier(i) => self.visit_identifier(i),
                    minic::parser::Expression::Binary(b) => self.visit_binary(b),
                    minic::parser::Expression::Unary(u) => self.visit_unary(u),
                    minic::parser::Expression::Assignment(a) => self.visit_assignment(a),
                    minic::parser::Expression::Call(c) => self.visit_call(c),
                    minic::parser::Expression::StructAccess(sa) => self.visit_struct_access(sa),
                    minic::parser::Expression::Grouped(g) => self.visit_grouped(g),
                }
            }

            fn visit_unary(&mut self, unary: &minic::parser::UnaryExpr) {
                self.node_count += 1;
                match unary.operand.as_ref() {
                    minic::parser::Expression::Literal(l) => self.visit_literal(l),
                    minic::parser::Expression::Identifier(i) => self.visit_identifier(i),
                    minic::parser::Expression::Binary(b) => self.visit_binary(b),
                    minic::parser::Expression::Unary(u) => self.visit_unary(u),
                    minic::parser::Expression::Assignment(a) => self.visit_assignment(a),
                    minic::parser::Expression::Call(c) => self.visit_call(c),
                    minic::parser::Expression::StructAccess(sa) => self.visit_struct_access(sa),
                    minic::parser::Expression::Grouped(g) => self.visit_grouped(g),
                }
            }

            fn visit_assignment(&mut self, assignment: &minic::parser::AssignmentExpr) {
                self.node_count += 1;
                match assignment.target.as_ref() {
                    minic::parser::Expression::Literal(l) => self.visit_literal(l),
                    minic::parser::Expression::Identifier(i) => self.visit_identifier(i),
                    minic::parser::Expression::Binary(b) => self.visit_binary(b),
                    minic::parser::Expression::Unary(u) => self.visit_unary(u),
                    minic::parser::Expression::Assignment(a) => self.visit_assignment(a),
                    minic::parser::Expression::Call(c) => self.visit_call(c),
                    minic::parser::Expression::StructAccess(sa) => self.visit_struct_access(sa),
                    minic::parser::Expression::Grouped(g) => self.visit_grouped(g),
                }
                match assignment.value.as_ref() {
                    minic::parser::Expression::Literal(l) => self.visit_literal(l),
                    minic::parser::Expression::Identifier(i) => self.visit_identifier(i),
                    minic::parser::Expression::Binary(b) => self.visit_binary(b),
                    minic::parser::Expression::Unary(u) => self.visit_unary(u),
                    minic::parser::Expression::Assignment(a) => self.visit_assignment(a),
                    minic::parser::Expression::Call(c) => self.visit_call(c),
                    minic::parser::Expression::StructAccess(sa) => self.visit_struct_access(sa),
                    minic::parser::Expression::Grouped(g) => self.visit_grouped(g),
                }
            }

            fn visit_call(&mut self, call: &minic::parser::CallExpr) {
                self.node_count += 1;
                match call.callee.as_ref() {
                    minic::parser::Expression::Literal(l) => self.visit_literal(l),
                    minic::parser::Expression::Identifier(i) => self.visit_identifier(i),
                    minic::parser::Expression::Binary(b) => self.visit_binary(b),
                    minic::parser::Expression::Unary(u) => self.visit_unary(u),
                    minic::parser::Expression::Assignment(a) => self.visit_assignment(a),
                    minic::parser::Expression::Call(c) => self.visit_call(c),
                    minic::parser::Expression::StructAccess(sa) => self.visit_struct_access(sa),
                    minic::parser::Expression::Grouped(g) => self.visit_grouped(g),
                }
                for arg in &call.arguments {
                    match arg {
                        minic::parser::Expression::Literal(l) => self.visit_literal(l),
                        minic::parser::Expression::Identifier(i) => self.visit_identifier(i),
                        minic::parser::Expression::Binary(b) => self.visit_binary(b),
                        minic::parser::Expression::Unary(u) => self.visit_unary(u),
                        minic::parser::Expression::Assignment(a) => self.visit_assignment(a),
                        minic::parser::Expression::Call(c) => self.visit_call(c),
                        minic::parser::Expression::StructAccess(sa) => self.visit_struct_access(sa),
                        minic::parser::Expression::Grouped(g) => self.visit_grouped(g),
                    }
                }
            }

            fn visit_struct_access(&mut self, access: &minic::parser::StructAccessExpr) {
                self.node_count += 1;
                match access.object.as_ref() {
                    minic::parser::Expression::Literal(l) => self.visit_literal(l),
                    minic::parser::Expression::Identifier(i) => self.visit_identifier(i),
                    minic::parser::Expression::Binary(b) => self.visit_binary(b),
                    minic::parser::Expression::Unary(u) => self.visit_unary(u),
                    minic::parser::Expression::Assignment(a) => self.visit_assignment(a),
                    minic::parser::Expression::Call(c) => self.visit_call(c),
                    minic::parser::Expression::StructAccess(sa) => self.visit_struct_access(sa),
                    minic::parser::Expression::Grouped(g) => self.visit_grouped(g),
                }
            }

            fn visit_grouped(&mut self, grouped: &minic::parser::GroupedExpr) {
                self.node_count += 1;
                match grouped.expr.as_ref() {
                    minic::parser::Expression::Literal(l) => self.visit_literal(l),
                    minic::parser::Expression::Identifier(i) => self.visit_identifier(i),
                    minic::parser::Expression::Binary(b) => self.visit_binary(b),
                    minic::parser::Expression::Unary(u) => self.visit_unary(u),
                    minic::parser::Expression::Assignment(a) => self.visit_assignment(a),
                    minic::parser::Expression::Call(c) => self.visit_call(c),
                    minic::parser::Expression::StructAccess(sa) => self.visit_struct_access(sa),
                    minic::parser::Expression::Grouped(g) => self.visit_grouped(g),
                }
            }
        }

        let mut visitor = CountingVisitor::new();
        visitor.visit_program(&ast);

        assert!(
            visitor.node_count > 0,
            "Visitor должен найти хотя бы один узел"
        );
        println!("Найдено узлов: {}", visitor.node_count);
        assert!(
            visitor.node_count >= 20,
            "Найдено только {} узлов, ожидалось минимум 20",
            visitor.node_count
        );
    }
}
