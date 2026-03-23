//! Тесты семантического анализатора

#[cfg(test)]
mod tests {
    use minic::compiler;
    use minic::semantic::{SemanticAnalyzer, SemanticErrorKind};

    /// Вспомогательная функция для тестирования семантики
    fn analyze(source: &str) -> (bool, Vec<SemanticErrorKind>) {
        let parse_output = compiler::syntactic_analysis(source);
        assert!(parse_output.ast.is_some(), "Синтаксическая ошибка в тесте");

        let mut analyzer = SemanticAnalyzer::new();
        let output = analyzer.analyze(parse_output.ast.unwrap());

        let error_kinds: Vec<_> = output
            .errors
            .errors
            .iter()
            .map(|e| e.kind.clone())
            .collect();
        (output.is_valid(), error_kinds)
    }

    #[test]
    fn test_valid_variable_declaration() {
        let source = "fn main() { int x = 42; }";
        let (valid, errors) = analyze(source);
        assert!(valid, "Ошибки: {:?}", errors);
    }

    #[test]
    fn test_valid_function_call() {
        let source = r#"
            fn add(int a, int b) -> int {
                return a + b;
            }
            fn main() {
                int x = add(5, 3);
            }
        "#;
        let (valid, errors) = analyze(source);
        assert!(valid, "Ошибки: {:?}", errors);
    }

    #[test]
    fn test_undeclared_variable() {
        let source = "fn main() { int x = y; }";
        let (valid, errors) = analyze(source);
        assert!(!valid);
        assert!(errors.contains(&SemanticErrorKind::UndeclaredIdentifier));
    }

    #[test]
    fn test_type_mismatch() {
        let source = "fn main() { int x = 3.14; }";
        let (valid, errors) = analyze(source);
        assert!(!valid);
        assert!(errors.contains(&SemanticErrorKind::AssignmentTypeMismatch));
    }

    #[test]
    fn test_duplicate_variable() {
        let source = "fn main() { int x = 5; int x = 10; }";
        let (valid, errors) = analyze(source);
        assert!(!valid);
        assert!(errors.contains(&SemanticErrorKind::DuplicateDeclaration));
    }

    #[test]
    fn test_missing_return() {
        let source = "fn add(int a, int b) -> int { }";
        let (valid, errors) = analyze(source);
        assert!(!valid);
        assert!(errors.contains(&SemanticErrorKind::InvalidReturnType));
    }

    #[test]
    fn test_wrong_return_type() {
        let source = "fn add(int a, int b) -> int { return 3.14; }";
        let (valid, errors) = analyze(source);
        assert!(!valid);
        assert!(errors.contains(&SemanticErrorKind::InvalidReturnType));
    }

    #[test]
    fn test_void_function_return() {
        let source = "fn main() -> void { return; }";
        let (valid, errors) = analyze(source);
        assert!(valid, "Ошибки: {:?}", errors);
    }

    #[test]
    fn test_function_argument_count() {
        let source = r#"
            fn add(int a, int b) -> int { return a + b; }
            fn main() { int x = add(5); }
        "#;
        let (valid, errors) = analyze(source);
        assert!(!valid);
        assert!(errors.contains(&SemanticErrorKind::ArgumentCountMismatch));
    }

    #[test]
    fn test_function_argument_type() {
        let source = r#"
            fn add(int a, int b) -> int { return a + b; }
            fn main() { int x = add(5, "hello"); }
        "#;
        let (valid, errors) = analyze(source);
        assert!(!valid);
        assert!(errors.contains(&SemanticErrorKind::ArgumentTypeMismatch));
    }

    #[test]
    fn test_struct_field_access() {
        let source = r#"
            struct Point { int x; int y; }
            fn main() {
                struct Point p;
                p.x = 10;
                p.y = 20;
                int z = p.x + p.y;
            }
        "#;
        let (valid, errors) = analyze(source);
        assert!(valid, "Ошибки: {:?}", errors);
    }

    #[test]
    fn test_undeclared_struct_field() {
        let source = r#"
            struct Point { int x; int y; }
            fn main() {
                struct Point p;
                p.z = 10;
            }
        "#;
        let (valid, errors) = analyze(source);
        assert!(!valid);
        assert!(errors.contains(&SemanticErrorKind::UndeclaredField));
    }

    #[test]
    fn test_if_condition_type() {
        let source = "fn main() { if (42) { return 0; } }";
        let (valid, errors) = analyze(source);
        assert!(!valid);
        assert!(errors.contains(&SemanticErrorKind::InvalidConditionType));
    }

    #[test]
    fn test_while_condition_type() {
        let source = "fn main() { while (42) { } }";
        let (valid, errors) = analyze(source);
        assert!(!valid);
        assert!(errors.contains(&SemanticErrorKind::InvalidConditionType));
    }

    #[test]
    fn test_nested_scopes() {
        let source = r#"
            fn main() {
                int x = 5;
                if (x > 0) {
                    int y = 10;
                    x = y;
                }
                y = 15;  // Ошибка: y вне области видимости
            }
        "#;
        let (valid, errors) = analyze(source);
        assert!(!valid);
        assert!(errors.contains(&SemanticErrorKind::UndeclaredIdentifier));
    }

    #[test]
    fn test_type_inference_arithmetic() {
        let source = r#"
            fn main() {
                int a = 5;
                float b = 3.14;
                float c = a + b;  // int + float -> float
                int d = a + 10;    // int + int -> int
            }
        "#;
        let (valid, errors) = analyze(source);
        assert!(valid, "Ошибки: {:?}", errors);
    }

    #[test]
    fn test_increment_operators() {
        let source = r#"
            fn main() {
                int x = 5;
                x++;
                ++x;
                int y = x++ + ++x;
                float f = 3.14;
                f++;
            }
        "#;
        let (valid, errors) = analyze(source);
        assert!(valid, "Ошибки: {:?}", errors);
    }

    #[test]
    fn test_comparison_operators() {
        let source = r#"
            fn main() {
                bool b1 = 5 > 3;
                bool b2 = 3.14 <= 2.71;
                bool b3 = 5 == 5.0;
            }
        "#;
        let (valid, errors) = analyze(source);
        assert!(valid, "Ошибки: {:?}", errors);
    }

    #[test]
    fn test_var_type_inference() {
        let source = r#"
        fn main() {
            var x = 42;           // выводится int
            var y = 3.14;         // выводится float
            var z = true;          // выводится bool
            var s = "hello";       // выводится string

            int a = x;             // int = int - OK
            float b = y;           // float = float - OK
            bool c = z;            // bool = bool - OK

            x = 100;               // int = int - OK
            x = 3.14;              // Ошибка: int != float
        }
    "#;
        let (valid, errors) = analyze(source);
        assert!(!valid);
        assert!(errors.contains(&SemanticErrorKind::AssignmentTypeMismatch));
    }

    #[test]
    fn test_var_inference_consistency() {
        let source = r#"
        fn main() {
            var x = 42;
            x = 100;      // OK
            x = "hello";  // Ошибка: тип несовместим
        }
    "#;
        let (valid, errors) = analyze(source);
        assert!(!valid);
        assert!(errors.contains(&SemanticErrorKind::AssignmentTypeMismatch));
    }

    #[test]
    fn test_var_without_initializer() {
        let source = r#"
        fn main() {
            var x;  // Ошибка: var требует инициализатора
        }
    "#;
        let (valid, _errors) = analyze(source);
        assert!(!valid);
    }

    #[test]
    fn test_struct_with_layout() {
        let source = r#"
        struct Point {
            int x;
            int y;
        }

        fn main() {
            struct Point p;
            p.x = 10;
            p.y = 20;
        }
    "#;
        let parse_output = compiler::syntactic_analysis(source);
        let mut analyzer = SemanticAnalyzer::new();
        let output = analyzer.analyze(parse_output.ast.unwrap());

        assert!(
            output.is_valid(),
            "Ожидалось отсутствие ошибок, получено {}",
            output.errors.len()
        );

        let dump = output.symbol_table.dump_with_layout();
        println!("{}", dump);

        assert!(
            dump.contains("смещение"),
            "Таблица не содержит информацию о смещениях, got: {}",
            dump
        );
        assert!(
            dump.contains("размер"),
            "Таблица не содержит информацию о размерах, got: {}",
            dump
        );

        assert!(
            (dump.contains("x: int [смещение: 0] [размер: 4]")
                && dump.contains("y: int [смещение: 4] [размер: 4]"))
                || (dump.contains("y: int [смещение: 0] [размер: 4]")
                    && dump.contains("x: int [смещение: 4] [размер: 4]")),
            "Поля должны иметь правильные смещения (x:0,y:4 или y:0,x:4), получено: {}",
            dump
        );
    }

    #[test]
    fn test_var_type_inference_with_layout() {
        let source = r#"
        fn main() {
            var x = 42;
            var y = 3.14;
            var z = true;
            var s = "hello";

            x = 100;
            y = 2.71;
            z = false;
            s = "world";
        }
    "#;
        let parse_output = compiler::syntactic_analysis(source);
        let mut analyzer = SemanticAnalyzer::new();
        let output = analyzer.analyze(parse_output.ast.unwrap());

        assert!(
            output.is_valid(),
            "Expected no errors, got {}",
            output.errors.len()
        );

        let dump = output.symbol_table.dump_with_layout();
        println!("{}", dump);

        assert!(!output.has_errors(), "There should be no semantic errors");
    }

    #[test]
    fn test_global_var_type_inference() {
        let source = r#"
        var global_x = 42;
        var global_y = 3.14;
        var global_z = true;
        var global_s = "hello";

        fn main() -> int {
            return 0;
        }
    "#;
        let parse_output = compiler::syntactic_analysis(source);
        let mut analyzer = SemanticAnalyzer::new();
        let output = analyzer.analyze(parse_output.ast.unwrap());

        if output.has_errors() {
            println!("Семантические ошибки:");
            for error in &output.errors.errors {
                println!("  {}", error);
            }
        }

        assert!(
            output.is_valid(),
            "Ожидалось отсутствие ошибок, получено {}",
            output.errors.len()
        );

        let dump = output.symbol_table.dump_with_layout();
        println!("{}", dump);

        assert!(
            dump.contains("global_x: переменная - int"),
            "global_x should be int, got: {}",
            dump
        );
        assert!(
            dump.contains("global_y: переменная - float"),
            "global_y should be float, got: {}",
            dump
        );
        assert!(
            dump.contains("global_z: переменная - bool"),
            "global_z should be bool, got: {}",
            dump
        );
        assert!(
            dump.contains("global_s: переменная - string"),
            "global_s should be string, got: {}",
            dump
        );
    }
}
