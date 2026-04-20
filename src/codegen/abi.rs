//! Константы и утилиты для System V AMD64 ABI

/// ABI константы для x86-64
pub mod abi_constants {
    /// Выравнивание стека (16 байт)
    pub const STACK_ALIGNMENT: usize = 16;
}

/// Регистры для передачи целочисленных аргументов
pub const INTEGER_ARG_REGISTERS: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

/// Регистры для передачи аргументов с плавающей точкой
pub const FLOAT_ARG_REGISTERS: [&str; 8] = [
    "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6", "xmm7",
];

/// Регистры для возвращаемых значений
pub const RETURN_REGISTERS: [&str; 2] = ["rax", "rdx"];

/// Caller-saved регистры (могут быть изменены при вызове)
pub const CALLER_SAVED_REGISTERS: [&str; 11] = [
    "rax", "rcx", "rdx", "rsi", "rdi", "r8", "r9", "r10", "r11", "xmm0", "xmm1",
];

/// Callee-saved регистры (должны быть сохранены вызываемой функцией)
pub const CALLEE_SAVED_REGISTERS: [&str; 7] = ["rbx", "rsp", "rbp", "r12", "r13", "r14", "r15"];

/// Соглашение о вызовах
#[derive(Debug, Clone, Copy)]
pub struct CallingConvention;

impl CallingConvention {
    /// Возвращает регистр для n-го целочисленного аргумента
    pub fn integer_arg_register(index: usize) -> Option<&'static str> {
        INTEGER_ARG_REGISTERS.get(index).copied()
    }

    /// Возвращает регистр для n-го аргумента с плавающей точкой
    pub fn float_arg_register(index: usize) -> Option<&'static str> {
        FLOAT_ARG_REGISTERS.get(index).copied()
    }

    /// Возвращает регистр для возвращаемого значения
    pub fn return_register() -> &'static str {
        "rax"
    }

    /// Возвращает регистр для второго возвращаемого значения (для 128-битных значений)
    pub fn second_return_register() -> &'static str {
        "rdx"
    }

    /// Проверяет, является ли регистр caller-saved
    pub fn is_caller_saved(reg: &str) -> bool {
        CALLER_SAVED_REGISTERS.contains(&reg)
    }

    /// Проверяет, является ли регистр callee-saved
    pub fn is_callee_saved(reg: &str) -> bool {
        CALLEE_SAVED_REGISTERS.contains(&reg)
    }

    /// Возвращает выровненный размер стека
    pub fn align_stack(size: usize) -> usize {
        (size + abi_constants::STACK_ALIGNMENT - 1) & !(abi_constants::STACK_ALIGNMENT - 1)
    }
}

/// Информация о регистре
#[derive(Debug, Clone)]
pub struct RegisterInfo {
    pub name: String,
    pub size: usize,
    pub is_caller_saved: bool,
    pub is_callee_saved: bool,
    pub purpose: RegisterPurpose,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegisterPurpose {
    GeneralPurpose,
    Argument,
    ReturnValue,
    StackPointer,
    FramePointer,
    InstructionPointer,
    Temporary,
}

impl RegisterInfo {
    /// Возвращает информацию о регистре по имени
    pub fn for_name(name: &str) -> Option<Self> {
        match name {
            "rax" => Some(RegisterInfo {
                name: "rax".to_string(),
                size: 8,
                is_caller_saved: true,
                is_callee_saved: false,
                purpose: RegisterPurpose::ReturnValue,
            }),
            "rbx" => Some(RegisterInfo {
                name: "rbx".to_string(),
                size: 8,
                is_caller_saved: false,
                is_callee_saved: true,
                purpose: RegisterPurpose::GeneralPurpose,
            }),
            "rcx" => Some(RegisterInfo {
                name: "rcx".to_string(),
                size: 8,
                is_caller_saved: true,
                is_callee_saved: false,
                purpose: RegisterPurpose::Argument,
            }),
            "rdx" => Some(RegisterInfo {
                name: "rdx".to_string(),
                size: 8,
                is_caller_saved: true,
                is_callee_saved: false,
                purpose: RegisterPurpose::Argument,
            }),
            "rsi" => Some(RegisterInfo {
                name: "rsi".to_string(),
                size: 8,
                is_caller_saved: true,
                is_callee_saved: false,
                purpose: RegisterPurpose::Argument,
            }),
            "rdi" => Some(RegisterInfo {
                name: "rdi".to_string(),
                size: 8,
                is_caller_saved: true,
                is_callee_saved: false,
                purpose: RegisterPurpose::Argument,
            }),
            "rbp" => Some(RegisterInfo {
                name: "rbp".to_string(),
                size: 8,
                is_caller_saved: false,
                is_callee_saved: true,
                purpose: RegisterPurpose::FramePointer,
            }),
            "rsp" => Some(RegisterInfo {
                name: "rsp".to_string(),
                size: 8,
                is_caller_saved: false,
                is_callee_saved: true,
                purpose: RegisterPurpose::StackPointer,
            }),
            "r8" => Some(RegisterInfo {
                name: "r8".to_string(),
                size: 8,
                is_caller_saved: true,
                is_callee_saved: false,
                purpose: RegisterPurpose::Argument,
            }),
            "r9" => Some(RegisterInfo {
                name: "r9".to_string(),
                size: 8,
                is_caller_saved: true,
                is_callee_saved: false,
                purpose: RegisterPurpose::Argument,
            }),
            "r10" | "r11" => Some(RegisterInfo {
                name: name.to_string(),
                size: 8,
                is_caller_saved: true,
                is_callee_saved: false,
                purpose: RegisterPurpose::Temporary,
            }),
            "r12" | "r13" | "r14" | "r15" => Some(RegisterInfo {
                name: name.to_string(),
                size: 8,
                is_caller_saved: false,
                is_callee_saved: true,
                purpose: RegisterPurpose::GeneralPurpose,
            }),
            _ => None,
        }
    }
}

/// ABI для System V AMD64
#[derive(Debug)]
pub struct ABI;

impl ABI {
    /// Возвращает размер типа в байтах
    pub fn type_size(typ: &crate::ir::IRType) -> usize {
        match typ {
            crate::ir::IRType::Int => 4,
            crate::ir::IRType::Float => 8,
            crate::ir::IRType::Bool => 1,
            crate::ir::IRType::Void => 0,
            crate::ir::IRType::String => 8,
            crate::ir::IRType::Struct(_) => 8,
            crate::ir::IRType::Pointer(_) => 8,
            crate::ir::IRType::Unknown => 0,
        }
    }

    /// Возвращает выравнивание типа
    pub fn type_alignment(typ: &crate::ir::IRType) -> usize {
        match typ {
            crate::ir::IRType::Int => 4,
            crate::ir::IRType::Float => 8,
            crate::ir::IRType::Bool => 1,
            crate::ir::IRType::Void => 0,
            crate::ir::IRType::String => 8,
            crate::ir::IRType::Struct(_) => 8,
            crate::ir::IRType::Pointer(_) => 8,
            crate::ir::IRType::Unknown => 0,
        }
    }

    /// Проверяет, является ли тип агрегатным (структура)
    pub fn is_aggregate_type(typ: &crate::ir::IRType) -> bool {
        matches!(typ, crate::ir::IRType::Struct(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calling_convention() {
        assert_eq!(CallingConvention::integer_arg_register(0), Some("rdi"));
        assert_eq!(CallingConvention::integer_arg_register(5), Some("r9"));
        assert_eq!(CallingConvention::integer_arg_register(6), None);

        assert_eq!(CallingConvention::return_register(), "rax");
    }

    #[test]
    fn test_register_info() {
        let rax = RegisterInfo::for_name("rax").unwrap();
        assert!(rax.is_caller_saved);
        assert!(!rax.is_callee_saved);

        let rbx = RegisterInfo::for_name("rbx").unwrap();
        assert!(!rbx.is_caller_saved);
        assert!(rbx.is_callee_saved);
    }
}
