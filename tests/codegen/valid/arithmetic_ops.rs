//! Тесты арифметических операций

use minic::compiler::compile_with_ir;
use minic::codegen::generate_assembly;

#[test]
fn test_addition() {
    let source = "fn main() -> int { return 5 + 3; }";
    let (parse_output, ir) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());
    let result = generate_assembly(&ir.unwrap(), false);
    assert!(result.assembly.contains("add"));
}

#[test]
fn test_subtraction() {
    let source = "fn main() -> int { return 10 - 3; }";
    let (parse_output, ir) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());
    let result = generate_assembly(&ir.unwrap(), false);
    assert!(result.assembly.contains("sub"));
}

#[test]
fn test_multiplication() {
    let source = "fn main() -> int { return 6 * 7; }";
    let (parse_output, ir) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());
    let result = generate_assembly(&ir.unwrap(), false);
    assert!(result.assembly.contains("imul"));
}

#[test]
fn test_division() {
    let source = "fn main() -> int { return 10 / 2; }";
    let (parse_output, ir) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());
    let result = generate_assembly(&ir.unwrap(), false);
    assert!(result.assembly.contains("idiv"));
}