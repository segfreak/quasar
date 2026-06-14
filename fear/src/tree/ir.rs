use crate::{
    ssa::FuncId,
    types::{CastKind, FloatCmp, IntCmp, Type},
};
use std::{
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
};

pub type ValueId = u32;
pub type BlockId = u32;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expr {
    pub ty: Type,
    pub kind: ExprKind,
}

impl Expr {
    pub fn get_cost(&self) -> u32 {
        self.kind.get_cost()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExprKind {
    Var(ValueId),
    Const(i64),
    /// raw bits, use with f64::from_bits()
    FConst(u64),

    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(bool, Box<Expr>, Box<Expr>),
    Rem(bool, Box<Expr>, Box<Expr>),
    Square(Box<Expr>),

    FAdd(Box<Expr>, Box<Expr>),
    FSub(Box<Expr>, Box<Expr>),
    FMul(Box<Expr>, Box<Expr>),
    FDiv(Box<Expr>, Box<Expr>),
    FRem(Box<Expr>, Box<Expr>),
    FSquare(Box<Expr>),

    BitShl(Box<Expr>, Box<Expr>),
    BitShr(Box<Expr>, Box<Expr>),
    ArithShr(Box<Expr>, Box<Expr>),

    BitNeg(Box<Expr>),
    BitAnd(Box<Expr>, Box<Expr>),
    BitOr(Box<Expr>, Box<Expr>),
    BitXor(Box<Expr>, Box<Expr>),

    Cmp(IntCmp, Box<Expr>, Box<Expr>),
    FCmp(FloatCmp, Box<Expr>, Box<Expr>),
    Cast(CastKind, Box<Expr>),

    Alloca(Type),
    NAlloca(Type, usize),
    Load(/* volatile */ bool, /* ptr */ Box<Expr>),
    Store(
        /* volatile */ bool,
        /* ptr */ Box<Expr>,
        /* value */ Box<Expr>,
    ),
    PtrOffset(/* ptr */ Box<Expr>, /* offset */ Box<Expr>),
    ElementPtr(
        /* addressation unit */ Type,
        /* ptr */ Box<Expr>,
        /* offset */ Box<Expr>,
    ),

    Call(/* using fear modules */ crate::ssa::FuncId, Vec<Expr>),

    Undef,
    Select(
        /* cond */ Box<Expr>,
        /* then value */ Box<Expr>,
        /* else value */ Box<Expr>,
    ),
}

impl From<ValueId> for ExprKind {
    fn from(value: ValueId) -> Self {
        Self::Var(value)
    }
}

impl ExprKind {
    pub fn get_operands(&self) -> Vec<Expr> {
        match self {
            Self::Var(_) => vec![],
            Self::Const(_) | Self::FConst(_) => vec![],

            Self::Add(a, b)
            | Self::Sub(a, b)
            | Self::Mul(a, b)
            | Self::FAdd(a, b)
            | Self::FSub(a, b)
            | Self::FMul(a, b)
            | Self::FDiv(a, b)
            | Self::FRem(a, b)
            | Self::BitShl(a, b)
            | Self::BitShr(a, b)
            | Self::ArithShr(a, b)
            | Self::BitAnd(a, b)
            | Self::BitOr(a, b)
            | Self::BitXor(a, b)
            | Self::PtrOffset(a, b) => {
                vec![a.as_ref().clone(), b.as_ref().clone()]
            }

            Self::Div(_, a, b)
            | Self::Rem(_, a, b)
            | Self::Cmp(_, a, b)
            | Self::FCmp(_, a, b)
            | Self::ElementPtr(_, a, b) => {
                vec![a.as_ref().clone(), b.as_ref().clone()]
            }

            Self::Square(v)
            | Self::FSquare(v)
            | Self::BitNeg(v)
            | Self::Cast(_, v)
            | Self::Load(_, v) => {
                vec![v.as_ref().clone()]
            }

            Self::Store(_, ptr, value) => {
                vec![ptr.as_ref().clone(), value.as_ref().clone()]
            }

            Self::Call(_, args) => args.clone(),

            Self::Alloca(_) => vec![],
            Self::NAlloca(_, _) => vec![],
            Self::Undef => vec![],

            Self::Select(c, t, e) => {
                vec![c.as_ref().clone(), t.as_ref().clone(), e.as_ref().clone()]
            }
        }
    }

    pub fn get_operands_mut(&mut self) -> Vec<&mut Expr> {
        match self {
            Self::Var(_) => vec![],
            Self::Const(_) | Self::FConst(_) => vec![],

            Self::Add(a, b)
            | Self::Sub(a, b)
            | Self::Mul(a, b)
            | Self::FAdd(a, b)
            | Self::FSub(a, b)
            | Self::FMul(a, b)
            | Self::FDiv(a, b)
            | Self::FRem(a, b)
            | Self::BitShl(a, b)
            | Self::BitShr(a, b)
            | Self::ArithShr(a, b)
            | Self::BitAnd(a, b)
            | Self::BitOr(a, b)
            | Self::BitXor(a, b)
            | Self::PtrOffset(a, b) => {
                vec![a.as_mut(), b.as_mut()]
            }

            Self::Div(_, a, b)
            | Self::Rem(_, a, b)
            | Self::Cmp(_, a, b)
            | Self::FCmp(_, a, b)
            | Self::ElementPtr(_, a, b) => {
                vec![a.as_mut(), b.as_mut()]
            }

            Self::Square(v)
            | Self::FSquare(v)
            | Self::BitNeg(v)
            | Self::Cast(_, v)
            | Self::Load(_, v) => {
                vec![v.as_mut()]
            }

            Self::Store(_, ptr, value) => {
                vec![ptr.as_mut(), value.as_mut()]
            }

            Self::Call(_, args) => args.iter_mut().collect(),

            Self::Alloca(_) => vec![],
            Self::NAlloca(_, _) => vec![],
            Self::Undef => vec![],

            Self::Select(c, t, e) => {
                vec![c.as_mut(), t.as_mut(), e.as_mut()]
            }
        }
    }

    pub fn is_volatile(&self) -> bool {
        match self {
            Self::Load(volatile, _) | Self::Store(volatile, _, _) => *volatile,
            _ => false,
        }
    }

    pub fn has_side_effects(&self) -> bool {
        self.is_call() || self.is_store() || self.is_volatile()
    }

    pub fn is_call(&self) -> bool {
        matches!(self, Self::Call(_, _))
    }

    pub fn is_memory(&self) -> bool {
        self.is_load() || self.is_store()
    }

    pub fn is_load(&self) -> bool {
        matches!(self, Self::Load(_, _))
    }

    pub fn is_store(&self) -> bool {
        matches!(self, Self::Store(_, _, _))
    }

    pub fn can_eliminate(&self) -> bool {
        !self.has_side_effects()
    }

    fn cast_cost(kind: CastKind) -> u32 {
        use CastKind::*;
        match kind {
            // zero extension is free in x86
            Zext => 0,
            Sext => 1,
            // truncate is free in x86
            Trunc => 0,
            // bitcast is free in x86
            Bitcast => 0,
            SIToFP | UIToFP => 5,
            FPToSI | FPToUI => 5,
            FPromote => 3,
            FTrunc => 4,
        }
    }

    pub fn get_cost(&self) -> u32 {
        match self {
            Self::Var(_) | Self::Const(_) | Self::FConst(_) => 0,
            Self::Undef => 0,

            Self::Add(a, b) | Self::Sub(a, b) => 1 + a.get_cost() + b.get_cost(),
            Self::BitShl(a, b)
            | Self::BitShr(a, b)
            | Self::ArithShr(a, b)
            | Self::BitAnd(a, b)
            | Self::BitOr(a, b)
            | Self::BitXor(a, b) => 1 + a.get_cost() + b.get_cost(),

            Self::FAdd(a, b) | Self::FSub(a, b) | Self::FMul(a, b) => {
                4 + a.get_cost() + b.get_cost()
            }
            Self::FSquare(a) => 4 + a.get_cost(),

            Self::FDiv(a, b) => 13 + a.get_cost() + b.get_cost(),
            Self::FRem(a, b) => 15 + a.get_cost() + b.get_cost(),

            Self::BitNeg(a) => 1 + a.get_cost(),

            Self::Square(a) => 3 + a.get_cost(),
            Self::Mul(a, b) => 3 + a.get_cost() + b.get_cost(),
            Self::Div(_, a, b) => 25 + a.get_cost() + b.get_cost(),
            Self::Rem(_, a, b) => 25 + a.get_cost() + b.get_cost(),

            Self::Cmp(_, a, b) => 1 + a.get_cost() + b.get_cost(),
            Self::FCmp(_, a, b) => 4 + a.get_cost() + b.get_cost(),
            Self::Cast(kind, a) => Self::cast_cost(*kind) + a.get_cost(),

            Self::Alloca(_) | ExprKind::NAlloca(_, _) => 2,
            Self::PtrOffset(_, _) | Self::ElementPtr(_, _, _) => 2,
            Self::Load(_, _) | Self::Store(_, _, _) => 5,

            Self::Call(_, _) => 50,

            Self::Select(c, t, e) => 2 + c.get_cost() + t.get_cost() + e.get_cost(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Terminator {
    Ret(Expr),
    RetVoid,
    Br {
        bb: BlockId,
        params: Vec<Expr>,
    },
    BrIf {
        cond: Expr,
        then_bb: BlockId,
        then_params: Vec<Expr>,
        else_bb: BlockId,
        else_params: Vec<Expr>,
    },
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct BasicBlock {
    pub id: BlockId,
    pub params: Vec<ValueId>,
    pub values: Vec<ValueId>,
    pub terminator: Option<Terminator>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FunctionDef {
    pub next_value: ValueId,
    pub next_block: BlockId,

    pub blocks: HashMap<BlockId, BasicBlock>,
    pub values: HashMap<ValueId, Expr>,

    entry: BlockId,
}

impl FunctionDef {
    pub fn new() -> Self {
        let mut t = Self::default();
        t.entry = t.new_block();
        t
    }

    pub fn reverse_post_order(&self) -> Vec<BlockId> {
        fn dfs(
            f: &FunctionDef,
            b: BlockId,
            visited: &mut HashSet<BlockId>,
            out: &mut Vec<BlockId>,
        ) {
            if !visited.insert(b) {
                return;
            }

            let block = f.blocks.get(&b).unwrap();

            match &block.terminator {
                Some(Terminator::Br { bb, .. }) => {
                    dfs(f, *bb, visited, out);
                }

                Some(Terminator::BrIf {
                    then_bb, else_bb, ..
                }) => {
                    dfs(f, *then_bb, visited, out);
                    dfs(f, *else_bb, visited, out);
                }

                Some(Terminator::Ret(_) | Terminator::RetVoid) | None => {}
            }

            out.push(b);
        }

        let mut visited = HashSet::new();
        let mut post = Vec::new();

        dfs(self, self.entry, &mut visited, &mut post);

        post.reverse();
        post
    }

    pub fn dirty_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        self.next_value.hash(&mut hasher);
        self.next_block.hash(&mut hasher);

        let mut blocks: Vec<_> = self.blocks.iter().collect();
        blocks.sort_by_key(|(id, _)| *id);

        for (id, block) in blocks {
            id.hash(&mut hasher);
            block.hash(&mut hasher);
        }

        let mut values: Vec<_> = self.values.iter().collect();
        values.sort_by_key(|(id, _)| *id);

        for (id, value) in values {
            id.hash(&mut hasher);
            value.hash(&mut hasher);
        }

        self.entry.hash(&mut hasher);

        hasher.finish()
    }

    pub fn get_entry(&self) -> BlockId {
        self.entry
    }

    pub fn get_cost(&self) -> u32 {
        self.values.values().map(|val| val.kind.get_cost()).sum()
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

    pub fn get_entry_param_exprs(&self) -> Vec<&Expr> {
        self.get_block_param_exprs(self.get_entry())
    }

    pub fn get_block_param_exprs(&self, block: BlockId) -> Vec<&Expr> {
        let block = self.blocks.get(&block).unwrap();
        block
            .params
            .iter()
            .map(|vid| self.values.get(vid).unwrap())
            .collect()
    }

    pub fn get_block_params(&self, block: BlockId) -> &Vec<ValueId> {
        let block = self.blocks.get(&block).unwrap();
        &block.params
    }

    pub fn add_block_param(&mut self, block: BlockId, ty: Type) -> Expr {
        let vid = self.next_value;
        let expr = Expr {
            ty,
            kind: ExprKind::Var(vid),
        };
        self.next_value += 1;
        self.values.insert(vid, expr.clone());
        self.blocks.get_mut(&block).unwrap().params.push(vid);
        expr
    }

    pub fn append_expr(&mut self, block: BlockId, expr: Expr) -> Expr {
        let ty = expr.ty;

        let vid = self.next_value;
        self.next_value += 1;

        self.values.insert(vid, expr);
        self.blocks.get_mut(&block).unwrap().values.push(vid);

        Expr {
            ty,
            kind: ExprKind::Var(vid),
        }
    }

    pub fn make_undef(&mut self, block: BlockId, ty: Type) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Undef,
            },
        )
    }

    pub fn make_select(
        &mut self,
        block: BlockId,
        ty: Type,
        cond: &Expr,
        then_value: &Expr,
        else_value: &Expr,
    ) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Select(
                    Box::new(cond.clone()),
                    Box::new(then_value.clone()),
                    Box::new(else_value.clone()),
                ),
            },
        )
    }

    pub fn make_call(&mut self, block: BlockId, ty: Type, func: FuncId, params: Vec<Expr>) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Call(func, params),
            },
        )
    }

    pub fn make_iconst(&mut self, block: BlockId, ty: Type, value: i64) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Const(value),
            },
        )
    }

    pub fn make_fconst(&mut self, block: BlockId, ty: Type, bits: u64) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::FConst(bits),
            },
        )
    }

    pub fn make_cast(&mut self, block: BlockId, ty: Type, kind: CastKind, value: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Cast(kind, Box::new(value.clone())),
            },
        )
    }

    pub fn make_add(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Add(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_sub(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Sub(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_mul(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Mul(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_square(&mut self, block: BlockId, ty: Type, value: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Square(Box::new(value.clone())),
            },
        )
    }

    pub fn make_div(
        &mut self,
        block: BlockId,
        ty: Type,
        signed: bool,
        left: &Expr,
        right: &Expr,
    ) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Div(signed, Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_rem(
        &mut self,
        block: BlockId,
        ty: Type,
        signed: bool,
        left: &Expr,
        right: &Expr,
    ) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Rem(signed, Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_fadd(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::FAdd(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_fsub(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::FSub(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_fmul(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::FMul(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_fdiv(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::FDiv(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_frem(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::FRem(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_fsquare(&mut self, block: BlockId, ty: Type, value: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::FSquare(Box::new(value.clone())),
            },
        )
    }

    pub fn make_shl(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::BitShl(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_shr(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::BitShr(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_ashr(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::ArithShr(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_bitneg(&mut self, block: BlockId, ty: Type, value: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::BitNeg(Box::new(value.clone())),
            },
        )
    }

    pub fn make_bitand(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::BitAnd(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_bitor(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::BitOr(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_bitxor(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::BitXor(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_icmp(&mut self, block: BlockId, kind: IntCmp, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty: Type::Int1,
                kind: ExprKind::Cmp(kind, Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_fcmp(&mut self, block: BlockId, kind: FloatCmp, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty: Type::Int1,
                kind: ExprKind::FCmp(kind, Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_alloca(&mut self, block: BlockId, ty: Type) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty: Type::Pointer,
                kind: ExprKind::Alloca(ty),
            },
        )
    }

    pub fn make_nalloca(&mut self, block: BlockId, ty: Type, count: usize) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty: Type::Pointer,
                kind: ExprKind::NAlloca(ty, count),
            },
        )
    }

    pub fn make_load(&mut self, block: BlockId, ty: Type, volatile: bool, ptr: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Load(volatile, Box::new(ptr.clone())),
            },
        )
    }

    pub fn make_store(&mut self, block: BlockId, volatile: bool, ptr: &Expr, value: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty: Type::Void,
                kind: ExprKind::Store(volatile, Box::new(ptr.clone()), Box::new(value.clone())),
            },
        )
    }

    pub fn make_ptr_offset(&mut self, block: BlockId, base: &Expr, offset: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty: Type::Pointer,
                kind: ExprKind::PtrOffset(Box::new(base.clone()), Box::new(offset.clone())),
            },
        )
    }

    pub fn make_element_ptr(
        &mut self,
        block: BlockId,
        ty: Type,
        base: &Expr,
        offset: &Expr,
    ) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty: Type::Pointer,
                kind: ExprKind::ElementPtr(ty, Box::new(base.clone()), Box::new(offset.clone())),
            },
        )
    }
    pub fn make_ret(&mut self, block: BlockId, value: &Expr) {
        self.blocks.get_mut(&block).unwrap().terminator = Some(Terminator::Ret(value.clone()));
    }

    pub fn make_retvoid(&mut self, block: BlockId) {
        self.blocks.get_mut(&block).unwrap().terminator = Some(Terminator::RetVoid);
    }

    pub fn make_br(&mut self, block: BlockId, target: BlockId, params: Vec<Expr>) {
        self.blocks.get_mut(&block).unwrap().terminator =
            Some(Terminator::Br { bb: target, params });
    }

    pub fn make_brif(
        &mut self,
        block: BlockId,
        cond: &Expr,
        then_bb: BlockId,
        then_params: Vec<Expr>,
        else_bb: BlockId,
        else_params: Vec<Expr>,
    ) {
        self.blocks.get_mut(&block).unwrap().terminator = Some(Terminator::BrIf {
            cond: cond.clone(),
            then_bb,
            then_params,
            else_bb,
            else_params,
        });
    }

    pub fn get_expr(&self, v: ValueId) -> &Expr {
        self.values.get(&v).unwrap()
    }
}
