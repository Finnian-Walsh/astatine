use std::collections::HashMap;

use crate::syntax::{Declaration, FunctionDefinition, IdentifierId, Type, TypeId};

#[derive(Clone, Copy, Debug)]
pub enum SymbolDefinition<'a> {
    Variable(&'a Declaration),
    Function(&'a FunctionDefinition),
}

#[derive(Debug)]
pub struct Symbol<'a> {
    pub name: String,
    pub definition: SymbolDefinition<'a>,
}

#[derive(Debug)]
pub struct SymbolTable<'a> {
    pub id_map: HashMap<String, IdentifierId>,
    pub symbols: Vec<Symbol<'a>>,
    pub parent: Option<&'a SymbolTable<'a>>,
}

impl<'a> SymbolTable<'a> {
    pub fn new() -> Self {
        Self {
            id_map: HashMap::new(),
            symbols: vec![],
            parent: None,
        }
    }

    pub fn with_parent(parent: &'a SymbolTable) -> Self {
        Self {
            id_map: HashMap::new(),
            symbols: vec![],
            parent: Some(parent),
        }
    }

    pub fn push(&mut self, symbol: Symbol<'a>) {
        self.id_map
            .insert(symbol.name.clone(), IdentifierId(self.symbols.len()));
        self.symbols.push(symbol);
    }
}

#[derive(Debug)]
pub struct TypeTable {
    pub types: Vec<Type>,
    pub id_map: HashMap<String, TypeId>,
}

impl TypeTable {
    pub fn new() -> Self {
        Self {
            types: vec![],
            id_map: HashMap::new(),
        }
    }

    pub fn push(&mut self, ty: Type) {
        self.id_map
            .insert(ty.name().to_string(), TypeId(self.types.len()));
        self.types.push(ty);
    }
}
