use minic::compiler;
use minic::ir::IRPrinter;
use std::fs;
use std::path::PathBuf;

/// Загружает исходный код из файла
fn load_source(name: &str) -> String {
    let path = PathBuf::from(format!("tests/ir/golden/{}.src", name));
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("Не удалось прочитать файл: {:?}", path))
}

/// Сравнивает сгенерированный IR с эталоном
fn assert_ir_matches(name: &str) {
    let source = load_source(name);

    let parse_output = compiler::syntactic_analysis(&source);

    if parse_output.has_errors() {
        println!("Синтаксические ошибки в {}:", name);
        for error in &parse_output.errors.errors {
            println!("  {}", error);
        }
        panic!("Синтаксические ошибки в {}", name);
    }

    let ast = parse_output.ast.expect("AST не построен");

    let mut analyzer = minic::semantic::SemanticAnalyzer::new();
    let semantic_output = analyzer.analyze(ast);

    if semantic_output.has_errors() {
        println!("Семантические ошибки в {}:", name);
        println!("{}", semantic_output.errors);
        panic!("Семантические ошибки в {}", name);
    }

    let mut ir_generator = minic::ir::IRGenerator::new(semantic_output.symbol_table);
    let program = ir_generator.generate(semantic_output.decorated_ast.expect("AST отсутствует"));

    let actual = IRPrinter::to_text(&program);

    let expected_path = PathBuf::from(format!("tests/ir/golden/expected/{}.ir", name));

    if !expected_path.exists() {
        println!("Создаем эталонный файл: {:?}", expected_path);
        fs::write(&expected_path, &actual).unwrap();
    }

    let expected = fs::read_to_string(&expected_path).unwrap();

    let actual_normalized = actual
        .lines()
        .filter(|line| !line.contains("# Generated:"))
        .collect::<Vec<_>>()
        .join("\n");

    let expected_normalized = expected
        .lines()
        .filter(|line| !line.contains("# Generated:"))
        .collect::<Vec<_>>()
        .join("\n");

    if actual_normalized != expected_normalized {
        println!("\n=== ФАКТИЧЕСКИЙ IR ===");
        println!("{}", actual_normalized);
        println!("=== ОЖИДАЕМЫЙ IR ===");
        println!("{}", expected_normalized);
        println!("=====================");
    }

    assert_eq!(
        actual_normalized, expected_normalized,
        "IR для {} не соответствует эталону",
        name
    );
}

/// Тест: простая арифметика
#[test]
fn golden_simple_arith() {
    assert_ir_matches("simple_arith");
}

/// Тест: if-else
#[test]
fn golden_if_else() {
    assert_ir_matches("if_else");
}

/// Тест: while цикл
#[test]
fn golden_while_loop() {
    assert_ir_matches("while_loop");
}

/// Тест: рекурсивный факториал
#[test]
fn golden_factorial() {
    assert_ir_matches("factorial");
}

/// Тест: вложенные if
#[test]
fn golden_nested_if() {
    assert_ir_matches("nested_if");
}

/// Тест: логические операции
/// Тест: логические операции
#[test]
fn golden_logical_ops() {
    let source = r#"
        fn main() -> bool {
            bool a = true;
            bool b = false;
            bool c = a && b;
            bool d = a || b;
            bool e = !a;
            return c;
        }
    "#;

    let parse_output = compiler::syntactic_analysis(source);
    assert!(!parse_output.has_errors(), "Синтаксические ошибки");

    let ast = parse_output.ast.unwrap();
    let mut analyzer = minic::semantic::SemanticAnalyzer::new();
    let semantic_output = analyzer.analyze(ast);
    assert!(
        !semantic_output.has_errors(),
        "Семантические ошибки: {}",
        semantic_output.errors
    );

    let mut ir_generator = minic::ir::IRGenerator::new(semantic_output.symbol_table);
    let program = ir_generator.generate(semantic_output.decorated_ast.unwrap());
    let actual = IRPrinter::to_text(&program);

    assert!(
        actual.contains("JUMP_IF") || actual.contains("JUMP_IF_NOT"),
        "Должны быть условные переходы для short-circuit"
    );
    assert!(
        actual.contains("XOR") || actual.contains("NOT"),
        "Должна быть инструкция NOT (или XOR)"
    );
}

/// Тест: операции сравнения
#[test]
fn golden_comparison_ops() {
    let source = r#"
        fn main() -> bool {
            int x = 5;
            int y = 10;
            bool eq = x == y;
            bool ne = x != y;
            bool lt = x < y;
            bool le = x <= y;
            bool gt = x > y;
            bool ge = x >= y;
            return eq;
        }
    "#;

    let parse_output = compiler::syntactic_analysis(source);
    assert!(!parse_output.has_errors(), "Синтаксические ошибки");

    let ast = parse_output.ast.unwrap();
    let mut analyzer = minic::semantic::SemanticAnalyzer::new();
    let semantic_output = analyzer.analyze(ast);
    assert!(
        !semantic_output.has_errors(),
        "Семантические ошибки: {}",
        semantic_output.errors
    );

    let mut ir_generator = minic::ir::IRGenerator::new(semantic_output.symbol_table);
    let program = ir_generator.generate(semantic_output.decorated_ast.unwrap());
    let actual = IRPrinter::to_text(&program);

    assert!(actual.contains("CMP_EQ"), "Должна быть инструкция CMP_EQ");
    assert!(actual.contains("CMP_NE"), "Должна быть инструкция CMP_NE");
    assert!(actual.contains("CMP_LT"), "Должна быть инструкция CMP_LT");
    assert!(actual.contains("CMP_LE"), "Должна быть инструкция CMP_LE");
    assert!(actual.contains("CMP_GT"), "Должна быть инструкция CMP_GT");
    assert!(actual.contains("CMP_GE"), "Должна быть инструкция CMP_GE");
}

/// Тест: вызов функций
#[test]
fn golden_function_call() {
    let source = r#"
        fn add(int a, int b) -> int {
            return a + b;
        }

        fn main() -> int {
            return add(5, 3);
        }
    "#;

    let parse_output = compiler::syntactic_analysis(source);
    assert!(!parse_output.has_errors(), "Синтаксические ошибки");

    let ast = parse_output.ast.unwrap();
    let mut analyzer = minic::semantic::SemanticAnalyzer::new();
    let semantic_output = analyzer.analyze(ast);
    assert!(
        !semantic_output.has_errors(),
        "Семантические ошибки: {}",
        semantic_output.errors
    );

    let mut ir_generator = minic::ir::IRGenerator::new(semantic_output.symbol_table);
    let program = ir_generator.generate(semantic_output.decorated_ast.unwrap());
    let actual = IRPrinter::to_text(&program);

    assert!(actual.contains("PARAM"), "Должна быть инструкция PARAM");
    assert!(actual.contains("CALL"), "Должна быть инструкция CALL");
}
