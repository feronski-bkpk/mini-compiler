//! Модуль промежуточного представления (IR) для Mini Compiler
//!
//! Этот модуль реализует генерацию трехадресного кода из декорированного AST.

mod basic_block;
mod control_flow;
mod ir_generator;
mod ir_instructions;
mod ir_printer;
mod peephole_optimizer;

pub use basic_block::*;
pub use control_flow::ControlFlowGraph;
pub use ir_generator::IRGenerator;
pub use ir_instructions::*;
pub use ir_printer::IRPrinter;
pub use peephole_optimizer::{OptimizationReport, PeepholeOptimizer};
