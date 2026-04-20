//! Тесты управления потоком

use minic::compiler::compile_with_ir;
use minic::codegen::generate_assembly;

#[test]
fn test_if_statement() {
    let source = r#"
        fn main() -> int {
            int x = 10;
            if (x > 5) {
                return 1;
            }
            return 0;
        }
    "#;
    let (parse_output, ir) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());
    let result = generate_assembly(&ir.unwrap(), false);
    assert!(result.assembly.contains("cmp"));
    assert!(result.assembly.contains("jz") || result.assembly.contains("jnz"));
}

#[test]
fn test_while_loop() {
    let source = r#"
        fn main() -> int {
            int i = 0;
            int sum = 0;
            while (i < 10) {
                sum = sum + i;
                i = i + 1;
            }
            return sum;
        }
    "#;
    let (parse_output, ir) = compile_with_ir(source, vec![]);
    assert!(parse_output.is_valid());
    let result = generate_assembly(&ir.unwrap(), false);
    assert!(result.assembly.contains("jmp"));
    assert!(result.assembly.contains("cmp"));
}