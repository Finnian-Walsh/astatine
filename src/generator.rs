use std::collections::HashMap;

use crate::{
    parser::Node,
    syntax::{Argument, Declaration, Expression, FunctionDefinition, IdentifierId, Type, TypeId},
};

// #[derive(Error)]
// enum Error {}

#[derive(Debug)]
enum SymbolDefinition<'a> {
    Variable(&'a Declaration),
    Function(&'a FunctionDefinition),
}

#[derive(Debug)]
struct Symbol<'a> {
    name: String,
    definition: Option<SymbolDefinition<'a>>,
}

#[derive(Debug)]
struct SymbolTable<'a> {
    id_map: HashMap<String, IdentifierId>,
    symbols: Vec<Symbol<'a>>,
    parent: Option<&'a SymbolTable<'a>>,
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
struct TypeTable {
    types: Vec<Type>,
    id_map: HashMap<String, TypeId>,
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

#[derive(Debug)]
pub struct Generator<'a> {
    ast: &'a [Node],

    types: TypeTable, // index via type id
    global_symbols: SymbolTable<'a>,
}

impl<'a> Generator<'a> {
    pub fn new(ast: &'a [Node]) -> Self {
        let mut types = TypeTable::new();
        let mut global_symbols = SymbolTable::new();

        // TODO: primitive types

        for node in ast {
            match node {
                Node::Function(function) => global_symbols.push(Symbol {
                    name: function.name.clone(),
                    definition: Some(SymbolDefinition::Function(function)),
                }),
                Node::ConstDecl(decl) => {
                    global_symbols.push(Symbol {
                        name: decl.name.clone(),
                        definition: Some(SymbolDefinition::Variable(decl)),
                    });
                }
                Node::StructDef(struct_def) => types.push(Type::Struct(struct_def.clone())),
            }
        }

        Self {
            ast,
            types,
            global_symbols,
        }
    }

    pub fn generate_asm() {
        // let labels = HashMap::new();
        //
        // let mut asm = "
        //     global _start\n\
        //     _start:\n\
        // "
        // .to_string();
        //
        // for (name, instructions) in labels {}
    }

    fn const_eval(expr: Expression, value_type: TypeId) {}
}
