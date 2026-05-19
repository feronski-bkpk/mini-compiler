//! Основной семантический анализатор

use crate::parser::ast::*;
use crate::semantic::errors::{SemanticError, SemanticErrorKind, SemanticErrors};
use crate::semantic::symbol_table::{Symbol, SymbolTable};
use crate::semantic::type_system::{BinaryOpType, Type, TypeChecker, UnaryOpType};

/// Декорированный AST с аннотациями типов
#[derive(Debug, Clone)]
pub struct DecoratedNode {
    pub node: Node,
    pub typ: Option<Type>,
    pub symbol: Option<Symbol>,
}

impl DecoratedNode {
    pub fn new(node: Node) -> Self {
        Self {
            node,
            typ: None,
            symbol: None,
        }
    }

    pub fn with_type(mut self, typ: Type) -> Self {
        self.typ = Some(typ);
        self
    }

    pub fn with_symbol(mut self, symbol: Symbol) -> Self {
        self.symbol = Some(symbol);
        self
    }
}

/// Результат семантического анализа
#[derive(Debug)]
pub struct SemanticOutput {
    pub decorated_ast: Option<Program>,
    pub symbol_table: SymbolTable,
    pub errors: SemanticErrors,
}

impl SemanticOutput {
    pub fn new(ast: Option<Program>, symbol_table: SymbolTable, errors: SemanticErrors) -> Self {
        Self {
            decorated_ast: ast,
            symbol_table,
            errors,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.errors.has_errors()
    }

    pub fn has_errors(&self) -> bool {
        self.errors.has_errors()
    }
}

/// Семантический анализатор
pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    type_checker: TypeChecker,
    errors: SemanticErrors,
    current_function: Option<Symbol>,
    has_return: bool,
    loop_depth: usize,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            type_checker: TypeChecker::new(),
            errors: SemanticErrors::new(),
            current_function: None,
            has_return: false,
            loop_depth: 0,
        }
    }

    pub fn with_max_errors(mut self, max: usize) -> Self {
        self.errors = self.errors.with_max_errors(max);
        self
    }

    pub fn analyze(&mut self, program: Program) -> SemanticOutput {
        self.collect_declarations(&program);
        self.analyze_program(&program);
        SemanticOutput::new(
            Some(program),
            self.symbol_table.clone(),
            self.errors.clone(),
        )
    }

    fn collect_declarations(&mut self, program: &Program) {
        for decl in &program.declarations {
            match decl {
                Declaration::Function(func) => self.collect_function(func),
                Declaration::Struct(struct_decl) => self.collect_struct(struct_decl),
                Declaration::Variable(var) => self.collect_global_variable(var),
                Declaration::ExternFunction(ext) => {
                    let param_types: Vec<Type> = ext
                        .parameters
                        .iter()
                        .map(|p| Type::from_ast(&p.param_type))
                        .collect();
                    let return_type = Type::from_ast(&ext.return_type);
                    let symbol = Symbol::function(
                        ext.name.clone(),
                        return_type,
                        param_types,
                        ext.is_variadic,
                        ext.node.position(),
                    );
                    self.symbol_table.insert(&ext.name, symbol);
                }
            }
        }
    }

    fn collect_function(&mut self, func: &FunctionDecl) {
        let param_types: Vec<Type> = func
            .parameters
            .iter()
            .map(|p| Type::from_ast(&p.param_type))
            .collect();
        let return_type = Type::from_ast(&func.return_type);
        let symbol = Symbol::function(
            func.name.clone(),
            return_type,
            param_types,
            func.is_variadic,
            func.node.position(),
        );
        if !self.symbol_table.insert(&func.name, symbol) {
            self.errors.add(
                SemanticError::new(
                    SemanticErrorKind::DuplicateDeclaration,
                    func.node.position(),
                    format!("Функция '{}' уже объявлена", func.name),
                )
                .with_suggestion(
                    "Используйте другое имя функции или удалите предыдущее объявление".to_string(),
                ),
            );
        }
    }

    fn collect_struct(&mut self, struct_decl: &StructDecl) {
        let mut fields = std::collections::HashMap::new();
        let mut field_types = std::collections::HashMap::new();
        let mut field_symbols = std::collections::HashMap::new();
        let mut field_order = Vec::new();

        for field in &struct_decl.fields {
            let field_type = Type::from_ast(&field.var_type);
            if fields.contains_key(&field.name) {
                self.errors.add(
                    SemanticError::new(
                        SemanticErrorKind::DuplicateDeclaration,
                        field.node.position(),
                        format!(
                            "Поле '{}' уже объявлено в структуре '{}'",
                            field.name, struct_decl.name
                        ),
                    )
                    .with_suggestion("Используйте другое имя поля".to_string()),
                );
            } else {
                field_order.push(field.name.clone());
                field_types.insert(field.name.clone(), field_type.clone());
                let field_symbol = Symbol::field(
                    field.name.clone(),
                    field_type.clone(),
                    field.node.position(),
                );
                field_symbols.insert(field.name.clone(), field_symbol);
                fields.insert(field.name.clone(), field_type);
            }
        }

        let struct_offsets = Type::struct_offsets(&field_types, &field_order);
        for (field_name, offset) in struct_offsets {
            if let Some(field_symbol) = field_symbols.get_mut(&field_name) {
                field_symbol.stack_offset = Some(offset as i32);
            }
        }

        let symbol = Symbol::struct_type(
            struct_decl.name.clone(),
            fields,
            struct_decl.node.position(),
        );
        if !self.symbol_table.insert(&struct_decl.name, symbol) {
            self.errors.add(
                SemanticError::new(
                    SemanticErrorKind::DuplicateDeclaration,
                    struct_decl.node.position(),
                    format!("Структура '{}' уже объявлена", struct_decl.name),
                )
                .with_suggestion("Используйте другое имя структуры".to_string()),
            );
        }
    }

    fn collect_global_variable(&mut self, var: &VarDecl) {
        let var_type = match &var.var_type {
            crate::parser::ast::Type::Inferred => {
                if var.initializer.is_none() {
                    self.errors.add(
                        SemanticError::new(
                            SemanticErrorKind::InvalidExpression,
                            var.node.position(),
                            format!(
                                "Глобальная переменная '{}' с типом var требует инициализатора",
                                var.name
                            ),
                        )
                        .with_suggestion("Добавьте инициализатор для переменной var".to_string()),
                    );
                    return;
                }
                Type::Int
            }
            _ => Type::from_ast(&var.var_type),
        };
        let symbol = Symbol::variable(var.name.clone(), var_type, var.node.position());
        if !self.symbol_table.insert(&var.name, symbol) {
            self.errors.add(
                SemanticError::new(
                    SemanticErrorKind::DuplicateDeclaration,
                    var.node.position(),
                    format!("Переменная '{}' уже объявлена в этой области", var.name),
                )
                .with_suggestion("Используйте другое имя переменной".to_string()),
            );
        }
    }

    fn analyze_program(&mut self, program: &Program) {
        for decl in &program.declarations {
            match decl {
                Declaration::Function(func) => self.analyze_function(func),
                Declaration::Struct(_) => {}
                Declaration::ExternFunction(ext) => {
                    let param_types: Vec<Type> = ext
                        .parameters
                        .iter()
                        .map(|p| Type::from_ast(&p.param_type))
                        .collect();
                    let return_type = Type::from_ast(&ext.return_type);
                    let symbol = Symbol::function(
                        ext.name.clone(),
                        return_type,
                        param_types,
                        ext.is_variadic,
                        ext.node.position(),
                    );
                    self.symbol_table.insert(&ext.name, symbol);
                }
                Declaration::Variable(var) => {
                    if let crate::parser::ast::Type::Inferred = &var.var_type {
                        if let Some(initializer) = &var.initializer {
                            let init_type = self.analyze_expression(initializer);
                            if let Some(init_type) = init_type {
                                let new_symbol = Symbol::variable(
                                    var.name.clone(),
                                    init_type.clone(),
                                    var.node.position(),
                                );
                                self.symbol_table.update_symbol(&var.name, &new_symbol);
                            }
                        }
                    } else if let Some(initializer) = &var.initializer {
                        let init_type = self.analyze_expression(initializer);
                        let var_type = Type::from_ast(&var.var_type);
                        if let Some(init_type) = init_type {
                            if !self.type_checker.is_assignable(&var_type, &init_type) {
                                self.errors.add(
                                    SemanticError::new(
                                        SemanticErrorKind::AssignmentTypeMismatch,
                                        var.node.position(),
                                        format!("Несоответствие типов при инициализации глобальной переменной '{}'", var.name),
                                    )
                                    .with_types(var_type, init_type),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    fn analyze_function(&mut self, func: &FunctionDecl) {
        self.symbol_table.enter_scope();
        let previous_function = self.current_function.take();
        let return_type = Type::from_ast(&func.return_type);
        self.current_function = Some(Symbol::function(
            func.name.clone(),
            return_type,
            vec![],
            func.is_variadic,
            func.node.position(),
        ));
        self.has_return = false;

        for param in &func.parameters {
            let param_type = Type::from_ast(&param.param_type);
            let symbol = Symbol::parameter(param.name.clone(), param_type, param.node.position());
            if !self.symbol_table.insert(&param.name, symbol) {
                self.errors.add(
                    SemanticError::new(
                        SemanticErrorKind::DuplicateDeclaration,
                        param.node.position(),
                        format!("Параметр '{}' уже объявлен", param.name),
                    )
                    .with_suggestion("Используйте другое имя параметра".to_string()),
                );
            }
        }

        self.analyze_block(&func.body);

        if !func.return_type.is_void() && !self.has_return {
            self.errors.add(
                SemanticError::new(
                    SemanticErrorKind::InvalidReturnType,
                    func.node.position(),
                    format!("Функция '{}' должна возвращать значение", func.name),
                )
                .with_suggestion(format!("Добавьте 'return <значение>;' в конец функции или измените возвращаемый тип на void")),
            );
        }

        self.current_function = previous_function;
        self.symbol_table.exit_scope();
    }

    fn analyze_variable_decl(&mut self, var: &VarDecl) {
        if self.symbol_table.exists_local(&var.name) {
            self.errors.add(
                SemanticError::new(
                    SemanticErrorKind::DuplicateDeclaration,
                    var.node.position(),
                    format!(
                        "Переменная '{}' уже объявлена в этой области видимости",
                        var.name
                    ),
                )
                .with_suggestion(
                    "Используйте другое имя переменной или удалите предыдущее объявление"
                        .to_string(),
                ),
            );
            return;
        }

        let var_type = match &var.var_type {
            crate::parser::ast::Type::Inferred => {
                if let Some(initializer) = &var.initializer {
                    let init_type = self.analyze_expression(initializer);
                    if let Some(init_type) = init_type {
                        match self.type_checker.infer_type(&var.name, &init_type) {
                            Ok(inferred) => inferred,
                            Err(e) => {
                                self.errors.add(
                                    SemanticError::new(
                                        SemanticErrorKind::TypeMismatch,
                                        var.node.position(),
                                        e.message,
                                    )
                                    .with_types(e.expected, e.found),
                                );
                                return;
                            }
                        }
                    } else {
                        self.errors.add(
                            SemanticError::new(
                                SemanticErrorKind::InvalidExpression,
                                var.node.position(),
                                format!("Не удалось вывести тип для переменной '{}'", var.name),
                            )
                            .with_suggestion(
                                "Убедитесь, что инициализатор имеет корректный тип".to_string(),
                            ),
                        );
                        return;
                    }
                } else {
                    self.errors.add(
                        SemanticError::new(
                            SemanticErrorKind::InvalidExpression,
                            var.node.position(),
                            format!(
                                "Переменная '{}' с типом var требует инициализатора",
                                var.name
                            ),
                        )
                        .with_suggestion("Добавьте инициализатор для переменной var".to_string()),
                    );
                    return;
                }
            }
            _ => Type::from_ast(&var.var_type),
        };

        if let Some(init) = &var.initializer {
            if let Expression::ArrayInitializer(arr_init) = init.as_ref() {
                if let crate::parser::ast::Type::Array(_, Some(max_size)) = &var.var_type {
                    if arr_init.elements.len() as i32 > *max_size {
                        self.errors.add(
                            SemanticError::new(
                                SemanticErrorKind::TypeMismatch,
                                var.node.position(),
                                format!(
                                    "Слишком много элементов в инициализаторе массива: объявлено [{}], получено {}",
                                    max_size,
                                    arr_init.elements.len()
                                ),
                            ),
                        );
                    }
                }
            } else {
                let init_type = self.analyze_expression(init);
                if let Some(init_type) = init_type {
                    if !self.type_checker.is_assignable(&var_type, &init_type) {
                        self.errors.add(
                            SemanticError::new(
                                SemanticErrorKind::AssignmentTypeMismatch,
                                var.node.position(),
                                format!(
                                    "Несоответствие типов при инициализации переменной '{}'",
                                    var.name
                                ),
                            )
                            .with_types(var_type.clone(), init_type.clone())
                            .with_suggestion(format!(
                                "Ожидался тип {}, получен {}",
                                var_type, init_type
                            )),
                        );
                    }
                }
            }
        }

        let symbol = Symbol::variable(var.name.clone(), var_type, var.node.position());
        self.symbol_table.insert_with_offset(&var.name, symbol);
    }

    fn analyze_block(&mut self, block: &BlockStmt) {
        self.symbol_table.enter_scope();
        for stmt in &block.statements {
            self.analyze_statement(stmt);
        }
        self.symbol_table.exit_scope();
    }

    fn analyze_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::VariableDecl(var) => self.analyze_variable_decl(var),
            Statement::Expression(expr_stmt) => {
                self.analyze_expression(&expr_stmt.expr);
            }
            Statement::If(if_stmt) => self.analyze_if(if_stmt),
            Statement::While(while_stmt) => self.analyze_while(while_stmt),
            Statement::For(for_stmt) => self.analyze_for(for_stmt),
            Statement::Return(return_stmt) => self.analyze_return(return_stmt),
            Statement::Block(block) => self.analyze_block(block),
            Statement::Break(break_stmt) => {
                if self.loop_depth == 0 {
                    self.errors.add(
                        SemanticError::new(
                            SemanticErrorKind::InvalidBreak,
                            break_stmt.node.position(),
                            "break используется вне цикла".to_string(),
                        )
                        .with_suggestion(
                            "break можно использовать только внутри while или for".to_string(),
                        ),
                    );
                }
            }
            Statement::Continue(continue_stmt) => {
                if self.loop_depth == 0 {
                    self.errors.add(
                        SemanticError::new(
                            SemanticErrorKind::InvalidContinue,
                            continue_stmt.node.position(),
                            "continue используется вне цикла".to_string(),
                        )
                        .with_suggestion(
                            "continue можно использовать только внутри while или for".to_string(),
                        ),
                    );
                }
            }
            Statement::Switch(switch_stmt) => self.analyze_switch(switch_stmt),
            Statement::Empty(_) => {}
        }
    }

    fn analyze_if(&mut self, if_stmt: &IfStmt) {
        let cond_type = self.analyze_expression(&if_stmt.condition);
        if let Some(cond_type) = cond_type {
            if !self.type_checker.is_compatible(&Type::Bool, &cond_type) {
                self.errors.add(
                    SemanticError::new(
                        SemanticErrorKind::InvalidConditionType,
                        if_stmt.condition.node_position(),
                        format!("Условие if должно иметь булевый тип"),
                    )
                    .with_types(Type::Bool, cond_type.clone())
                    .with_suggestion("Используйте выражение, возвращающее true/false".to_string()),
                );
            }
        }
        self.analyze_statement(&if_stmt.then_branch);
        if let Some(else_branch) = &if_stmt.else_branch {
            self.analyze_statement(else_branch);
        }
    }

    fn analyze_while(&mut self, while_stmt: &WhileStmt) {
        let cond_type = self.analyze_expression(&while_stmt.condition);
        if let Some(cond_type) = cond_type {
            if !self.type_checker.is_compatible(&Type::Bool, &cond_type) {
                self.errors.add(
                    SemanticError::new(
                        SemanticErrorKind::InvalidConditionType,
                        while_stmt.condition.node_position(),
                        format!("Условие while должно иметь булевый тип"),
                    )
                    .with_types(Type::Bool, cond_type.clone())
                    .with_suggestion("Используйте выражение, возвращающее true/false".to_string()),
                );
            }
        }
        self.loop_depth += 1;
        self.analyze_statement(&while_stmt.body);
        self.loop_depth -= 1;
    }

    fn analyze_switch(&mut self, switch_stmt: &SwitchStmt) {
        let _expr_type = self.analyze_expression(&switch_stmt.expression);
        for case in &switch_stmt.cases {
            self.analyze_statement(&case.body);
        }
        if let Some(default) = &switch_stmt.default {
            self.analyze_statement(default);
        }
    }

    fn analyze_for(&mut self, for_stmt: &ForStmt) {
        self.symbol_table.enter_scope();
        if let Some(init) = &for_stmt.init {
            self.analyze_statement(init);
        }
        if let Some(condition) = &for_stmt.condition {
            let cond_type = self.analyze_expression(condition);
            if let Some(cond_type) = cond_type {
                if !self.type_checker.is_compatible(&Type::Bool, &cond_type) {
                    self.errors.add(
                        SemanticError::new(
                            SemanticErrorKind::InvalidConditionType,
                            condition.node_position(),
                            format!("Условие for должно иметь булевый тип"),
                        )
                        .with_types(Type::Bool, cond_type.clone())
                        .with_suggestion(
                            "Используйте выражение, возвращающее true/false".to_string(),
                        ),
                    );
                }
            }
        }
        if let Some(update) = &for_stmt.update {
            self.analyze_expression(update);
        }
        self.loop_depth += 1;
        self.analyze_statement(&for_stmt.body);
        self.loop_depth -= 1;
        self.symbol_table.exit_scope();
    }

    fn analyze_return(&mut self, return_stmt: &ReturnStmt) {
        let current_func_info = self.current_function.as_ref().map(|f| {
            (
                f.name.clone(),
                f.return_type().cloned().unwrap_or(Type::Void),
            )
        });

        if let Some((func_name, expected_type)) = current_func_info {
            if let Some(value) = &return_stmt.value {
                let actual_type = self.analyze_expression(value);
                if let Some(actual_type) = actual_type {
                    if !self
                        .type_checker
                        .is_assignable(&expected_type, &actual_type)
                    {
                        self.errors.add(
                            SemanticError::new(
                                SemanticErrorKind::InvalidReturnType,
                                return_stmt.node.position(),
                                format!("Несоответствие возвращаемого типа"),
                            )
                            .with_types(expected_type.clone(), actual_type.clone())
                            .with_context(format!("функция '{}'", func_name))
                            .with_suggestion(format!(
                                "Функция ожидает {}, но возвращается {}",
                                expected_type, actual_type
                            )),
                        );
                    }
                }
                self.has_return = true;
            } else if !expected_type.is_void() {
                self.errors.add(
                    SemanticError::new(
                        SemanticErrorKind::InvalidReturnType,
                        return_stmt.node.position(),
                        format!("Функция должна возвращать значение"),
                    )
                    .with_types(expected_type.clone(), Type::Void)
                    .with_context(format!("функция '{}'", func_name))
                    .with_suggestion(format!(
                        "Добавьте возвращаемое значение типа {}",
                        expected_type
                    )),
                );
                self.has_return = true;
            } else {
                self.has_return = true;
            }
        } else {
            self.errors.add(
                SemanticError::new(
                    SemanticErrorKind::InvalidReturnType,
                    return_stmt.node.position(),
                    format!("return вне функции"),
                )
                .with_suggestion("Используйте return только внутри функции".to_string()),
            );
        }
    }

    fn analyze_expression(&mut self, expr: &Expression) -> Option<Type> {
        match expr {
            Expression::Literal(lit) => self.analyze_literal(lit),
            Expression::Identifier(ident) => self.analyze_identifier(ident),
            Expression::Binary(binary) => self.analyze_binary(binary),
            Expression::Unary(unary) => self.analyze_unary(unary),
            Expression::Assignment(assign) => self.analyze_assignment(assign),
            Expression::Call(call) => self.analyze_call(call),
            Expression::StructAccess(access) => self.analyze_struct_access(access),
            Expression::Grouped(grouped) => self.analyze_expression(&grouped.expr),
            Expression::ArrayAccess(access) => self.analyze_array_access(access),
            Expression::ArrayInitializer(arr) => {
                for elem in &arr.elements {
                    self.analyze_expression(elem);
                }
                arr.elements
                    .first()
                    .and_then(|e| self.analyze_expression(e))
            }
        }
    }

    fn analyze_literal(&self, lit: &Literal) -> Option<Type> {
        match lit.value {
            LiteralValue::Int(_) => Some(Type::Int),
            LiteralValue::Float(_) => Some(Type::Float),
            LiteralValue::Bool(_) => Some(Type::Bool),
            LiteralValue::String(_) => Some(Type::Pointer(Box::new(Type::Char))),
        }
    }

    fn analyze_identifier(&mut self, ident: &IdentifierExpr) -> Option<Type> {
        if let Some(symbol) = self.symbol_table.lookup(&ident.name) {
            Some(symbol.typ.clone())
        } else {
            eprintln!("DEBUG: Undeclared identifier '{}'", ident.name);
            self.errors.add(
                SemanticError::new(
                    SemanticErrorKind::UndeclaredIdentifier,
                    ident.node.position(),
                    format!("Переменная '{}' не объявлена", ident.name),
                )
                .with_suggestion(format!("Объявите '{}' перед использованием", ident.name)),
            );
            None
        }
    }

    fn analyze_binary(&mut self, binary: &BinaryExpr) -> Option<Type> {
        let left_type = self.analyze_expression(&binary.left);
        let right_type = self.analyze_expression(&binary.right);
        if let (Some(left), Some(right)) = (left_type, right_type) {
            let op_type: BinaryOpType = (&binary.operator).into();
            if !self.type_checker.are_compatible_binary(&left, &right) {
                self.errors.add(
                    SemanticError::new(
                        SemanticErrorKind::TypeMismatch,
                        binary.node.position(),
                        format!("Несовместимые типы в бинарной операции"),
                    )
                    .with_types(left.clone(), right.clone())
                    .with_suggestion(format!("Операнды должны иметь совместимые типы")),
                );
                return None;
            }
            self.type_checker.binary_result_type(&left, &right, op_type)
        } else {
            None
        }
    }

    fn analyze_unary(&mut self, unary: &UnaryExpr) -> Option<Type> {
        let operand_type = self.analyze_expression(&unary.operand);
        if let Some(operand) = operand_type {
            let op_type: UnaryOpType = (&unary.operator).into();
            self.type_checker.unary_result_type(&operand, op_type)
        } else {
            None
        }
    }

    fn analyze_assignment(&mut self, assign: &AssignmentExpr) -> Option<Type> {
        let target_type = self.analyze_expression(&assign.target);
        let value_type = self.analyze_expression(&assign.value);
        if let (Some(target), Some(value)) = (target_type, value_type) {
            if !self.type_checker.is_assignable(&target, &value) {
                self.errors.add(
                    SemanticError::new(
                        SemanticErrorKind::AssignmentTypeMismatch,
                        assign.node.position(),
                        format!("Несоответствие типов при присваивании"),
                    )
                    .with_types(target.clone(), value.clone())
                    .with_suggestion(format!("Значение должно быть типа {}", target)),
                );
            }
            Some(target)
        } else {
            None
        }
    }

    fn analyze_call(&mut self, call: &CallExpr) -> Option<Type> {
        let func_name = match &*call.callee {
            Expression::Identifier(ident) => ident.name.clone(),
            expr => {
                self.errors.add(
                    SemanticError::new(
                        SemanticErrorKind::InvalidExpression,
                        expr.node_position(),
                        format!("Вызов должен быть по имени функции"),
                    )
                    .with_suggestion("Используйте имя функции".to_string()),
                );
                return None;
            }
        };

        let func_info = self
            .symbol_table
            .lookup(&func_name)
            .and_then(|symbol| match &symbol.typ {
                Type::Function {
                    return_type,
                    param_types,
                } => Some((
                    (**return_type).clone(),
                    param_types.clone(),
                    symbol.is_variadic,
                )),
                _ => None,
            });

        if let Some((return_type, param_types, is_variadic)) = func_info {
            if call.arguments.len() < param_types.len() {
                let expected_count = param_types.len();
                let found_count = call.arguments.len();
                self.errors.add(
                    SemanticError::new(
                        SemanticErrorKind::ArgumentCountMismatch,
                        call.node.position(),
                        format!(
                            "Функция '{}' ожидает минимум {} аргументов, получено {}",
                            func_name, expected_count, found_count
                        ),
                    )
                    .with_suggestion(format!(
                        "Функция объявлена как {} ({}{}) -> {}",
                        func_name,
                        param_types
                            .iter()
                            .map(|t| t.to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                        if is_variadic { ", ..." } else { "" },
                        return_type
                    )),
                );
                return None;
            } else if !is_variadic && call.arguments.len() > param_types.len() {
                let expected_count = param_types.len();
                let found_count = call.arguments.len();
                self.errors.add(
                    SemanticError::new(
                        SemanticErrorKind::ArgumentCountMismatch,
                        call.node.position(),
                        format!(
                            "Функция '{}' ожидает {} аргументов, получено {}",
                            func_name, expected_count, found_count
                        ),
                    )
                    .with_suggestion(format!(
                        "Функция объявлена как {} ({}) -> {}",
                        func_name,
                        param_types
                            .iter()
                            .map(|t| t.to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                        return_type
                    )),
                );
                return None;
            }

            for (i, arg) in call.arguments.iter().enumerate() {
                let arg_type = self.analyze_expression(arg);
                if let Some(arg_type) = arg_type {
                    if i < param_types.len() {
                        let expected = &param_types[i];
                        if !self.type_checker.is_assignable(expected, &arg_type) {
                            self.errors.add(
                                SemanticError::new(
                                    SemanticErrorKind::ArgumentTypeMismatch,
                                    arg.node_position(),
                                    format!("Аргумент {} не соответствует типу параметра", i + 1),
                                )
                                .with_types(expected.clone(), arg_type.clone())
                                .with_suggestion(format!(
                                    "Ожидался тип {}, получен {}",
                                    expected, arg_type
                                )),
                            );
                        }
                    }
                }
            }

            Some(return_type)
        } else {
            if self.symbol_table.lookup(&func_name).is_some() {
                self.errors.add(
                    SemanticError::new(
                        SemanticErrorKind::UndeclaredIdentifier,
                        call.node.position(),
                        format!("'{}' не является функцией", func_name),
                    )
                    .with_suggestion("Убедитесь, что имя функции объявлено корректно".to_string()),
                );
            } else {
                self.errors.add(
                    SemanticError::new(
                        SemanticErrorKind::UndeclaredIdentifier,
                        call.node.position(),
                        format!("Функция '{}' не объявлена", func_name),
                    )
                    .with_suggestion(format!(
                        "Объявите функцию '{}' перед использованием",
                        func_name
                    )),
                );
            }
            None
        }
    }

    fn analyze_struct_access(&mut self, access: &StructAccessExpr) -> Option<Type> {
        let object_type = self.analyze_expression(&access.object);
        if let Some(typ) = object_type {
            match typ {
                Type::Struct(name) => {
                    if let Some(symbol) = self.symbol_table.lookup(&name) {
                        if let Some(fields) = &symbol.fields {
                            if let Some(field_type) = fields.get(&access.field) {
                                Some(field_type.clone())
                            } else {
                                self.errors.add(
                                    SemanticError::new(
                                        SemanticErrorKind::UndeclaredField,
                                        access.node.position(),
                                        format!(
                                            "Поле '{}' не найдено в структуре '{}'",
                                            access.field, name
                                        ),
                                    )
                                    .with_suggestion(format!(
                                        "Доступные поля: {:?}",
                                        fields.keys().collect::<Vec<_>>()
                                    )),
                                );
                                None
                            }
                        } else {
                            self.errors.add(
                                SemanticError::new(
                                    SemanticErrorKind::UndeclaredIdentifier,
                                    access.node.position(),
                                    format!("Структура '{}' не определена", name),
                                )
                                .with_suggestion(format!("Объявите структуру '{}'", name)),
                            );
                            None
                        }
                    } else {
                        self.errors.add(
                            SemanticError::new(
                                SemanticErrorKind::UndeclaredIdentifier,
                                access.node.position(),
                                format!("Структура '{}' не объявлена", name),
                            )
                            .with_suggestion(format!("Объявите структуру '{}'", name)),
                        );
                        None
                    }
                }
                _ => {
                    self.errors.add(
                        SemanticError::new(
                            SemanticErrorKind::TypeMismatch,
                            access.node.position(),
                            format!("Доступ к полю возможен только для структур"),
                        )
                        .with_types(Type::Struct("?".to_string()), typ)
                        .with_suggestion("Объект должен иметь тип структуры".to_string()),
                    );
                    None
                }
            }
        } else {
            None
        }
    }

    fn analyze_array_access(&mut self, access: &ArrayAccessExpr) -> Option<Type> {
        let array_type = self.analyze_expression(&access.array);
        let _index_type = self.analyze_expression(&access.index);
        if let Some(arr_typ) = array_type {
            match arr_typ {
                Type::Array(inner, _) => Some(*inner),
                Type::Pointer(inner) => Some(*inner),
                _ => {
                    self.errors.add(
                        SemanticError::new(
                            SemanticErrorKind::TypeMismatch,
                            access.node.position(),
                            format!("Индексация возможна только для массивов и указателей"),
                        )
                        .with_suggestion(
                            "Объект должен иметь тип массива или указателя".to_string(),
                        ),
                    );
                    None
                }
            }
        } else {
            None
        }
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
