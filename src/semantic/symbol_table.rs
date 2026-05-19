//! Таблица символов для отслеживания идентификаторов

use crate::common::position::Position;
use crate::semantic::type_system::Type;
use std::collections::HashMap;

/// Вид символа
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable,
    Parameter,
    Function,
    Struct,
    Field,
}

/// Информация о символе
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub typ: Type,
    pub kind: SymbolKind,
    pub position: Position,
    pub param_types: Option<Vec<Type>>,
    pub fields: Option<HashMap<String, Type>>,
    pub stack_offset: Option<i32>,
    pub is_variadic: bool,
}

impl Symbol {
    pub fn variable(name: String, typ: Type, position: Position) -> Self {
        Self {
            name,
            typ,
            kind: SymbolKind::Variable,
            position,
            param_types: None,
            fields: None,
            stack_offset: None,
            is_variadic: false,
        }
    }

    pub fn parameter(name: String, typ: Type, position: Position) -> Self {
        Self {
            name,
            typ,
            kind: SymbolKind::Parameter,
            position,
            param_types: None,
            fields: None,
            stack_offset: None,
            is_variadic: false,
        }
    }

    pub fn function(
        name: String,
        return_type: Type,
        param_types: Vec<Type>,
        is_variadic: bool,
        position: Position,
    ) -> Self {
        Self {
            name: name.clone(),
            typ: Type::Function {
                return_type: Box::new(return_type),
                param_types: param_types.clone(),
            },
            kind: SymbolKind::Function,
            position,
            param_types: Some(param_types),
            fields: None,
            stack_offset: None,
            is_variadic,
        }
    }

    pub fn struct_type(name: String, fields: HashMap<String, Type>, position: Position) -> Self {
        Self {
            name: name.clone(),
            typ: Type::Struct(name),
            kind: SymbolKind::Struct,
            position,
            param_types: None,
            fields: Some(fields),
            stack_offset: None,
            is_variadic: false,
        }
    }

    pub fn field(name: String, typ: Type, position: Position) -> Self {
        Self {
            name,
            typ,
            kind: SymbolKind::Field,
            position,
            param_types: None,
            fields: None,
            stack_offset: None,
            is_variadic: false,
        }
    }

    pub fn return_type(&self) -> Option<&Type> {
        if let Type::Function { return_type, .. } = &self.typ {
            Some(return_type)
        } else {
            None
        }
    }

    pub fn param_types(&self) -> Option<&[Type]> {
        self.param_types.as_deref()
    }

    pub fn is_function(&self) -> bool {
        matches!(self.kind, SymbolKind::Function)
    }
}

/// Таблица символов с поддержкой вложенных областей видимости
#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
    depth: usize,
    stack_offset: i32,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            depth: 0,
            stack_offset: 0,
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.depth += 1;
    }

    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.stack_offset = 0;
            self.scopes.pop();
            self.depth -= 1;
        }
    }

    pub fn insert(&mut self, name: &str, symbol: Symbol) -> bool {
        let current_scope = self.scopes.last_mut().unwrap();
        if current_scope.contains_key(name) {
            false
        } else {
            current_scope.insert(name.to_string(), symbol);
            true
        }
    }

    pub fn insert_with_offset(&mut self, name: &str, mut symbol: Symbol) -> bool {
        if matches!(symbol.kind, SymbolKind::Variable | SymbolKind::Parameter) {
            if let Some(size) = symbol.typ.size() {
                symbol.stack_offset = Some(self.stack_offset);
                self.stack_offset += size as i32;
            }
        }
        self.insert(name, symbol)
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }

    pub fn lookup_local(&self, name: &str) -> Option<&Symbol> {
        self.scopes.last().unwrap().get(name)
    }

    pub fn exists_local(&self, name: &str) -> bool {
        self.scopes.last().unwrap().contains_key(name)
    }

    pub fn exists(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn global_symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.scopes[0].values()
    }

    pub fn current_symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.scopes.last().unwrap().values()
    }

    pub fn current_offset(&self) -> i32 {
        self.stack_offset
    }

    pub fn frame_size(&self) -> i32 {
        self.stack_offset
    }

    pub fn dump(&self) -> String {
        let mut output = String::new();
        output.push_str("=== ТАБЛИЦА СИМВОЛОВ ===\n");
        for (i, scope) in self.scopes.iter().enumerate() {
            output.push_str(&format!("Область {} (глубина {}):\n", i, i));
            if scope.is_empty() {
                output.push_str("  (пусто)\n");
            } else {
                let mut symbols: Vec<(&String, &Symbol)> = scope.iter().collect();
                symbols.sort_by_key(|(name, _)| *name);

                for (name, symbol) in symbols {
                    let kind_ru = match symbol.kind {
                        SymbolKind::Variable => "переменная",
                        SymbolKind::Parameter => "параметр",
                        SymbolKind::Function => "функция",
                        SymbolKind::Struct => "структура",
                        SymbolKind::Field => "поле",
                    };
                    output.push_str(&format!("  {}: {} - {}\n", name, kind_ru, symbol.typ));
                }
            }
            output.push('\n');
        }
        output
    }

    pub fn update_symbol(&mut self, name: &str, new_symbol: &Symbol) -> bool {
        if let Some(current_scope) = self.scopes.last_mut() {
            if current_scope.contains_key(name) {
                current_scope.insert(name.to_string(), new_symbol.clone());
                return true;
            }
        }
        false
    }

    pub fn dump_with_layout(&self) -> String {
        let mut output = String::new();
        output.push_str("=== ТАБЛИЦА СИМВОЛОВ (РАЗМЕРЫ И СМЕЩЕНИЯ) ===\n");
        output.push_str(&format!(
            "Текущий размер фрейма: {} байт\n",
            self.frame_size()
        ));
        output.push_str(&format!("Глубина стека: {}\n\n", self.depth));

        for (i, scope) in self.scopes.iter().enumerate() {
            output.push_str(&format!("Область {} (глубина {}):\n", i, i));
            if scope.is_empty() {
                output.push_str("  (пусто)\n");
            } else {
                let mut symbols: Vec<(&String, &Symbol)> = scope.iter().collect();
                symbols.sort_by_key(|(name, _)| *name);

                for (name, symbol) in symbols {
                    let kind_ru = match symbol.kind {
                        SymbolKind::Variable => "переменная",
                        SymbolKind::Parameter => "параметр",
                        SymbolKind::Function => "функция",
                        SymbolKind::Struct => "структура",
                        SymbolKind::Field => "поле",
                    };
                    output.push_str(&format!("  {}: {} - {}", name, kind_ru, symbol.typ));

                    if let Some(offset) = symbol.stack_offset {
                        output.push_str(&format!(" [смещение: {}]", offset));
                    }

                    if let Some(size) = symbol.typ.size() {
                        output.push_str(&format!(" [размер: {}]", size));
                    }

                    if let Some(fields) = &symbol.fields {
                        output.push_str(" {\n");
                        let mut sorted_fields: Vec<(&String, &Type)> = fields.iter().collect();
                        sorted_fields.sort_by_key(|(name, _)| *name);

                        let mut current_offset = 0;
                        for (field_name, field_type) in sorted_fields {
                            output.push_str(&format!("    {}: {}", field_name, field_type));
                            output.push_str(&format!(" [смещение: {}]", current_offset));
                            if let Some(size) = field_type.size() {
                                output.push_str(&format!(" [размер: {}]", size));
                                current_offset += size;
                            }
                            output.push_str("\n");
                        }
                        output.push_str("  }");
                    }

                    output.push_str("\n");
                }
            }
            output.push_str("\n");
        }
        output
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::position::Position;

    #[test]
    fn test_symbol_table_operations() {
        let pos = Position::new(1, 1);
        let mut table = SymbolTable::new();

        let var = Symbol::variable("x".to_string(), Type::Int, pos.clone());
        assert!(table.insert("x", var));
        assert!(!table.insert(
            "x",
            Symbol::variable("x".to_string(), Type::Int, pos.clone())
        ));

        assert!(table.lookup("x").is_some());
        assert!(table.lookup_local("x").is_some());

        table.enter_scope();
        assert_eq!(table.depth(), 1);

        let y = Symbol::variable("y".to_string(), Type::Float, pos);
        assert!(table.insert("y", y));

        assert!(table.lookup_local("y").is_some());
        assert!(table.lookup("y").is_some());

        assert!(table.lookup("x").is_some());

        table.exit_scope();
        assert_eq!(table.depth(), 0);

        assert!(table.lookup("y").is_none());
        assert!(table.lookup("x").is_some());
    }
}
