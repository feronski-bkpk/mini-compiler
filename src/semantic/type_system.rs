//! Система типов для семантического анализатора

use std::fmt;

/// Типы данных языка MiniC
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Целочисленный тип
    Int,
    /// Число с плавающей точкой
    Float,
    /// Логический тип
    Bool,
    /// Без типа (для void функций)
    Void,
    /// Строковый тип
    String,
    /// Тип структуры
    Struct(String),
    /// Тип функции (возвращаемый тип и параметры)
    Function {
        return_type: Box<Type>,
        param_types: Vec<Type>,
    },
    /// Тип массива (элемент и размер)
    Array(Box<Type>, usize),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::Void => write!(f, "void"),
            Type::String => write!(f, "string"),
            Type::Struct(name) => write!(f, "struct {}", name),
            Type::Array(inner, size) => write!(f, "{}[{}]", inner, size),
            Type::Function {
                return_type,
                param_types,
            } => {
                write!(f, "fn(")?;
                for (i, param) in param_types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", return_type)
            }
        }
    }
}

impl Type {
    /// Преобразует ast::Type в semantic::Type
    pub fn from_ast(ast_type: &crate::parser::ast::Type) -> Self {
        match ast_type {
            crate::parser::ast::Type::Int => Type::Int,
            crate::parser::ast::Type::Float => Type::Float,
            crate::parser::ast::Type::Bool => Type::Bool,
            crate::parser::ast::Type::Void => Type::Void,
            crate::parser::ast::Type::String => Type::String,
            crate::parser::ast::Type::Struct(name) => Type::Struct(name.clone()),
            crate::parser::ast::Type::Inferred => Type::Int,
            crate::parser::ast::Type::Array(inner, size) => {
                Type::Array(Box::new(Type::from_ast(inner)), size.unwrap_or(0) as usize)
            }
        }
    }

    /// Проверяет, является ли тип числовым (int или float)
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float)
    }

    /// Проверяет, является ли тип целочисленным
    pub fn is_integer(&self) -> bool {
        matches!(self, Type::Int)
    }

    /// Проверяет, является ли тип логическим
    pub fn is_boolean(&self) -> bool {
        matches!(self, Type::Bool)
    }

    /// Проверяет, является ли тип void
    pub fn is_void(&self) -> bool {
        matches!(self, Type::Void)
    }

    /// Проверяет, является ли тип структурой
    pub fn is_struct(&self) -> bool {
        matches!(self, Type::Struct(_))
    }

    /// Проверяет, является ли тип функцией
    pub fn is_function(&self) -> bool {
        matches!(self, Type::Function { .. })
    }

    /// Возвращает размер типа в байтах
    pub fn size(&self) -> Option<usize> {
        match self {
            Type::Int | Type::Bool => Some(4),
            Type::Float => Some(8),
            Type::Void => Some(0),
            Type::String => Some(8),
            Type::Struct(_name) => None,
            Type::Function { .. } => Some(8),
            Type::Array(inner, count) => inner.size().map(|s| s * count),
        }
    }

    /// Вычисляет размер структуры по полям
    pub fn struct_size(fields: &std::collections::HashMap<String, Type>) -> usize {
        let mut total_size = 0;
        for field_type in fields.values() {
            if let Some(size) = field_type.size() {
                total_size += size;
            }
        }
        total_size
    }

    /// Вычисляет смещения для полей структуры (в порядке объявления)
    pub fn struct_offsets(
        fields: &std::collections::HashMap<String, Type>,
        field_order: &[String],
    ) -> std::collections::HashMap<String, usize> {
        let mut offsets = std::collections::HashMap::new();
        let mut current_offset = 0;

        for name in field_order {
            if let Some(field_type) = fields.get(name) {
                if let Some(size) = field_type.size() {
                    offsets.insert(name.clone(), current_offset);
                    current_offset += size;
                }
            }
        }

        offsets
    }

    /// Возвращает выравнивание типа в байтах
    pub fn alignment(&self) -> Option<usize> {
        match self {
            Type::Int | Type::Bool => Some(4),
            Type::Float => Some(8),
            Type::Void => Some(0),
            Type::String => Some(8),
            Type::Struct(_) => None,
            Type::Function { .. } => Some(8),
            Type::Array(inner, count) => inner.size().map(|s| s * count),
        }
    }
}

/// Результат проверки типа
pub type TypeResult<T> = Result<T, TypeError>;

/// Ошибка проверки типов
#[derive(Debug, Clone, PartialEq)]
pub struct TypeError {
    pub expected: Type,
    pub found: Type,
    pub message: String,
}

impl TypeError {
    pub fn new(expected: Type, found: Type, message: String) -> Self {
        Self {
            expected,
            found,
            message,
        }
    }
}

/// Проверщик типов с поддержкой вывода
#[derive(Debug, Default)]
pub struct TypeChecker {
    /// Карта выведенных типов для переменных var
    inferred_types: std::collections::HashMap<String, Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            inferred_types: std::collections::HashMap::new(),
        }
    }

    /// Получает выведенный тип переменной
    pub fn get_inferred_type(&self, var_name: &str) -> Option<&Type> {
        self.inferred_types.get(var_name)
    }

    /// Выводит тип для переменной var
    pub fn infer_type(&mut self, var_name: &str, init_type: &Type) -> Result<Type, TypeError> {
        if let Some(existing) = self.inferred_types.get(var_name) {
            if !self.is_compatible(existing, init_type) {
                return Err(TypeError::new(
                    existing.clone(),
                    init_type.clone(),
                    format!("Несовместимые типы для переменной '{}'", var_name),
                ));
            }
            Ok(existing.clone())
        } else {
            self.inferred_types
                .insert(var_name.to_string(), init_type.clone());
            Ok(init_type.clone())
        }
    }

    /// Проверяет, совместимы ли типы для присваивания
    ///
    /// Правила:
    /// - Точное совпадение типов
    /// - int может быть преобразован в float (расширение)
    /// - float не может быть преобразован в int без явного приведения
    pub fn is_assignable(&self, target: &Type, source: &Type) -> bool {
        match (target, source) {
            (Type::Float, Type::Int) => true,
            (t, s) => self.is_compatible(t, s),
        }
    }

    /// Проверяет, совместимы ли типы для бинарных операций
    pub fn are_compatible_binary(&self, left: &Type, right: &Type) -> bool {
        match (left, right) {
            (Type::Int, Type::Int) => true,
            (Type::Float, Type::Float) => true,
            (Type::Int, Type::Float) => true,
            (Type::Float, Type::Int) => true,
            (Type::Bool, Type::Bool) => true,
            (l, r) if l.is_numeric() && r.is_numeric() => true,
            _ => false,
        }
    }

    /// Возвращает результирующий тип бинарной операции
    pub fn binary_result_type(&self, left: &Type, right: &Type, op: BinaryOpType) -> Option<Type> {
        match op {
            BinaryOpType::Arithmetic | BinaryOpType::ArithmeticAssign => match (left, right) {
                (Type::Float, _) | (_, Type::Float) => Some(Type::Float),
                (Type::Int, Type::Int) => Some(Type::Int),
                _ => None,
            },
            BinaryOpType::Comparison => {
                if left.is_numeric() && right.is_numeric() {
                    Some(Type::Bool)
                } else {
                    None
                }
            }
            BinaryOpType::Logical => {
                if matches!(left, Type::Bool) && matches!(right, Type::Bool) {
                    Some(Type::Bool)
                } else {
                    None
                }
            }
        }
    }

    /// Возвращает результирующий тип унарной операции
    pub fn unary_result_type(&self, operand: &Type, op: UnaryOpType) -> Option<Type> {
        match op {
            UnaryOpType::Neg => {
                if operand.is_numeric() {
                    Some(operand.clone())
                } else {
                    None
                }
            }
            UnaryOpType::Not => {
                if matches!(operand, Type::Bool) {
                    Some(Type::Bool)
                } else {
                    None
                }
            }
            UnaryOpType::Plus => {
                if operand.is_numeric() {
                    Some(operand.clone())
                } else {
                    None
                }
            }
            UnaryOpType::Increment | UnaryOpType::Decrement => {
                if operand.is_numeric() {
                    Some(operand.clone())
                } else {
                    None
                }
            }
        }
    }

    /// Проверяет, совместимы ли типы для сравнения
    pub fn are_comparable(&self, left: &Type, right: &Type) -> bool {
        match (left, right) {
            (Type::Int, Type::Int) => true,
            (Type::Float, Type::Float) => true,
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::String, Type::String) => true,
            _ => false,
        }
    }

    /// Проверяет точное соответствие типов
    pub fn is_compatible(&self, expected: &Type, actual: &Type) -> bool {
        std::mem::discriminant(expected) == std::mem::discriminant(actual)
    }

    /// Получает тип после неявного преобразования для арифметики
    pub fn common_numeric_type(&self, left: &Type, right: &Type) -> Option<Type> {
        match (left, right) {
            (Type::Float, _) | (_, Type::Float) => Some(Type::Float),
            (Type::Int, Type::Int) => Some(Type::Int),
            _ => None,
        }
    }
}

/// Тип бинарной операции для проверки типов
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOpType {
    Arithmetic,
    ArithmeticAssign,
    Comparison,
    Logical,
}

impl From<&crate::parser::ast::BinaryOp> for BinaryOpType {
    fn from(op: &crate::parser::ast::BinaryOp) -> Self {
        use crate::parser::ast::BinaryOp;
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                BinaryOpType::Arithmetic
            }
            BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::Lt
            | BinaryOp::Le
            | BinaryOp::Gt
            | BinaryOp::Ge => BinaryOpType::Comparison,
            BinaryOp::And | BinaryOp::Or => BinaryOpType::Logical,
        }
    }
}

/// Тип унарной операции для проверки типов
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOpType {
    Neg,
    Not,
    Plus,
    Increment,
    Decrement,
}

impl From<&crate::parser::ast::UnaryOp> for UnaryOpType {
    fn from(op: &crate::parser::ast::UnaryOp) -> Self {
        use crate::parser::ast::UnaryOp;
        match op {
            UnaryOp::Neg => UnaryOpType::Neg,
            UnaryOp::Not => UnaryOpType::Not,
            UnaryOp::Plus => UnaryOpType::Plus,
            UnaryOp::PreIncrement | UnaryOp::PostIncrement => UnaryOpType::Increment,
            UnaryOp::PreDecrement | UnaryOp::PostDecrement => UnaryOpType::Decrement,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_compatibility() {
        let checker = TypeChecker::new();

        assert!(checker.is_assignable(&Type::Int, &Type::Int));
        assert!(checker.is_assignable(&Type::Float, &Type::Int));
        assert!(!checker.is_assignable(&Type::Int, &Type::Float));
        assert!(!checker.is_assignable(&Type::Bool, &Type::Int));
    }

    #[test]
    fn test_binary_result_type() {
        let checker = TypeChecker::new();

        assert_eq!(
            checker.binary_result_type(&Type::Int, &Type::Int, BinaryOpType::Arithmetic),
            Some(Type::Int)
        );
        assert_eq!(
            checker.binary_result_type(&Type::Int, &Type::Float, BinaryOpType::Arithmetic),
            Some(Type::Float)
        );
        assert_eq!(
            checker.binary_result_type(&Type::Bool, &Type::Bool, BinaryOpType::Logical),
            Some(Type::Bool)
        );
    }
}
