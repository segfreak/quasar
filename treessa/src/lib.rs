#![feature(box_patterns)]

use fear::types::Type;
use std::collections::{HashMap, HashSet};

pub mod dump;
pub mod passes;

pub type ValueId = u32;
pub type BlockId = u32;

#[derive(Debug, Clone, Hash)]
pub struct Value {
    pub ty: Type,
    pub expr: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Var(ValueId),
    Const(i64),

    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Ret(ValueId),
}

#[derive(Debug)]
pub struct BasicBlock {
    pub id: BlockId,
    pub params: Vec<ValueId>,
    pub values: Vec<ValueId>,
    pub terminator: Option<Terminator>,
}

#[derive(Debug, Default)]
pub struct FunctionDef {
    pub next_value: ValueId,
    pub next_block: BlockId,

    pub blocks: HashMap<BlockId, BasicBlock>,
    pub values: HashMap<ValueId, Value>,
    pub params: Vec<ValueId>,
}

impl FunctionDef {
    pub fn new() -> Self {
        Self {
            next_value: 0,
            next_block: 0,
            blocks: HashMap::new(),
            values: HashMap::new(),
            params: Vec::new(),
        }
    }

    pub fn new_block(&mut self) -> BlockId {
        let id = self.next_block;
        self.next_block += 1;

        self.blocks.insert(
            id,
            BasicBlock {
                id,
                params: vec![],
                values: vec![],
                terminator: None,
            },
        );

        id
    }

    pub fn get_block_params(&self, block: BlockId) -> &Vec<ValueId> {
        let block = self.blocks.get(&block).unwrap();
        &block.params
    }

    pub fn add_block_param(&mut self, block: BlockId, ty: Type) -> ValueId {
        let vid = self.next_value;
        self.next_value += 1;
        self.values.insert(
            vid,
            Value {
                ty,
                expr: Expr::Var(vid),
            },
        );
        self.blocks.get_mut(&block).unwrap().params.push(vid);
        vid
    }

    pub fn append_expr(&mut self, block: BlockId, ty: Type, expr: Expr) -> ValueId {
        let vid = self.next_value;
        self.next_value += 1;

        self.values.insert(vid, Value { ty, expr });

        self.blocks.get_mut(&block).unwrap().values.push(vid);

        vid
    }

    pub fn get_expr(&self, v: ValueId) -> &Expr {
        &self.values.get(&v).unwrap().expr
    }
}
