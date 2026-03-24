//! Определения инструкций IR, операндов и типов

use std::fmt;
use std::hash::{Hash, Hasher};

/// Тип данных в IR
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IRType {
    Int,
    Float,
    Bool,
    Void,
    String,
    Struct(String),
    Pointer(Box<IRType>),
    Unknown,
}

impl fmt::Display for IRType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IRType::Int => write!(f, "i32"),
            IRType::Float => write!(f, "f64"),
            IRType::Bool => write!(f, "bool"),
            IRType::Void => write!(f, "void"),
            IRType::String => write!(f, "string"),
            IRType::Struct(name) => write!(f, "struct {}", name),
            IRType::Pointer(inner) => write!(f, "{}*", inner),
            IRType::Unknown => write!(f, "?"),
        }
    }
}

impl IRType {
    /// Возвращает размер типа в байтах
    pub fn size(&self) -> usize {
        match self {
            IRType::Int => 4,
            IRType::Float => 8,
            IRType::Bool => 1,
            IRType::Void => 0,
            IRType::String => 8,
            IRType::Struct(_) => 0,
            IRType::Pointer(_) => 8,
            IRType::Unknown => 0,
        }
    }
}

/// Тип операнда в IR (без Hash для Operand)
#[derive(Debug, Clone, PartialEq)]
pub struct TypedOperand {
    pub operand: Operand,
    pub typ: IRType,
}

impl TypedOperand {
    pub fn new(operand: Operand, typ: IRType) -> Self {
        Self { operand, typ }
    }

    pub fn with_type(mut self, typ: IRType) -> Self {
        self.typ = typ;
        self
    }
}

/// Тип операнда в IR
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    /// Временная переменная (виртуальный регистр)
    Temporary(String),
    /// Имя переменной
    Variable(String),
    /// Целочисленный литерал
    IntLiteral(i32),
    /// Вещественный литерал
    FloatLiteral(f64),
    /// Булев литерал
    BoolLiteral(bool),
    /// Строковый литерал
    StringLiteral(String),
    /// Метка базового блока
    Label(String),
    /// Адрес в памяти (база + смещение)
    MemoryAddress { base: String, offset: i32 },
}

impl Hash for Operand {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Operand::Temporary(s) => s.hash(state),
            Operand::Variable(s) => s.hash(state),
            Operand::IntLiteral(i) => i.hash(state),
            Operand::FloatLiteral(f) => f.to_bits().hash(state),
            Operand::BoolLiteral(b) => b.hash(state),
            Operand::StringLiteral(s) => s.hash(state),
            Operand::Label(s) => s.hash(state),
            Operand::MemoryAddress { base, offset } => {
                base.hash(state);
                offset.hash(state);
            }
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Temporary(name) => write!(f, "{}", name),
            Operand::Variable(name) => write!(f, "{}", name),
            Operand::IntLiteral(val) => write!(f, "{}", val),
            Operand::FloatLiteral(val) => write!(f, "{}", val),
            Operand::BoolLiteral(val) => write!(f, "{}", val),
            Operand::StringLiteral(val) => write!(f, "\"{}\"", val),
            Operand::Label(name) => write!(f, "{}", name),
            Operand::MemoryAddress { base, offset } => {
                if *offset == 0 {
                    write!(f, "[{}]", base)
                } else {
                    write!(f, "[{}+{}]", base, offset)
                }
            }
        }
    }
}

/// Инструкции IR (трехадресный код)
#[derive(Debug, Clone, PartialEq)]
pub enum IRInstruction {
    Add(Operand, Operand, Operand),
    Sub(Operand, Operand, Operand),
    Mul(Operand, Operand, Operand),
    Div(Operand, Operand, Operand),
    Mod(Operand, Operand, Operand),
    Neg(Operand, Operand),

    And(Operand, Operand, Operand),
    Or(Operand, Operand, Operand),
    Not(Operand, Operand),
    Xor(Operand, Operand, Operand),

    CmpEq(Operand, Operand, Operand),
    CmpNe(Operand, Operand, Operand),
    CmpLt(Operand, Operand, Operand),
    CmpLe(Operand, Operand, Operand),
    CmpGt(Operand, Operand, Operand),
    CmpGe(Operand, Operand, Operand),

    Load(Operand, Operand),
    Store(Operand, Operand),
    Alloca(Operand, u32),
    Gep(Operand, Operand, u32),

    Jump(Operand),
    JumpIf(Operand, Operand),
    JumpIfNot(Operand, Operand),
    Label(Operand),
    Phi(Operand, Vec<(Operand, Operand)>),

    Call(Operand, Operand, Vec<Operand>),
    Return(Option<Operand>),
    Param(u32, Operand),

    Move(Operand, Operand),
}

impl IRInstruction {
    /// Возвращает все операнды инструкции
    pub fn operands(&self) -> Vec<&Operand> {
        match self {
            IRInstruction::Add(_, s1, s2) => vec![s1, s2],
            IRInstruction::Sub(_, s1, s2) => vec![s1, s2],
            IRInstruction::Mul(_, s1, s2) => vec![s1, s2],
            IRInstruction::Div(_, s1, s2) => vec![s1, s2],
            IRInstruction::Mod(_, s1, s2) => vec![s1, s2],
            IRInstruction::Neg(_, s) => vec![s],
            IRInstruction::And(_, s1, s2) => vec![s1, s2],
            IRInstruction::Or(_, s1, s2) => vec![s1, s2],
            IRInstruction::Not(_, s) => vec![s],
            IRInstruction::Xor(_, s1, s2) => vec![s1, s2],
            IRInstruction::CmpEq(_, s1, s2) => vec![s1, s2],
            IRInstruction::CmpNe(_, s1, s2) => vec![s1, s2],
            IRInstruction::CmpLt(_, s1, s2) => vec![s1, s2],
            IRInstruction::CmpLe(_, s1, s2) => vec![s1, s2],
            IRInstruction::CmpGt(_, s1, s2) => vec![s1, s2],
            IRInstruction::CmpGe(_, s1, s2) => vec![s1, s2],
            IRInstruction::Load(_, a) => vec![a],
            IRInstruction::Store(a, s) => vec![a, s],
            IRInstruction::Alloca(_, _) => vec![],
            IRInstruction::Gep(_, b, _) => vec![b],
            IRInstruction::Jump(l) => vec![l],
            IRInstruction::JumpIf(c, l) => vec![c, l],
            IRInstruction::JumpIfNot(c, l) => vec![c, l],
            IRInstruction::Label(_) => vec![],
            IRInstruction::Phi(_, pairs) => pairs.iter().flat_map(|(val, _)| vec![val]).collect(),
            IRInstruction::Call(_, f, args) => {
                let mut ops = vec![f];
                ops.extend(args);
                ops
            }
            IRInstruction::Return(Some(v)) => vec![v],
            IRInstruction::Return(None) => vec![],
            IRInstruction::Param(_, v) => vec![v],
            IRInstruction::Move(_, s) => vec![s],
        }
    }

    /// Проверяет, является ли инструкция управляющей (завершает блок)
    pub fn is_terminator(&self) -> bool {
        matches!(
            self,
            IRInstruction::Jump(_)
                | IRInstruction::JumpIf(_, _)
                | IRInstruction::JumpIfNot(_, _)
                | IRInstruction::Return(_)
        )
    }

    /// Возвращает тип результата инструкции
    pub fn result_type(
        &self,
        operand_types: &std::collections::HashMap<String, IRType>,
    ) -> Option<IRType> {
        match self {
            IRInstruction::Add(_, _, _)
            | IRInstruction::Sub(_, _, _)
            | IRInstruction::Mul(_, _, _)
            | IRInstruction::Div(_, _, _)
            | IRInstruction::Mod(_, _, _) => Some(IRType::Int),
            IRInstruction::CmpEq(_, _, _)
            | IRInstruction::CmpNe(_, _, _)
            | IRInstruction::CmpLt(_, _, _)
            | IRInstruction::CmpLe(_, _, _)
            | IRInstruction::CmpGt(_, _, _)
            | IRInstruction::CmpGe(_, _, _) => Some(IRType::Bool),
            IRInstruction::Load(d, _) => {
                if let Operand::Temporary(name) = d {
                    operand_types.get(name).cloned()
                } else {
                    None
                }
            }
            IRInstruction::Call(d, _, _) => {
                if let Operand::Temporary(name) = d {
                    operand_types.get(name).cloned()
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl fmt::Display for IRInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IRInstruction::Add(d, s1, s2) => write!(f, "{} = ADD {}, {}", d, s1, s2),
            IRInstruction::Sub(d, s1, s2) => write!(f, "{} = SUB {}, {}", d, s1, s2),
            IRInstruction::Mul(d, s1, s2) => write!(f, "{} = MUL {}, {}", d, s1, s2),
            IRInstruction::Div(d, s1, s2) => write!(f, "{} = DIV {}, {}", d, s1, s2),
            IRInstruction::Mod(d, s1, s2) => write!(f, "{} = MOD {}, {}", d, s1, s2),
            IRInstruction::Neg(d, s) => write!(f, "{} = NEG {}", d, s),
            IRInstruction::And(d, s1, s2) => write!(f, "{} = AND {}, {}", d, s1, s2),
            IRInstruction::Or(d, s1, s2) => write!(f, "{} = OR {}, {}", d, s1, s2),
            IRInstruction::Not(d, s) => write!(f, "{} = NOT {}", d, s),
            IRInstruction::Xor(d, s1, s2) => write!(f, "{} = XOR {}, {}", d, s1, s2),
            IRInstruction::CmpEq(d, s1, s2) => write!(f, "{} = CMP_EQ {}, {}", d, s1, s2),
            IRInstruction::CmpNe(d, s1, s2) => write!(f, "{} = CMP_NE {}, {}", d, s1, s2),
            IRInstruction::CmpLt(d, s1, s2) => write!(f, "{} = CMP_LT {}, {}", d, s1, s2),
            IRInstruction::CmpLe(d, s1, s2) => write!(f, "{} = CMP_LE {}, {}", d, s1, s2),
            IRInstruction::CmpGt(d, s1, s2) => write!(f, "{} = CMP_GT {}, {}", d, s1, s2),
            IRInstruction::CmpGe(d, s1, s2) => write!(f, "{} = CMP_GE {}, {}", d, s1, s2),
            IRInstruction::Load(d, a) => write!(f, "{} = LOAD {}", d, a),
            IRInstruction::Store(a, s) => write!(f, "STORE {}, {}", a, s),
            IRInstruction::Alloca(d, size) => write!(f, "{} = ALLOCA {}", d, size),
            IRInstruction::Gep(d, b, idx) => write!(f, "{} = GEP {}, {}", d, b, idx),
            IRInstruction::Jump(l) => write!(f, "JUMP {}", l),
            IRInstruction::JumpIf(c, l) => write!(f, "JUMP_IF {}, {}", c, l),
            IRInstruction::JumpIfNot(c, l) => write!(f, "JUMP_IF_NOT {}, {}", c, l),
            IRInstruction::Label(l) => write!(f, "{}:", l),
            IRInstruction::Phi(d, pairs) => {
                write!(f, "{} = PHI ", d)?;
                for (i, (val, block)) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "({}, {})", val, block)?;
                }
                Ok(())
            }
            IRInstruction::Call(d, fname, args) => {
                write!(f, "{} = CALL {}", d, fname)?;
                for arg in args {
                    write!(f, ", {}", arg)?;
                }
                Ok(())
            }
            IRInstruction::Return(Some(v)) => write!(f, "RETURN {}", v),
            IRInstruction::Return(None) => write!(f, "RETURN"),
            IRInstruction::Param(idx, v) => write!(f, "PARAM {}, {}", idx, v),
            IRInstruction::Move(d, s) => write!(f, "{} = MOVE {}", d, s),
        }
    }
}
