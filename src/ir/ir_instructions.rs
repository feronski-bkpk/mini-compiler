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
    Array(Box<IRType>, usize),
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
            IRType::Array(inner, size) => write!(f, "{}[{}]", inner, size),
            IRType::Unknown => write!(f, "?"),
        }
    }
}

impl IRType {
    pub fn size(&self) -> usize {
        match self {
            IRType::Int => 4,
            IRType::Float => 8,
            IRType::Bool => 1,
            IRType::Void => 0,
            IRType::String => 8,
            IRType::Struct(_) => 0,
            IRType::Pointer(_) => 8,
            IRType::Array(inner, count) => inner.size() * count,
            IRType::Unknown => 0,
        }
    }
}

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

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Temporary(String),
    Variable(String),
    IntLiteral(i32),
    FloatLiteral(f64),
    BoolLiteral(bool),
    StringLiteral(String),
    Label(String),
    MemoryAddress {
        base: String,
        offset: i32,
    },
    ArrayAccess {
        base: String,
        index: Box<Operand>,
        stride: usize,
    },
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
            Operand::ArrayAccess {
                base,
                index,
                stride,
            } => {
                base.hash(state);
                index.hash(state);
                stride.hash(state);
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
            Operand::ArrayAccess {
                base,
                index,
                stride,
            } => {
                write!(f, "{}[{}*{}]", base, index, stride)
            }
        }
    }
}

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

    CmpEqF(Operand, Operand, Operand),
    CmpNeF(Operand, Operand, Operand),
    CmpLtF(Operand, Operand, Operand),
    CmpLeF(Operand, Operand, Operand),
    CmpGtF(Operand, Operand, Operand),
    CmpGeF(Operand, Operand, Operand),

    CmpLtU(Operand, Operand, Operand),
    CmpLeU(Operand, Operand, Operand),
    CmpGtU(Operand, Operand, Operand),
    CmpGeU(Operand, Operand, Operand),

    Load(Operand, Operand),
    Store(Operand, Operand),
    Alloca(Operand, u32),
    Gep(Operand, Operand, u32),

    ArrayLoad(Operand, Operand, Operand),
    ArrayStore(Operand, Operand, Operand),

    Jump(Operand),
    JumpIf(Operand, Operand),
    JumpIfNot(Operand, Operand),
    Label(Operand),
    Phi(Operand, Vec<(Operand, Operand)>),

    Call(Operand, Operand, Vec<Operand>),
    Return(Option<Operand>),
    Param(u32, Operand),

    Move(Operand, Operand),

    IntToFloat(Operand, Operand),
    FloatToInt(Operand, Operand),
}

impl IRInstruction {
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
            IRInstruction::CmpEqF(_, s1, s2) => vec![s1, s2],
            IRInstruction::CmpNeF(_, s1, s2) => vec![s1, s2],
            IRInstruction::CmpLtF(_, s1, s2) => vec![s1, s2],
            IRInstruction::CmpLeF(_, s1, s2) => vec![s1, s2],
            IRInstruction::CmpGtF(_, s1, s2) => vec![s1, s2],
            IRInstruction::CmpGeF(_, s1, s2) => vec![s1, s2],
            IRInstruction::CmpLtU(_, s1, s2) => vec![s1, s2],
            IRInstruction::CmpLeU(_, s1, s2) => vec![s1, s2],
            IRInstruction::CmpGtU(_, s1, s2) => vec![s1, s2],
            IRInstruction::CmpGeU(_, s1, s2) => vec![s1, s2],
            IRInstruction::Load(_, a) => vec![a],
            IRInstruction::Store(a, s) => vec![a, s],
            IRInstruction::Alloca(_, _) => vec![],
            IRInstruction::Gep(_, b, _) => vec![b],
            IRInstruction::ArrayLoad(_, base, index) => vec![base, index],
            IRInstruction::ArrayStore(base, index, val) => vec![base, index, val],
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
            IRInstruction::IntToFloat(_, s) => vec![s],
            IRInstruction::FloatToInt(_, s) => vec![s],
        }
    }

    pub fn is_terminator(&self) -> bool {
        matches!(
            self,
            IRInstruction::Jump(_)
                | IRInstruction::JumpIf(_, _)
                | IRInstruction::JumpIfNot(_, _)
                | IRInstruction::Return(_)
        )
    }

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
            | IRInstruction::CmpGe(_, _, _)
            | IRInstruction::CmpEqF(_, _, _)
            | IRInstruction::CmpNeF(_, _, _)
            | IRInstruction::CmpLtF(_, _, _)
            | IRInstruction::CmpLeF(_, _, _)
            | IRInstruction::CmpGtF(_, _, _)
            | IRInstruction::CmpGeF(_, _, _)
            | IRInstruction::CmpLtU(_, _, _)
            | IRInstruction::CmpLeU(_, _, _)
            | IRInstruction::CmpGtU(_, _, _)
            | IRInstruction::CmpGeU(_, _, _) => Some(IRType::Bool),
            IRInstruction::IntToFloat(_, _) => Some(IRType::Float),
            IRInstruction::FloatToInt(_, _) => Some(IRType::Int),
            IRInstruction::ArrayLoad(_, _, _) => Some(IRType::Int),
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
            IRInstruction::CmpEqF(d, s1, s2) => write!(f, "{} = CMP_EQF {}, {}", d, s1, s2),
            IRInstruction::CmpNeF(d, s1, s2) => write!(f, "{} = CMP_NEF {}, {}", d, s1, s2),
            IRInstruction::CmpLtF(d, s1, s2) => write!(f, "{} = CMP_LTF {}, {}", d, s1, s2),
            IRInstruction::CmpLeF(d, s1, s2) => write!(f, "{} = CMP_LEF {}, {}", d, s1, s2),
            IRInstruction::CmpGtF(d, s1, s2) => write!(f, "{} = CMP_GTF {}, {}", d, s1, s2),
            IRInstruction::CmpGeF(d, s1, s2) => write!(f, "{} = CMP_GEF {}, {}", d, s1, s2),
            IRInstruction::CmpLtU(d, s1, s2) => write!(f, "{} = CMP_LTU {}, {}", d, s1, s2),
            IRInstruction::CmpLeU(d, s1, s2) => write!(f, "{} = CMP_LEU {}, {}", d, s1, s2),
            IRInstruction::CmpGtU(d, s1, s2) => write!(f, "{} = CMP_GTU {}, {}", d, s1, s2),
            IRInstruction::CmpGeU(d, s1, s2) => write!(f, "{} = CMP_GEU {}, {}", d, s1, s2),
            IRInstruction::Load(d, a) => write!(f, "{} = LOAD {}", d, a),
            IRInstruction::Store(a, s) => write!(f, "STORE {}, {}", a, s),
            IRInstruction::Alloca(d, size) => write!(f, "{} = ALLOCA {}", d, size),
            IRInstruction::Gep(d, b, idx) => write!(f, "{} = GEP {}, {}", d, b, idx),
            IRInstruction::ArrayLoad(d, base, index) => {
                write!(f, "{} = ARRAY_LOAD {}, {}", d, base, index)
            }
            IRInstruction::ArrayStore(base, index, val) => {
                write!(f, "ARRAY_STORE {}, {}, {}", base, index, val)
            }
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
            IRInstruction::IntToFloat(d, s) => write!(f, "{} = INT_TO_FLOAT {}", d, s),
            IRInstruction::FloatToInt(d, s) => write!(f, "{} = FLOAT_TO_INT {}", d, s),
        }
    }
}
