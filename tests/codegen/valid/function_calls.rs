//! Тесты вызовов функций

use minic::compiler::compile_with_ir;
use minic::codegen::generate_assembly;

#[test]
fn test_simple_call() {
    let source = r#"
        fn add(int a, int b) -> int {
            return a + b;
        }
        fn main() -> int {
            return add(5, 3);
        }
    "#;
    let (parse_output, ir) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());
    let result = generate_assembly(&ir.unwrap(), false);
    assert!(result.assembly.contains("call add"));
}

#[test]
fn test_recursive_call() {
    let source = r#"
        fn factorial(int n) -> int {
            if (n <= 1) {
                return 1;
            }
            return n * factorial(n - 1);
        }
        fn main() -> int {
            return factorial(5);
        }
    "#;
    let (parse_output, ir) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());
    let result = generate_assembly(&ir.unwrap(), false);
    assert!(result.assembly.contains("call factorial"));
}