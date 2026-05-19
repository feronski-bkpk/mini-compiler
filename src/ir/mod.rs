//! Модуль промежуточного представления (IR) для Mini Compiler
//!
//! Этот модуль реализует генерацию трехадресного кода из декорированного AST.

pub mod basic_block;
pub mod control_flow;
pub mod inline_optimizer;
pub mod ir_generator;
pub mod ir_instructions;
pub mod ir_printer;
pub mod peephole_optimizer;

pub use basic_block::*;
pub use control_flow::ControlFlowGraph;
pub use ir_generator::IRGenerator;
pub use ir_instructions::*;
pub use ir_printer::IRPrinter;
pub use peephole_optimizer::{OptimizationReport, PeepholeOptimizer};
