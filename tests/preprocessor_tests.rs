//! Тесты для препроцессора.

use minic::preprocessor::Preprocessor;

#[test]
fn test_comment_removal() {
    let source = r#"
// Однострочный комментарий
int x = 42; // Комментарий после кода
/* Многострочный
   комментарий */
int y = /* встроенный комментарий */ 100;
"#;

    let mut preprocessor = Preprocessor::new(source);
    preprocessor.preserve_line_numbers(false);

    let result = preprocessor.process().unwrap();

    println!("Результат:\n{}", result);

    assert!(!result.contains("//"));
    assert!(!result.contains("/*"));
    assert!(!result.contains("*/"));
    assert!(result.contains("int x = 42;"));
    assert!(result.contains("int y ="));
    assert!(result.contains("100;"));
}

#[test]
fn test_preserve_line_numbers() {
    let source = "line1\n// comment\nline3";

    let mut preprocessor = Preprocessor::new(source);
    preprocessor.preserve_line_numbers(true);

    let result = preprocessor.process().unwrap();
    let lines: Vec<&str> = result.lines().collect();

    assert_eq!(lines.len(), 3);
}

#[test]
fn test_simple_macro() {
    let source = r#"
#define MAX 100
int x = MAX;
int y = MAX * 2;
"#;

    let mut preprocessor = Preprocessor::new(source);
    let result = preprocessor.process().unwrap();

    assert!(result.contains("int x = 100;"));
    assert!(result.contains("int y = 100 * 2;"));
    assert!(!result.contains("MAX"));
}

#[test]
fn test_conditional_directives() {
    let source = r#"
#ifdef DEBUG
int debug = 1;
#else
int debug = 0;
#endif
"#;

    let mut preprocessor1 = Preprocessor::new(source);
    preprocessor1.define("DEBUG", "1").unwrap();
    let result1 = preprocessor1.process().unwrap();
    assert!(result1.contains("int debug = 1;"));
    assert!(!result1.contains("int debug = 0;"));

    let mut preprocessor2 = Preprocessor::new(source);
    let result2 = preprocessor2.process().unwrap();
    assert!(!result2.contains("int debug = 1;"));
    assert!(result2.contains("int debug = 0;"));
}

#[test]
fn test_macro_recursion_detection() {
    let source = r#"
#define A B
#define B A
int x = A;
"#;

    let mut preprocessor = Preprocessor::new(source);
    let result = preprocessor.process();

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = err.to_string();
    println!("Ошибка: {}", err_str);

    assert!(
        err_str.contains("рекурсивное")
            || err_str.contains("Рекурсивное")
            || err_str.contains("recursive")
            || err_str.contains("Recursive")
    );
}

#[test]
fn test_comments_in_strings() {
    let source = r#"
char* msg = "// это не комментарий";
char* msg2 = "/* и это не комментарий */";
// А это комментарий
"#;

    let mut preprocessor = Preprocessor::new(source);
    let result = preprocessor.process().unwrap();

    assert!(result.contains("\"// это не комментарий\""));
    assert!(result.contains("\"/* и это не комментарий */\""));
    assert!(!result.contains("А это комментарий"));
}

#[test]
fn test_unterminated_comment_error() {
    let source = "/* незавершенный комментарий";

    let mut preprocessor = Preprocessor::new(source);
    let result = preprocessor.process();

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Незавершенный"));
}

#[test]
fn test_comment_removal_edge_cases() {
    let source = r#"str = "http://example.com"; // комментарий"#;
    let mut pp = Preprocessor::new(source);
    let result = pp.process().unwrap();
    assert!(result.contains("http://example.com"));
    assert!(!result.contains("// комментарий"));

    let source = "/* внешний /* внутренний */ все еще внешний */";
    let mut pp = Preprocessor::new(source);
    let _result = pp.process().unwrap();

    let source = "int x = 1; // EOF комментарий";
    let mut pp = Preprocessor::new(source);
    pp.preserve_line_numbers(true);
    let result = pp.process().unwrap();
    assert!(result.trim().ends_with(";"));
}
