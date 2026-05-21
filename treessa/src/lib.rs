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
    Div(Box<Expr>, Box<Expr>),

    BitShl(Box<Expr>, Box<Expr>),
    BitShr(Box<Expr>, Box<Expr>),
    ArithShr(Box<Expr>, Box<Expr>),
}

impl From<ValueId> for Expr {
    fn from(value: ValueId) -> Self {
        Self::Var(value)
    }
}

impl Expr {
    pub fn get_cost(&self) -> u32 {
        match self {
            Expr::Var(_) | Expr::Const(_) => 0,

            Expr::Add(a, b) | Expr::Sub(a, b) => 1 + a.get_cost() + b.get_cost(),
            Expr::BitShl(a, b) | Expr::BitShr(a, b) | Expr::ArithShr(a, b) => {
                1 + a.get_cost() + b.get_cost()
            }

            Expr::Mul(a, b) => 3 + a.get_cost() + b.get_cost(),
            Expr::Div(a, b) => 25 + a.get_cost() + b.get_cost(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Ret(ValueId),
    Br {
        bb: BlockId,
        params: Vec<ValueId>,
    },
    BrIf {
        cond: ValueId,
        then_bb: BlockId,
        then_params: Vec<ValueId>,
        else_bb: BlockId,
        else_params: Vec<ValueId>,
    },
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

    entry: BlockId,
}

impl FunctionDef {
    pub fn new() -> Self {
        let mut t = Self::default();
        t.entry = t.new_block();
        t
    }

    pub fn get_entry(&self) -> BlockId {
        self.entry
    }

    pub fn get_cost(&self) -> u32 {
        self.values.values().map(|val| val.expr.get_cost()).sum()
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

    pub fn make_iconst(&mut self, block: BlockId, ty: Type, value: i64) -> ValueId {
        self.append_expr(block, ty, Expr::Const(value))
    }

    pub fn make_add(&mut self, block: BlockId, ty: Type, left: Expr, right: Expr) -> ValueId {
        self.append_expr(block, ty, Expr::Add(Box::new(left), Box::new(right)))
    }

    pub fn make_sub(&mut self, block: BlockId, ty: Type, left: Expr, right: Expr) -> ValueId {
        self.append_expr(block, ty, Expr::Sub(Box::new(left), Box::new(right)))
    }

    pub fn make_mul(&mut self, block: BlockId, ty: Type, left: Expr, right: Expr) -> ValueId {
        self.append_expr(block, ty, Expr::Mul(Box::new(left), Box::new(right)))
    }

    pub fn make_div(&mut self, block: BlockId, ty: Type, left: Expr, right: Expr) -> ValueId {
        self.append_expr(block, ty, Expr::Div(Box::new(left), Box::new(right)))
    }

    pub fn make_ret(&mut self, block: BlockId, value: ValueId) {
        self.blocks.get_mut(&block).unwrap().terminator = Some(Terminator::Ret(value));
    }

    pub fn make_br(&mut self, block: BlockId, target: BlockId, params: Vec<ValueId>) {
        self.blocks.get_mut(&block).unwrap().terminator =
            Some(Terminator::Br { bb: target, params });
    }

    pub fn make_brif(
        &mut self,
        block: BlockId,
        cond: ValueId,
        then_bb: BlockId,
        then_params: Vec<ValueId>,
        else_bb: BlockId,
        else_params: Vec<ValueId>,
    ) {
        self.blocks.get_mut(&block).unwrap().terminator = Some(Terminator::BrIf {
            cond,
            then_bb,
            then_params,
            else_bb,
            else_params,
        });
    }

    pub fn get_expr(&self, v: ValueId) -> &Expr {
        &self.values.get(&v).unwrap().expr
    }
}
