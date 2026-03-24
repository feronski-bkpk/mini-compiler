//! Вывод IR в различных форматах

use super::basic_block::{FunctionIR, IRStatistics, ProgramIR};
use std::collections::HashMap;

/// Принтер IR
pub struct IRPrinter;

impl IRPrinter {
    /// Выводит IR в текстовом формате
    pub fn to_text(program: &ProgramIR) -> String {
        let mut output = String::new();

        output.push_str(&format!("# Program: MiniC IR\n"));
        output.push_str(&format!("# Generated: {}\n\n", Self::current_time()));

        if !program.globals.is_empty() {
            output.push_str(".global\n");
            for (name, typ) in &program.globals {
                output.push_str(&format!("  {}: {}\n", name, typ));
            }
            output.push_str("\n");
        }

        for func in program.functions.values() {
            output.push_str(&Self::function_to_text(func));
            output.push_str("\n");
        }

        output
    }

    /// Возвращает текущее время
    fn current_time() -> String {
        use std::time::SystemTime;
        use std::time::UNIX_EPOCH;

        let now = SystemTime::now();
        let since_epoch = now.duration_since(UNIX_EPOCH).unwrap();
        let seconds = since_epoch.as_secs();
        let datetime = chrono::DateTime::from_timestamp(seconds as i64, 0);

        if let Some(dt) = datetime {
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        } else {
            "2026-03-23 12:00:00".to_string()
        }
    }

    /// Выводит функцию в текстовом формате
    fn function_to_text(func: &FunctionIR) -> String {
        let mut output = String::new();

        let params: Vec<String> = func
            .parameters
            .iter()
            .map(|(name, typ)| format!("{} {}", typ, name))
            .collect();
        output.push_str(&format!(
            "function {}: {} ({})\n",
            func.name,
            func.return_type,
            params.join(", ")
        ));

        if !func.locals.is_empty() {
            output.push_str("  locals:\n");
            for (name, typ) in &func.locals {
                output.push_str(&format!("    {}: {}\n", name, typ));
            }
            output.push_str("\n");
        }

        let mut blocks: Vec<&String> = func.blocks.keys().collect();
        blocks.sort();

        for block_label in blocks {
            if let Some(block) = func.blocks.get(block_label) {
                output.push_str(&format!("  {}:\n", block_label));
                for instr in &block.instructions {
                    output.push_str(&format!("    {}\n", instr));
                }
                output.push_str("\n");
            }
        }

        output
    }

    /// Выводит IR в формате DOT (Graphviz)
    pub fn to_dot(program: &ProgramIR) -> String {
        let mut output = String::new();

        output.push_str("digraph IR {\n");
        output.push_str("  rankdir=TB;\n");
        output.push_str("  node [shape=box, fontname=\"Courier\"];\n\n");

        for func in program.functions.values() {
            output.push_str(&format!("  subgraph cluster_{} {{\n", func.name));
            output.push_str(&format!("    label = \"Function: {}\";\n", func.name));
            output.push_str("    style = filled;\n");
            output.push_str("    color = lightgrey;\n\n");

            for (label, block) in &func.blocks {
                let label_escaped = label.replace('"', "\\\"");

                let mut instr_text = String::new();
                for instr in &block.instructions {
                    instr_text.push_str(&format!("{}\\l", instr));
                }

                output.push_str(&format!(
                    "    \"{}\" [label=\"{}\\l{}\", shape=box];\n",
                    label_escaped, label_escaped, instr_text
                ));
            }

            output.push_str("\n");

            for (label, block) in &func.blocks {
                let from_escaped = label.replace('"', "\\\"");
                for succ in &block.successors {
                    let to_escaped = succ.replace('"', "\\\"");
                    output.push_str(&format!(
                        "    \"{}\" -> \"{}\";\n",
                        from_escaped, to_escaped
                    ));
                }
            }

            output.push_str("  }\n\n");
        }

        output.push_str("}\n");
        output
    }

    /// Выводит IR в формате JSON
    pub fn to_json(program: &ProgramIR) -> Result<String, serde_json::Error> {
        #[derive(serde::Serialize)]
        struct JSONProgram {
            functions: Vec<JSONFunction>,
            globals: Vec<JSONGlobal>,
            statistics: JSONStatistics,
        }

        #[derive(serde::Serialize)]
        struct JSONFunction {
            name: String,
            return_type: String,
            parameters: Vec<JSONParam>,
            locals: Vec<JSONVar>,
            blocks: Vec<JSONBlock>,
            entry_block: String,
            exit_blocks: Vec<String>,
        }

        #[derive(serde::Serialize)]
        struct JSONParam {
            name: String,
            typ: String,
        }

        #[derive(serde::Serialize)]
        struct JSONVar {
            name: String,
            typ: String,
        }

        #[derive(serde::Serialize)]
        struct JSONBlock {
            label: String,
            instructions: Vec<String>,
            predecessors: Vec<String>,
            successors: Vec<String>,
        }

        #[derive(serde::Serialize)]
        struct JSONGlobal {
            name: String,
            typ: String,
        }

        #[derive(serde::Serialize)]
        struct JSONStatistics {
            total_instructions: usize,
            basic_block_count: usize,
            temporary_count: usize,
            instruction_counts: HashMap<String, usize>,
        }

        let stats = IRStatistics::compute(program);

        let functions: Vec<JSONFunction> = program
            .functions
            .values()
            .map(|func| JSONFunction {
                name: func.name.clone(),
                return_type: func.return_type.clone(),
                parameters: func
                    .parameters
                    .iter()
                    .map(|(name, typ)| JSONParam {
                        name: name.clone(),
                        typ: typ.clone(),
                    })
                    .collect(),
                locals: func
                    .locals
                    .iter()
                    .map(|(name, typ)| JSONVar {
                        name: name.clone(),
                        typ: typ.clone(),
                    })
                    .collect(),
                blocks: func
                    .blocks
                    .values()
                    .map(|block| JSONBlock {
                        label: block.label.clone(),
                        instructions: block
                            .instructions
                            .iter()
                            .map(|instr| instr.to_string())
                            .collect(),
                        predecessors: block.predecessors.clone(),
                        successors: block.successors.clone(),
                    })
                    .collect(),
                entry_block: func.entry_block.clone(),
                exit_blocks: func.exit_blocks.clone(),
            })
            .collect();

        let globals: Vec<JSONGlobal> = program
            .globals
            .iter()
            .map(|(name, typ)| JSONGlobal {
                name: name.clone(),
                typ: typ.clone(),
            })
            .collect();

        let json_program = JSONProgram {
            functions,
            globals,
            statistics: JSONStatistics {
                total_instructions: stats.total_instructions,
                basic_block_count: stats.basic_block_count,
                temporary_count: stats.temporary_count,
                instruction_counts: stats.instruction_counts,
            },
        };

        serde_json::to_string_pretty(&json_program)
    }

    /// Выводит статистику IR
    pub fn print_stats(program: &ProgramIR) -> String {
        let stats = IRStatistics::compute(program);

        let mut output = String::new();
        output.push_str("=== IR STATISTICS ===\n");
        output.push_str(&format!(
            "Total instructions: {}\n",
            stats.total_instructions
        ));
        output.push_str(&format!("Basic blocks: {}\n", stats.basic_block_count));
        output.push_str(&format!("Temporaries used: {}\n", stats.temporary_count));
        output.push_str("\nInstruction breakdown:\n");

        let mut counts: Vec<(&String, &usize)> = stats.instruction_counts.iter().collect();
        counts.sort_by(|a, b| b.1.cmp(a.1));

        for (name, count) in counts {
            output.push_str(&format!("  {}: {}\n", name, count));
        }

        output
    }
}
