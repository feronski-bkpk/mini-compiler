//! Генерация x86-64 ассемблерного кода из IR
//!
//! Этот модуль реализует трансляцию промежуточного представления (IR)
//! в ассемблер x86-64 с соблюдением System V AMD64 ABI.

mod abi;
mod register_allocator;
mod stack_frame;
mod x86_generator;

pub use abi::{
    ABI, CALLEE_SAVED_REGISTERS, CALLER_SAVED_REGISTERS, CallingConvention, FLOAT_ARG_REGISTERS,
    INTEGER_ARG_REGISTERS, RETURN_REGISTERS, RegisterInfo, RegisterPurpose,
};
pub use register_allocator::{
    AdvancedRegisterAllocator, ConflictGraph, LiveRange, Register, RegisterStatistics,
};
pub use stack_frame::StackFrameManager;
pub use x86_generator::X86Generator;

use crate::ir::ProgramIR;
use std::fmt;

/// Результат генерации кода
#[derive(Debug, Clone)]
pub struct CodegenResult {
    /// Сгенерированный ассемблерный код
    pub assembly: String,
    /// Использованные регистры
    pub registers_used: Vec<String>,
    /// Размер стекового фрейма в байтах
    pub frame_size: usize,
    /// Количество сгенерированных инструкций
    pub instruction_count: usize,
}

impl fmt::Display for CodegenResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.assembly)
    }
}

/// Основная функция для генерации ассемблерного кода
pub fn generate_assembly(program: &ProgramIR, optimize: bool) -> CodegenResult {
    let mut generator = X86Generator::new();

    if optimize {
        let mut program_copy = program.clone();
        let _ = crate::ir::PeepholeOptimizer::optimize(&mut program_copy);
        generator.generate(&program_copy)
    } else {
        generator.generate(program)
    }
}

/// Генерирует ассемблерный код и сохраняет в файл
pub fn generate_to_file(
    program: &ProgramIR,
    output_path: &std::path::Path,
    optimize: bool,
) -> std::io::Result<()> {
    let result = generate_assembly(program, optimize);
    std::fs::write(output_path, result.assembly)?;
    Ok(())
}
