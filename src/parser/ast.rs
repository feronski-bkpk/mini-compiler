//! Абстрактное синтаксическое дерево (AST) для языка MiniC
//!
//! Этот модуль определяет все узлы AST, представляющие структуру программы.

use crate::common::position::Position;
use std::fmt;

/// Базовый узел AST с информацией о позиции в исходном коде
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// Строка в исходном файле (1-индексация)
    pub line: usize,
    /// Колонка в исходном файле (1-индексация)
    pub column: usize,
}

impl Node {
    /// Создает новый узел с указанной позицией
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// Возвращает позицию узла
    pub fn position(&self) -> Position {
        Position::new(self.line, self.column)
    }
}

/// Программа - корневой узел AST
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub node: Node,
    pub declarations: Vec<Declaration>,
}

impl Program {
    pub fn new(declarations: Vec<Declaration>, line: usize, column: usize) -> Self {
        Self {
            node: Node::new(line, column),
            declarations,
        }
    }
}

/// Объявления (верхний уровень)
#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    Function(FunctionDecl),
    Struct(StructDecl),
    Variable(VarDecl),
}

/// Объявление функции
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub node: Node,
    pub name: String,
    pub return_type: Type,
    pub parameters: Vec<Param>,
    pub body: BlockStmt,
}

impl FunctionDecl {
    pub fn new(
        name: String,
        return_type: Type,
        parameters: Vec<Param>,
        body: BlockStmt,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            node: Node::new(line, column),
            name,
            return_type,
            parameters,
            body,
        }
    }
}

/// Параметр функции
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub node: Node,
    pub param_type: Type,
    pub name: String,
}

impl Param {
    pub fn new(param_type: Type, name: String, line: usize, column: usize) -> Self {
        Self {
            node: Node::new(line, column),
            param_type,
            name,
        }
    }
}

/// Объявление структуры
#[derive(Debug, Clone, PartialEq)]
pub struct StructDecl {
    pub node: Node,
    pub name: String,
    pub fields: Vec<VarDecl>,
}

impl StructDecl {
    pub fn new(name: String, fields: Vec<VarDecl>, line: usize, column: usize) -> Self {
        Self {
            node: Node::new(line, column),
            name,
            fields,
        }
    }
}

/// Типы данных
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    Void,
    String,
    Struct(String),
    Inferred,
    Array(Box<Type>, Option<i32>),
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
            Type::Inferred => write!(f, "var"),
            Type::Array(inner, size) => {
                if let Some(s) = size {
                    write!(f, "{}[{}]", inner, s)
                } else {
                    write!(f, "{}[]", inner)
                }
            }
        }
    }
}

impl Type {
    /// Проверяет, является ли тип void
    pub fn is_void(&self) -> bool {
        matches!(self, Type::Void)
    }

    /// Проверяет, является ли тип числовым
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

    /// Проверяет, является ли тип массивом
    pub fn is_array(&self) -> bool {
        matches!(self, Type::Array(_, _))
    }
}

/// Инструкции
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    VariableDecl(VarDecl),
    Expression(ExprStmt),
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Return(ReturnStmt),
    Block(BlockStmt),
    Empty(EmptyStmt),
    Break(BreakStmt),
    Continue(ContinueStmt),
    Switch(SwitchStmt),
}

/// Объявление переменной
#[derive(Debug, Clone, PartialEq)]
pub struct VarDecl {
    pub node: Node,
    pub var_type: Type,
    pub name: String,
    pub initializer: Option<Box<Expression>>,
}

impl VarDecl {
    pub fn new(
        var_type: Type,
        name: String,
        initializer: Option<Expression>,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            node: Node::new(line, column),
            var_type,
            name,
            initializer: initializer.map(Box::new),
        }
    }
}

/// Инструкция-выражение (expression statement)
#[derive(Debug, Clone, PartialEq)]
pub struct ExprStmt {
    pub node: Node,
    pub expr: Box<Expression>,
}

impl ExprStmt {
    pub fn new(expr: Expression, line: usize, column: usize) -> Self {
        Self {
            node: Node::new(line, column),
            expr: Box::new(expr),
        }
    }
}

/// Пустая инструкция (;)
#[derive(Debug, Clone, PartialEq)]
pub struct EmptyStmt {
    pub node: Node,
}

impl EmptyStmt {
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            node: Node::new(line, column),
        }
    }
}

/// Блок инструкций { ... }
#[derive(Debug, Clone, PartialEq)]
pub struct BlockStmt {
    pub node: Node,
    pub statements: Vec<Statement>,
}

impl BlockStmt {
    pub fn new(statements: Vec<Statement>, line: usize, column: usize) -> Self {
        Self {
            node: Node::new(line, column),
            statements,
        }
    }
}

/// Условная инструкция if-else
#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub node: Node,
    pub condition: Box<Expression>,
    pub then_branch: Box<Statement>,
    pub else_branch: Option<Box<Statement>>,
}

impl IfStmt {
    pub fn new(
        condition: Expression,
        then_branch: Statement,
        else_branch: Option<Statement>,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            node: Node::new(line, column),
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        }
    }
}

/// Цикл while
#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt {
    pub node: Node,
    pub condition: Box<Expression>,
    pub body: Box<Statement>,
}

impl WhileStmt {
    pub fn new(condition: Expression, body: Statement, line: usize, column: usize) -> Self {
        Self {
            node: Node::new(line, column),
            condition: Box::new(condition),
            body: Box::new(body),
        }
    }
}

/// Цикл for
#[derive(Debug, Clone, PartialEq)]
pub struct ForStmt {
    pub node: Node,
    pub init: Option<Box<Statement>>,
    pub condition: Option<Box<Expression>>,
    pub update: Option<Box<Expression>>,
    pub body: Box<Statement>,
}

impl ForStmt {
    pub fn new(
        init: Option<Statement>,
        condition: Option<Expression>,
        update: Option<Expression>,
        body: Statement,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            node: Node::new(line, column),
            init: init.map(Box::new),
            condition: condition.map(Box::new),
            update: update.map(Box::new),
            body: Box::new(body),
        }
    }
}

/// Инструкция возврата return
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStmt {
    pub node: Node,
    pub value: Option<Box<Expression>>,
}

impl ReturnStmt {
    pub fn new(value: Option<Expression>, line: usize, column: usize) -> Self {
        Self {
            node: Node::new(line, column),
            value: value.map(Box::new),
        }
    }
}

/// Инструкция break
#[derive(Debug, Clone, PartialEq)]
pub struct BreakStmt {
    pub node: Node,
}

impl BreakStmt {
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            node: Node::new(line, column),
        }
    }
}

/// Инструкция continue
#[derive(Debug, Clone, PartialEq)]
pub struct ContinueStmt {
    pub node: Node,
}

impl ContinueStmt {
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            node: Node::new(line, column),
        }
    }
}

/// Оператор switch
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchStmt {
    pub node: Node,
    pub expression: Box<Expression>,
    pub cases: Vec<CaseStmt>,
    pub default: Option<Box<Statement>>,
}

impl SwitchStmt {
    pub fn new(
        expression: Expression,
        cases: Vec<CaseStmt>,
        default: Option<Statement>,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            node: Node::new(line, column),
            expression: Box::new(expression),
            cases,
            default: default.map(Box::new),
        }
    }
}

/// Ветка case в switch
#[derive(Debug, Clone, PartialEq)]
pub struct CaseStmt {
    pub node: Node,
    pub value: Literal,
    pub body: Box<Statement>,
}

impl CaseStmt {
    pub fn new(value: Literal, body: Statement, line: usize, column: usize) -> Self {
        Self {
            node: Node::new(line, column),
            value,
            body: Box::new(body),
        }
    }
}

/// Выражения
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Identifier(IdentifierExpr),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Assignment(AssignmentExpr),
    Call(CallExpr),
    StructAccess(StructAccessExpr),
    ArrayAccess(ArrayAccessExpr),
    Grouped(GroupedExpr),
}

impl Expression {
    /// Возвращает позицию выражения (узел, с которого начинается выражение)
    pub fn node_position(&self) -> Position {
        match self {
            Expression::Literal(lit) => lit.node.position(),
            Expression::Identifier(ident) => ident.node.position(),
            Expression::Binary(binary) => binary.node.position(),
            Expression::Unary(unary) => unary.node.position(),
            Expression::Assignment(assign) => assign.node.position(),
            Expression::Call(call) => call.node.position(),
            Expression::StructAccess(access) => access.node.position(),
            Expression::ArrayAccess(access) => access.node.position(),
            Expression::Grouped(grouped) => grouped.node.position(),
        }
    }
}

/// Литерал
#[derive(Debug, Clone, PartialEq)]
pub struct Literal {
    pub node: Node,
    pub value: LiteralValue,
}

impl Literal {
    pub fn new(value: LiteralValue, line: usize, column: usize) -> Self {
        Self {
            node: Node::new(line, column),
            value,
        }
    }
}

/// Значения литералов
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Int(i32),
    Float(f64),
    Bool(bool),
    String(String),
}

impl fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiteralValue::Int(i) => write!(f, "{}", i),
            LiteralValue::Float(fl) => write!(f, "{}", fl),
            LiteralValue::Bool(b) => write!(f, "{}", b),
            LiteralValue::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

/// Идентификатор
#[derive(Debug, Clone, PartialEq)]
pub struct IdentifierExpr {
    pub node: Node,
    pub name: String,
}

impl IdentifierExpr {
    pub fn new(name: String, line: usize, column: usize) -> Self {
        Self {
            node: Node::new(line, column),
            name,
        }
    }
}

/// Бинарная операция
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub node: Node,
    pub left: Box<Expression>,
    pub operator: BinaryOp,
    pub right: Box<Expression>,
}

impl BinaryExpr {
    pub fn new(
        left: Expression,
        operator: BinaryOp,
        right: Expression,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            node: Node::new(line, column),
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }
}

/// Унарная операция
#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    pub node: Node,
    pub operator: UnaryOp,
    pub operand: Box<Expression>,
}

impl UnaryExpr {
    pub fn new(operator: UnaryOp, operand: Expression, line: usize, column: usize) -> Self {
        Self {
            node: Node::new(line, column),
            operator,
            operand: Box::new(operand),
        }
    }
}

/// Присваивание
#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentExpr {
    pub node: Node,
    pub target: Box<Expression>,
    pub operator: AssignmentOp,
    pub value: Box<Expression>,
}

impl AssignmentExpr {
    pub fn new(
        target: Expression,
        operator: AssignmentOp,
        value: Expression,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            node: Node::new(line, column),
            target: Box::new(target),
            operator,
            value: Box::new(value),
        }
    }
}

/// Вызов функции
#[derive(Debug, Clone, PartialEq)]
pub struct CallExpr {
    pub node: Node,
    pub callee: Box<Expression>,
    pub arguments: Vec<Expression>,
}

impl CallExpr {
    pub fn new(callee: Expression, arguments: Vec<Expression>, line: usize, column: usize) -> Self {
        Self {
            node: Node::new(line, column),
            callee: Box::new(callee),
            arguments,
        }
    }
}

/// Доступ к полю структуры
#[derive(Debug, Clone, PartialEq)]
pub struct StructAccessExpr {
    pub node: Node,
    pub object: Box<Expression>,
    pub field: String,
}

impl StructAccessExpr {
    pub fn new(object: Expression, field: String, line: usize, column: usize) -> Self {
        Self {
            node: Node::new(line, column),
            object: Box::new(object),
            field,
        }
    }
}

/// Доступ к элементу массива
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayAccessExpr {
    pub node: Node,
    pub array: Box<Expression>,
    pub index: Box<Expression>,
}

impl ArrayAccessExpr {
    pub fn new(array: Expression, index: Expression, line: usize, column: usize) -> Self {
        Self {
            node: Node::new(line, column),
            array: Box::new(array),
            index: Box::new(index),
        }
    }
}

/// Сгруппированное выражение (в скобках)
#[derive(Debug, Clone, PartialEq)]
pub struct GroupedExpr {
    pub node: Node,
    pub expr: Box<Expression>,
}

impl GroupedExpr {
    pub fn new(expr: Expression, line: usize, column: usize) -> Self {
        Self {
            node: Node::new(line, column),
            expr: Box::new(expr),
        }
    }
}

/// Бинарные операторы
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::Mod => write!(f, "%"),
            BinaryOp::Eq => write!(f, "=="),
            BinaryOp::Ne => write!(f, "!="),
            BinaryOp::Lt => write!(f, "<"),
            BinaryOp::Le => write!(f, "<="),
            BinaryOp::Gt => write!(f, ">"),
            BinaryOp::Ge => write!(f, ">="),
            BinaryOp::And => write!(f, "&&"),
            BinaryOp::Or => write!(f, "||"),
        }
    }
}

/// Унарные операторы
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
    Plus,
    PreIncrement,
    PostIncrement,
    PreDecrement,
    PostDecrement,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Neg => write!(f, "-"),
            UnaryOp::Not => write!(f, "!"),
            UnaryOp::Plus => write!(f, "+"),
            UnaryOp::PreIncrement => write!(f, "++ (prefix)"),
            UnaryOp::PostIncrement => write!(f, "++ (postfix)"),
            UnaryOp::PreDecrement => write!(f, "-- (prefix)"),
            UnaryOp::PostDecrement => write!(f, "-- (postfix)"),
        }
    }
}

/// Операторы присваивания
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AssignmentOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
}

impl fmt::Display for AssignmentOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssignmentOp::Assign => write!(f, "="),
            AssignmentOp::AddAssign => write!(f, "+="),
            AssignmentOp::SubAssign => write!(f, "-="),
            AssignmentOp::MulAssign => write!(f, "*="),
            AssignmentOp::DivAssign => write!(f, "/="),
        }
    }
}
