//! Генератор выражений
//!
//! Отвечает за трансляцию арифметических и логических выражений
//! из IR в x86-64 ассемблер. Основная логика в `x86_generator.rs`.
//!
//! В будущем здесь можно реализовать:
//! - Оптимизацию вычисления выражений
//! - Лучшее распределение регистров
//! - Поддержку векторных инструкций (SSE/AVX)

/// Приоритет операций для генерации выражений
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ExpressionPriority {
    Primary = 1,
    Unary = 2,
    Multiplicative = 3,
    Additive = 4,
    Relational = 5,
    Equality = 6,
    LogicalAnd = 7,
    LogicalOr = 8,
    Assignment = 9,
}
