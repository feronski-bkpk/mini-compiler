//! Тесты для парсера языка MiniC

#[cfg(test)]
mod tests {
    use minic::compiler;
    use minic::parser::{ParseErrorKind, ParseOutput, Parser, PrettyPrinter, Visitor};

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

    /// Проверяет, что парсинг прошел успешно (может быть с ошибками, но AST построен)
    fn assert_parses(source: &str) {
        let output = parse_string(source);

        assert!(
            output.ast.is_some(),
            "Ожидалось построение AST, но получено None"
        );

        if output.has_errors() {
            println!("Найдены ошибки (но AST построен):");
            for error in &output.errors.errors {
                println!("  {}", error);
            }
        }
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
    fn test_increment_decrement() {
        let source = r#"
        fn main() {
            int x = 5;
            x++;
            ++x;
            x--;
            --x;
            int y = x++ + ++z;
        }
    "#;
        assert_parses(source);
    }

    #[test]
    fn test_increment_in_expression() {
        let source = r#"
        fn main() {
            int a = 5;
            int b = a++ + 10;
            int c = ++a * 2;
        }
    "#;
        assert_parses(source);
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

    #[test]
    fn test_missing_semicolon() {
        let source = "fn main() { return 42 }";
        let output = parse_string(source);

        println!("DEBUG: Missing semicolon test");
        println!("DEBUG: AST построен: {:?}", output.ast.is_some());
        println!("DEBUG: Количество ошибок: {}", output.errors.len());
        println!("DEBUG: Ошибки: {:?}", output.errors.errors);

        assert!(output.has_errors(), "Должны быть ошибки");
        assert!(output.errors.len() > 0, "Количество ошибок должно быть > 0");

        let has_semicolon_error = output
            .errors
            .errors
            .iter()
            .any(|e| matches!(e.kind, ParseErrorKind::MissingSemicolon));
        assert!(
            has_semicolon_error,
            "Должна быть ошибка о пропущенной точке с запятой"
        );

        assert!(
            output.ast.is_some(),
            "AST должен быть построен даже при ошибке"
        );

        println!("Тест test_missing_semicolon прошел успешно!");
    }

    #[test]
    fn test_invalid_function_params() {
        let source = "fn main(int x) { return 42; }";
        let output = parse_string(source);

        assert!(output.has_errors(), "Должны быть ошибки");

        if output.ast.is_some() {
            println!("AST построен несмотря на ошибку в параметрах");
        }
    }

    #[test]
    fn test_missing_paren() {
        let source = "fn main() { if (x > 0 { return 1; } }";
        let output = parse_string(source);

        println!("DEBUG: Missing paren test");
        println!("DEBUG: AST построен: {:?}", output.ast.is_some());
        println!("DEBUG: Количество ошибок: {}", output.errors.len());
        println!("DEBUG: Ошибки: {:?}", output.errors.errors);

        assert!(output.has_errors(), "Должны быть ошибки");
        assert!(output.errors.len() > 0, "Количество ошибок должно быть > 0");

        let has_paren_error = output
            .errors
            .errors
            .iter()
            .any(|e| matches!(e.kind, ParseErrorKind::MissingCloseParen));
        assert!(has_paren_error, "Должна быть ошибка о пропущенной скобке");

        println!("Тест test_missing_paren прошел успешно!");
    }

    #[test]
    fn test_missing_brace() {
        let source = "fn main() { return 42; ";
        let output = parse_string(source);

        println!("DEBUG: Missing brace test");
        println!("DEBUG: AST построен: {:?}", output.ast.is_some());
        println!("DEBUG: Количество ошибок: {}", output.errors.len());
        println!("DEBUG: Ошибки: {:#?}", output.errors.errors);
        println!("DEBUG: Метрики: {:?}", output.errors.metrics);

        assert!(output.has_errors(), "Должны быть ошибки");
        assert!(output.errors.len() > 0, "Количество ошибок должно быть > 0");

        let missing_brace = output
            .errors
            .errors
            .iter()
            .any(|e| matches!(e.kind, ParseErrorKind::MissingCloseBrace));
        assert!(
            missing_brace,
            "Должна быть ошибка о пропущенной закрывающей скобке"
        );

        println!("Тест test_missing_brace прошел успешно!");
    }

    #[test]
    fn test_invalid_expression() {
        let source = "fn main() { int x = 1 + * 2; }";
        let output = parse_string(source);

        assert!(output.has_errors(), "Должны быть ошибки");
    }

    #[test]
    fn test_invalid_assignment() {
        let source = "fn main() { 5 = x; }";
        let output = parse_string(source);

        assert!(output.has_errors(), "Должны быть ошибки");
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

        assert!(output.ast.is_some(), "AST должен быть построен");

        if output.has_errors() {
            println!("Найдены ошибки, но AST построен:");
            for error in &output.errors.errors {
                println!("  {}", error);
            }
        } else {
            println!("Парсинг без ошибок");
        }

        if let Some(ast) = &output.ast {
            let mut printer = PrettyPrinter::new();
            let ast_text = printer.format_program(ast);
            assert!(!ast_text.is_empty(), "AST не должно быть пустым");
            println!("AST:\n{}", ast_text);

            assert!(
                ast_text.contains("factorial"),
                "Должна быть функция factorial"
            );
            assert!(ast_text.contains("main"), "Должна быть функция main");
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

        assert!(output.ast.is_some(), "AST должен быть построен");

        if output.has_errors() {
            println!("Найдены ошибки, но AST построен:");
            for error in &output.errors.errors {
                println!("  {}", error);
            }
        }

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
                        minic::parser::Declaration::Function(f) => {
                            self.node_count += 1;
                            for _param in &f.parameters {
                                self.node_count += 1;
                            }
                            self.visit_block(&f.body);
                        }
                        minic::parser::Declaration::Struct(s) => {
                            self.node_count += 1;
                            for _field in &s.fields {
                                self.node_count += 1;
                            }
                        }
                        minic::parser::Declaration::Variable(_v) => {
                            self.node_count += 1;
                        }
                    }
                }
            }

            fn visit_block(&mut self, block: &minic::parser::BlockStmt) {
                self.node_count += 1;
                for stmt in &block.statements {
                    match stmt {
                        minic::parser::Statement::VariableDecl(_v) => {
                            self.node_count += 1;
                        }
                        minic::parser::Statement::Expression(_e) => {
                            self.node_count += 1;
                        }
                        minic::parser::Statement::Return(r) => {
                            self.node_count += 1;
                            if r.value.is_some() {
                                self.node_count += 1;
                            }
                        }
                        minic::parser::Statement::Block(b) => self.visit_block(b),
                        _ => self.node_count += 1,
                    }
                }
            }

            fn visit_function_decl(&mut self, _func: &minic::parser::FunctionDecl) {}
            fn visit_struct_decl(&mut self, _struct_decl: &minic::parser::StructDecl) {}
            fn visit_var_decl(&mut self, _var_decl: &minic::parser::VarDecl) {}
            fn visit_param(&mut self, _param: &minic::parser::Param) {}
            fn visit_if_stmt(&mut self, _if_stmt: &minic::parser::IfStmt) {}
            fn visit_while_stmt(&mut self, _while_stmt: &minic::parser::WhileStmt) {}
            fn visit_for_stmt(&mut self, _for_stmt: &minic::parser::ForStmt) {}
            fn visit_return_stmt(&mut self, _return_stmt: &minic::parser::ReturnStmt) {}
            fn visit_expr_stmt(&mut self, _expr_stmt: &minic::parser::ExprStmt) {}
            fn visit_empty_stmt(&mut self, _empty_stmt: &minic::parser::EmptyStmt) {}
            fn visit_literal(&mut self, _literal: &minic::parser::Literal) {}
            fn visit_identifier(&mut self, _identifier: &minic::parser::IdentifierExpr) {}
            fn visit_binary(&mut self, _binary: &minic::parser::BinaryExpr) {}
            fn visit_unary(&mut self, _unary: &minic::parser::UnaryExpr) {}
            fn visit_assignment(&mut self, _assignment: &minic::parser::AssignmentExpr) {}
            fn visit_call(&mut self, _call: &minic::parser::CallExpr) {}
            fn visit_struct_access(&mut self, _access: &minic::parser::StructAccessExpr) {}
            fn visit_grouped(&mut self, _grouped: &minic::parser::GroupedExpr) {}
            fn visit_break_stmt(&mut self, _break_stmt: &minic::parser::BreakStmt) {}
            fn visit_continue_stmt(&mut self, _continue_stmt: &minic::parser::ContinueStmt) {}
            fn visit_switch_stmt(&mut self, _switch_stmt: &minic::parser::SwitchStmt) {}
            fn visit_case_stmt(&mut self, _case_stmt: &minic::parser::CaseStmt) {}
            fn visit_array_access(&mut self, _access: &minic::parser::ArrayAccessExpr) {}
        }

        let mut visitor = CountingVisitor::new();
        visitor.visit_program(&ast);

        assert!(
            visitor.node_count > 0,
            "Visitor должен найти хотя бы один узел"
        );
        println!("Найдено узлов: {}", visitor.node_count);

        assert!(
            visitor.node_count >= 5,
            "Найдено только {} узлов, ожидалось минимум 5",
            visitor.node_count
        );
    }
}
