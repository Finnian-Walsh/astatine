use std::collections::HashMap;

use crate::{
    parser::Node,
    syntax::{
        Declaration, Expression, FunctionDefinition, IdentifierId, LiteralKind, Type, TypeId,
    },
};

// #[derive(Error)]
// enum Error {}

#[derive(Clone, Copy, Debug)]
enum SymbolDefinition<'a> {
    Variable(&'a Declaration),
    Function(&'a FunctionDefinition),
}

#[derive(Debug)]
struct Symbol<'a> {
    name: String,
    definition: SymbolDefinition<'a>,
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

pub enum ConstantResolveError<'a> {
    NoDefinition(&'a str),
    InvalidSymbol(&'a str, SymbolDefinition<'a>),
}

#[derive(Default)]
struct RoDataSection {
    entries: Vec<String>,
}

#[derive(Default)]
struct TextSection {
    entries: Vec<String>,
}

#[derive(Default)]
pub struct Sections {
    ro_data: RoDataSection,
    text: TextSection,
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
                    definition: SymbolDefinition::Function(function),
                }),
                Node::ConstDecl(decl) => {
                    global_symbols.push(Symbol {
                        name: decl.name.clone(),
                        definition: SymbolDefinition::Variable(decl),
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

    pub fn resolve_constant(&self, declaration: &'a Declaration) -> Vec<String> {
        // TODO: name mangling (use IDs)
        match &declaration.value {
            Expression::Literal { kind, value } => match kind {
                LiteralKind::Char => todo!(),
                LiteralKind::Integer => todo!(),
                LiteralKind::Float => todo!(),
                LiteralKind::String => {
                    let mut lines = vec![];
                    lines.push(format!("{} db \"{value}\"", declaration.name));
                    lines.push(format!(
                        "{}_len equ $ - {}",
                        declaration.name, declaration.name,
                    ));
                    lines
                }
            },
            _ => todo!("Implement constant abstract types"),
        }
    }

    pub fn generate_function(&self, definition: &FunctionDefinition) -> Vec<String> {
        let mut lines = vec![];

        lines.push(String::new());

        if !definition.params.is_empty() {
            lines.push(format!("; {:?}", definition.params));
        }

        lines.push(format!("{}:", definition.name));

        lines.push("ret".to_string());
        lines
    }

    pub fn generate_asm(&self) -> String {
        // TODO: create intermediary section structures to be assembled together

        let mut sections = Sections::default();

        for symbol in &self.global_symbols.symbols {
            match symbol.definition {
                SymbolDefinition::Variable(declaration) => sections
                    .ro_data
                    .entries
                    .extend(self.resolve_constant(declaration)),
                SymbolDefinition::Function(function_definition) => {
                    sections
                        .text
                        .entries
                        .extend(self.generate_function(function_definition));
                }
            }
        }

        let mut asm = String::new();

        asm.push_str("section .rodata\n\n");
        for entry in sections.ro_data.entries {
            asm.push_str(&entry);
            asm.push('\n');
        }

        asm.push_str(
            "section .text\n\n\
            global _start\n\
            _start:\n\n\
            call main\n\n\
            mov rdi, rax\n\
            mov rax, 60\n\
            syscall\n\
            ",
        );
        for entry in sections.text.entries {
            asm.push_str(&entry);
            asm.push('\n')
        }

        asm
    }

    fn const_eval(_expr: Expression, _value_type: TypeId) {
        // TODO: implement constant evaluations

        todo!()
    }
}
