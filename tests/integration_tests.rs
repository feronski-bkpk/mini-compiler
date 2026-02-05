//! Интеграционные тесты для полного пайплайна

use minic::lexer::Scanner;
use minic::preprocessor::Preprocessor;

#[test]
fn test_full_pipeline_with_macros() {
    let source = r#"
#define MAX 100
#define MESSAGE "Hello"

fn main() {
    int x = MAX;
    string msg = MESSAGE;

    #ifdef DEBUG
        log("Debug mode");
    #endif

    return x;
}
"#;

    let mut preprocessor = Preprocessor::new(source);
    preprocessor.define("DEBUG", "1").unwrap();
    let processed = preprocessor.process().unwrap();

    let mut scanner = Scanner::new(&processed);
    let (tokens, errors) = scanner.scan_all();

    assert!(errors.is_empty());
    assert!(!tokens.is_empty());

    let source_contains_max = processed.contains("MAX");
    let source_contains_100 = processed.contains("100");
    assert!(!source_contains_max || source_contains_100);
}

#[test]
fn test_conditional_compilation() {
    let source = r#"
#ifdef ENABLED
int value = 1;
#else
int value = 0;
#endif
"#;

    let mut pp1 = Preprocessor::new(source);
    pp1.define("ENABLED", "").unwrap();
    let result1 = pp1.process().unwrap();
    assert!(result1.contains("int value = 1;"));
    assert!(!result1.contains("int value = 0;"));

    let mut pp2 = Preprocessor::new(source);
    let result2 = pp2.process().unwrap();
    assert!(!result2.contains("int value = 1;"));
    assert!(result2.contains("int value = 0;"));
}
