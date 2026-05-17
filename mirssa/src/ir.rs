use enum_display::EnumDisplay;
use quasar::target::CallingConvention;
use quasar::*;

pub type ValueId = u32;
pub type InstId = u32;
pub type BlockId = u32;
pub type FuncId = u32;

#[derive(Debug, Clone)]
pub struct Value {
    pub ty: Type,
    pub def: InstId,
    pub uses: Vec<Use>,
}

#[derive(Debug, Clone, Copy)]
pub struct Use {
    pub inst: InstId,
    pub index: u32,
}

#[derive(Debug, Clone)]
pub struct Inst {
    pub kind: InstKind,
    pub operands: Vec<ValueId>,
    pub parent: BlockId,
    pub result: Option<ValueId>,
}

#[derive(Debug, EnumDisplay, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CmpKind {
    #[display("eq")]
    Eq,
    #[display("ne")]
    Ne,
    #[display("lt")]
    Lt,
    #[display("le")]
    Le,
    #[display("gt")]
    Gt,
    #[display("ge")]
    Ge,
    #[display("ult")]
    ULt,
    #[display("ule")]
    ULe,
    #[display("ugt")]
    UGt,
    #[display("uge")]
    UGe,
}

#[derive(Debug, EnumDisplay, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CastKind {
    #[display("zext")]
    Zext,
    #[display("sext")]
    Sext,
    #[display("trunc")]
    Trunc,
    #[display("bitcast")]
    Bitcast,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InstKind {
    IConst(i64),
    // float64 raw bits
    FConst(u64),

    Add,
    Sub,
    Mul,
    Div {
        signed: bool,
    },
    // remainder
    Rem {
        signed: bool,
    },

    FAdd,
    FSub,
    FMul,
    FDiv,
    FRem,

    And,
    Or,
    Xor,
    LShl,
    LShr,
    AShr,

    /// cmp {kind} {lhs}, {rhs}
    Cmp(CmpKind),
    /// alloca {type}
    Alloca(Type),
    /// alloca {size}
    NAlloca,
    /// load {ptr}
    Load {
        volatile: bool,
    },
    /// store {ptr}, {value}
    Store {
        volatile: bool,
    },
    /// elementptr {base}, {offset}
    ElementPtr,
    /// call {func} ({args..})
    Call(FuncId),
    /// cast {kind} {value}
    Cast(CastKind),
    /// j {block}
    Jump(BlockId),
    /// jc {cond} {then}(then_params..) {else}(else_params..)
    JumpIf {
        then_block: BlockId,
        else_block: BlockId,
    },
    /// ret {value}
    Ret,
}

impl InstKind {
    pub fn operand_count(&self) -> usize {
        use InstKind::*;

        match self {
            IConst(_) | FConst(_) => 0,

            // binary instructions
            Add
            | Sub
            | Mul
            | Div { .. }
            | Rem { .. }
            | FAdd
            | FSub
            | FMul
            | FDiv
            | FRem
            | And
            | Or
            | Xor
            | LShl
            | LShr
            | AShr => 2,
            Cmp(_) => 2,

            // type is not a operand
            Alloca(_) => 0,
            Load { .. } | NAlloca => 1,
            Store { .. } => 2,
            ElementPtr => 2,

            Cast(_) => 1,
            Ret => 1,

            // context depended
            Call(_) | Jump(_) | JumpIf { .. } => usize::MAX,
        }
    }

    pub fn get_cost(&self) -> u8 {
        match self {
            // constants (free)
            Self::IConst(_) | Self::FConst(_) => 0,

            // arithmetic (cheap ALU)
            Self::Add | Self::Sub => 2,
            Self::FAdd | Self::FSub => 4,
            // multiply/divide are expensive
            Self::Mul | Self::Div { .. } | Self::Rem { .. } => 4,
            Self::FMul | Self::FDiv | Self::FRem => 5,

            // bitwise ops (very cheap)
            Self::And | Self::Or | Self::Xor => 1,

            // shifts (cheap but not free)
            Self::LShl | Self::LShr | Self::AShr => 1,

            // comparisons (ALU + flag logic)
            Self::Cmp(_) => 2,

            // memory ops (expensive due to potential cache/memory)
            Self::Load { .. } => 5,
            Self::Store { .. } => 5,

            // address computation (usually cheap ALU-like)
            Self::NAlloca | Self::Alloca(_) => 3,
            Self::ElementPtr => 2,

            // function calls (very expensive, unknown cost)
            Self::Call(_) => 10,

            // type conversions (varies, but usually cheap-medium)
            Self::Cast(_) => 2,

            // control flow (terminators)
            Self::Jump(_) => 0,
            Self::JumpIf { .. } => 0,
            Self::Ret => 0,
        }
    }

    pub fn is_alloca(&self) -> bool {
        matches!(self, Self::NAlloca | Self::Alloca(_))
    }

    pub fn is_memory(&self) -> bool {
        matches!(self, Self::Load { .. } | Self::Store { .. })
    }

    pub fn is_call(&self) -> bool {
        matches!(self, Self::Call(_))
    }

    pub fn has_side_effects(&self) -> bool {
        self.is_call() || self.is_memory()
    }

    pub fn is_terminator(&self) -> bool {
        matches!(self, Self::Ret | Self::Jump(_) | Self::JumpIf { .. })
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub params: Vec<ValueId>,
    pub insts: Vec<InstId>,
    pub term: Option<InstId>,
    pub preds: Vec<BlockId>,
    pub succs: Vec<BlockId>,
}

#[derive(Debug, Clone, Default)]
pub struct FunctionDef {
    pub blocks: HashMap<BlockId, Block>,
    pub insts: HashMap<InstId, Inst>,
    pub values: HashMap<ValueId, Value>,

    next_block: BlockId,
    next_inst: InstId,
    next_value: ValueId,

    pub entry: BlockId,
    pub params: Vec<ValueId>,
}

impl FunctionDef {
    pub fn new() -> Self {
        let mut f = Self::default();
        let entry = f.new_block();
        f.entry = entry;
        f
    }

    pub fn reverse_post_order(&self) -> Vec<BlockId> {
        let mut visited = HashSet::new();
        let mut post_order = Vec::new();

        let mut stack: Vec<(BlockId, usize)> = vec![(self.entry, 0)];
        visited.insert(self.entry);

        while let Some((bid, succ_idx)) = stack.last_mut() {
            let bid = *bid;
            let succs = &self.blocks[&bid].succs;

            if *succ_idx < succs.len() {
                let next = succs[*succ_idx];
                *succ_idx += 1;

                if visited.insert(next) {
                    stack.push((next, 0));
                }
            } else {
                post_order.push(bid);
                stack.pop();
            }
        }

        post_order.reverse();
        post_order
    }

    pub fn new_block(&mut self) -> BlockId {
        let id = self.next_block;
        self.next_block += 1;

        self.blocks.insert(
            id,
            Block {
                params: Vec::new(),
                insts: Vec::new(),
                term: None,
                preds: Vec::new(),
                succs: Vec::new(),
            },
        );

        id
    }

    pub fn get_block_params(&self, block: BlockId) -> &Vec<ValueId> {
        let block = self.blocks.get(&block).unwrap();
        &block.params
    }

    pub fn add_param(&mut self, ty: Type) -> ValueId {
        let v = self.add_block_param(self.entry, ty);
        self.params.push(v);
        v
    }

    pub fn add_block_param(&mut self, block: BlockId, ty: Type) -> ValueId {
        let v = self.new_value(ty, InstId::MAX);
        self.blocks.get_mut(&block).unwrap().params.push(v);
        v
    }

    fn new_value(&mut self, ty: Type, def: InstId) -> ValueId {
        let id = self.next_value;
        self.next_value += 1;

        self.values.insert(
            id,
            Value {
                ty,
                def,
                uses: Vec::new(),
            },
        );

        id
    }

    pub fn add_use(&mut self, value: ValueId, inst: InstId, index: u32) {
        self.values
            .get_mut(&value)
            .unwrap()
            .uses
            .push(Use { inst, index });
    }

    pub fn append_inst(
        &mut self,
        block: BlockId,
        kind: InstKind,
        result_ty: Type,
        operands: Vec<ValueId>,
    ) -> (InstId, ValueId) {
        let (inst, val) = self.try_append_inst(block, kind, result_ty, operands);
        (inst, val.unwrap_or(ValueId::MAX))
    }

    pub fn try_append_inst(
        &mut self,
        block: BlockId,
        kind: InstKind,
        result_ty: Type,
        operands: Vec<ValueId>,
    ) -> (InstId, Option<ValueId>) {
        let inst_id = self.next_inst;
        self.next_inst += 1;

        let result = if !result_ty.is_void() {
            Some(self.new_value(result_ty, inst_id))
        } else {
            None
        };

        for (i, &op) in operands.iter().enumerate() {
            self.add_use(op, inst_id, i as u32);
        }

        if kind.is_terminator() {
            let mut edges_to_add = Vec::new();
            {
                let b = self.blocks.get_mut(&block).unwrap();
                match &kind {
                    InstKind::Jump(bb) => {
                        b.succs.push(*bb);
                        edges_to_add.push(*bb);
                    }
                    InstKind::JumpIf {
                        then_block,
                        else_block,
                    } => {
                        b.succs.push(*then_block);
                        b.succs.push(*else_block);
                        edges_to_add.push(*then_block);
                        edges_to_add.push(*else_block);
                    }
                    InstKind::Ret => {}
                    _ => unreachable!(),
                };
                b.term = Some(inst_id);
            }
            for target_bb in edges_to_add {
                self.blocks.get_mut(&target_bb).unwrap().preds.push(block);
            }
        }

        let inst = Inst {
            kind,
            operands,
            parent: block,
            result,
        };

        self.blocks.get_mut(&block).unwrap().insts.push(inst_id);
        self.insts.insert(inst_id, inst);

        (inst_id, result)
    }

    pub fn get_type(&self, v: ValueId) -> Type {
        self.values.get(&v).map(|v| v.ty).unwrap_or(Type::Void)
    }

    pub fn get_def_block(&self, v: ValueId) -> Option<BlockId> {
        let val = &self.values[&v];
        if val.def == InstId::MAX {
            None
        } else {
            Some(self.insts[&val.def].parent)
        }
    }

    pub fn get_def_inst(&self, v: ValueId) -> Option<InstId> {
        let val = self.values.get(&v)?;
        if val.def == InstId::MAX {
            None
        } else {
            Some(val.def)
        }
    }

    pub fn get_iconst(&self, v: ValueId) -> Option<i64> {
        let val = &self.values[&v];
        if val.def == InstId::MAX {
            return None;
        }

        match self.insts[&val.def].kind {
            InstKind::IConst(x) => Some(x),
            _ => None,
        }
    }

    pub fn get_fconst(&self, v: ValueId) -> Option<f64> {
        let val = &self.values[&v];
        if val.def == InstId::MAX {
            return None;
        }

        match self.insts[&val.def].kind {
            InstKind::FConst(x) => Some(f64::from_bits(x)),
            _ => None,
        }
    }

    pub fn lookup_block(&self, b: BlockId) -> Option<&Block> {
        self.blocks.get(&b)
    }

    pub(crate) fn get_jumpif_params<'a>(
        &'a self,
        inst: &'a Inst,
    ) -> Option<(ValueId, &'a [ValueId], &'a [ValueId])> {
        if let InstKind::JumpIf {
            then_block,
            else_block,
        } = inst.kind
        {
            let cond = inst.operands[0];

            let t = self.lookup_block(then_block).unwrap().params.len();
            let e = self.lookup_block(else_block).unwrap().params.len();

            let then_start = 1;
            let then_end = then_start + t;

            let else_start = then_end;
            let else_end = else_start + e;

            let then_params = &inst.operands[then_start..then_end];
            let else_params = &inst.operands[else_start..else_end];

            Some((cond, then_params, else_params))
        } else {
            None
        }
    }

    pub fn try_get_iconst(&self, v: ValueId) -> Option<i64> {
        let val = self.values.get(&v)?;
        if val.def == InstId::MAX {
            return None;
        }

        let inst = self.insts.get(&val.def)?;
        match inst.kind {
            InstKind::IConst(x) => Some(x),
            _ => None,
        }
    }

    pub fn is_value_valid(&self, v: ValueId) -> bool {
        self.values.contains_key(&v)
    }

    pub fn make_iconst(&mut self, block: BlockId, ty: Type, x: i64) -> ValueId {
        let (_, v) = self.append_inst(block, InstKind::IConst(x), ty, vec![]);
        v
    }

    fn make_unary(
        &mut self,
        block: BlockId,
        kind: InstKind,
        ty: Type,
        value: ValueId,
    ) -> (InstId, ValueId) {
        self.append_inst(block, kind, ty, vec![value])
    }

    fn make_binary(
        &mut self,
        block: BlockId,
        kind: InstKind,
        ty: Type,
        lhs: ValueId,
        rhs: ValueId,
    ) -> (InstId, ValueId) {
        self.append_inst(block, kind, ty, vec![lhs, rhs])
    }

    pub fn make_cmp(
        &mut self,
        block: BlockId,
        kind: CmpKind,
        lhs: ValueId,
        rhs: ValueId,
    ) -> (InstId, ValueId) {
        self.make_binary(block, InstKind::Cmp(kind), Type::I1, lhs, rhs)
    }

    pub fn make_cast(
        &mut self,
        block: BlockId,
        kind: CastKind,
        ty: Type,
        value: ValueId,
    ) -> (InstId, ValueId) {
        self.make_unary(block, InstKind::Cast(kind), ty, value)
    }

    pub fn make_add(
        &mut self,
        block: BlockId,
        ty: Type,
        lhs: ValueId,
        rhs: ValueId,
    ) -> (InstId, ValueId) {
        self.make_binary(block, InstKind::Add, ty, lhs, rhs)
    }

    pub fn make_sub(
        &mut self,
        block: BlockId,
        ty: Type,
        lhs: ValueId,
        rhs: ValueId,
    ) -> (InstId, ValueId) {
        self.make_binary(block, InstKind::Sub, ty, lhs, rhs)
    }

    pub fn make_mul(
        &mut self,
        block: BlockId,
        ty: Type,
        lhs: ValueId,
        rhs: ValueId,
    ) -> (InstId, ValueId) {
        self.make_binary(block, InstKind::Mul, ty, lhs, rhs)
    }

    pub fn make_div(
        &mut self,
        block: BlockId,
        signed: bool,
        ty: Type,
        lhs: ValueId,
        rhs: ValueId,
    ) -> (InstId, ValueId) {
        self.make_binary(block, InstKind::Div { signed }, ty, lhs, rhs)
    }

    pub fn make_lshl(
        &mut self,
        block: BlockId,
        ty: Type,
        lhs: ValueId,
        rhs: ValueId,
    ) -> (InstId, ValueId) {
        self.make_binary(block, InstKind::LShl, ty, lhs, rhs)
    }

    pub fn make_lshr(
        &mut self,
        block: BlockId,
        ty: Type,
        lhs: ValueId,
        rhs: ValueId,
    ) -> (InstId, ValueId) {
        self.make_binary(block, InstKind::LShr, ty, lhs, rhs)
    }

    pub fn make_ashr(
        &mut self,
        block: BlockId,
        ty: Type,
        lhs: ValueId,
        rhs: ValueId,
    ) -> (InstId, ValueId) {
        self.make_binary(block, InstKind::AShr, ty, lhs, rhs)
    }

    pub fn make_alloca(&mut self, block: BlockId, ty: Type) -> (InstId, ValueId) {
        self.append_inst(block, InstKind::Alloca(ty), Type::Ptr, vec![])
    }

    pub fn make_store(
        &mut self,
        block: BlockId,
        volatile: bool,
        ptr: ValueId,
        value: ValueId,
    ) -> (InstId, ValueId) {
        self.append_inst(
            block,
            InstKind::Store { volatile },
            Type::Void,
            vec![ptr, value],
        )
    }

    pub fn make_load(
        &mut self,
        block: BlockId,
        volatile: bool,
        ty: Type,
        ptr: ValueId,
    ) -> (InstId, ValueId) {
        self.append_inst(block, InstKind::Load { volatile }, ty, vec![ptr])
    }

    pub fn make_element_ptr(
        &mut self,
        block: BlockId,
        base: ValueId,
        offset: ValueId,
    ) -> (InstId, ValueId) {
        self.append_inst(block, InstKind::ElementPtr, Type::Ptr, vec![base, offset])
    }

    pub fn make_nalloca(
        &mut self,
        block: BlockId,
        ty: Type,
        operands: Vec<ValueId>,
    ) -> (InstId, ValueId) {
        self.append_inst(block, InstKind::NAlloca, ty, operands)
    }

    pub fn make_call(
        &mut self,
        block: BlockId,
        ty: Type,
        func: FuncId,
        operands: Vec<ValueId>,
    ) -> (InstId, ValueId) {
        self.append_inst(block, InstKind::Call(func), ty, operands)
    }

    pub fn make_ret(&mut self, block: BlockId, value: Option<ValueId>) -> (InstId, ValueId) {
        let mut operands = vec![];
        if let Some(v) = value {
            operands.push(v);
        }
        let (i, v) = self.append_inst(block, InstKind::Ret, Type::Void, operands);
        self.set_terminator(block, i);
        (i, v)
    }

    fn set_terminator(&mut self, block: BlockId, inst: InstId) {
        let blk = self.blocks.get_mut(&block).expect("block not exists");
        blk.term = Some(inst);
    }

    pub fn make_jump(
        &mut self,
        block: BlockId,
        target: BlockId,
        params: Vec<ValueId>,
    ) -> (InstId, ValueId) {
        let (i, v) = self.append_inst(block, InstKind::Jump(target), Type::Void, params);
        self.set_terminator(block, i);
        (i, v)
    }

    pub fn make_jumpif(
        &mut self,
        block: BlockId,
        cond: ValueId,
        then_block: BlockId,
        then_params: Vec<ValueId>,
        else_block: BlockId,
        else_params: Vec<ValueId>,
    ) -> (InstId, ValueId) {
        let mut operands = Vec::with_capacity(1 + then_params.len() + else_params.len());

        operands.push(cond);
        operands.extend(then_params);
        operands.extend(else_params);

        let (i, v) = self.append_inst(
            block,
            InstKind::JumpIf {
                then_block,
                else_block,
            },
            Type::Void,
            operands,
        );
        self.set_terminator(block, i);
        (i, v)
    }

    pub fn reconstruct_values(&mut self) {
        let mut id_map: HashMap<ValueId, ValueId> = HashMap::new();
        let mut next_id: ValueId = 0;

        let block_ids: Vec<BlockId> = self.blocks.keys().copied().collect();

        // pass 1: assign ids to all block params in order
        for &block_id in &block_ids {
            let old_params = self.blocks[&block_id].params.clone();
            for &old_param in &old_params {
                id_map.insert(old_param, next_id);
                next_id += 1;
            }
        }

        // pass 2: assign ids to constants and other insts by block
        for &block_id in &block_ids {
            let old_insts = self.blocks[&block_id].insts.clone();

            for &inst_id in &old_insts {
                #[allow(clippy::collapsible_if, clippy::map_entry)]
                if let Some(inst) = self.insts.get(&inst_id) {
                    if matches!(inst.kind, InstKind::IConst(_) | InstKind::FConst(_)) {
                        if let Some(old_result) = inst.result {
                            if !id_map.contains_key(&old_result) {
                                id_map.insert(old_result, next_id);
                                next_id += 1;
                            }
                        }
                    }
                }
            }

            for &inst_id in &old_insts {
                #[allow(clippy::collapsible_if, clippy::map_entry)]
                if let Some(inst) = self.insts.get(&inst_id) {
                    if !matches!(inst.kind, InstKind::IConst(_) | InstKind::FConst(_))
                        && !inst.kind.is_terminator()
                    {
                        if let Some(old_result) = inst.result {
                            if !id_map.contains_key(&old_result) {
                                id_map.insert(old_result, next_id);
                                next_id += 1;
                            }
                        }
                    }
                }
            }
        }

        for old_id in self.values.keys().copied().collect::<Vec<_>>() {
            #[allow(clippy::map_entry)]
            if !id_map.contains_key(&old_id) {
                id_map.insert(old_id, next_id);
                next_id += 1;
            }
        }

        // pass 3: rebuild with new ids
        let mut new_values: HashMap<ValueId, Value> = HashMap::new();
        let mut new_insts: HashMap<InstId, Inst> = HashMap::new();

        for (old_id, old_val) in &self.values {
            if let Some(&new_id) = id_map.get(old_id) {
                new_values.insert(new_id, old_val.clone());
            }
        }

        for (inst_id, inst) in &self.insts {
            let mut new_inst = inst.clone();

            new_inst.operands = new_inst
                .operands
                .iter()
                .map(|&op| id_map.get(&op).copied().unwrap_or(op))
                .collect();

            if let Some(old_result) = inst.result {
                new_inst.result = id_map.get(&old_result).copied();
            }

            new_insts.insert(*inst_id, new_inst);
        }

        // pass 4: reconstruct block.insts: constants first, then others
        for &block_id in &block_ids {
            let old_insts = self.blocks[&block_id].insts.clone();

            let mut const_insts = Vec::new();
            let mut other_insts = Vec::new();

            for &inst_id in &old_insts {
                if let Some(inst) = new_insts.get(&inst_id) {
                    if matches!(inst.kind, InstKind::IConst(_) | InstKind::FConst(_)) {
                        const_insts.push(inst_id);
                    } else if !inst.kind.is_terminator() {
                        other_insts.push(inst_id);
                    }
                }
            }

            // reconstruction: constants, instructions, terminator
            let mut new_block_insts = Vec::new();
            new_block_insts.extend(&const_insts);
            new_block_insts.extend(&other_insts);

            if let Some(term_id) = self.blocks[&block_id].term {
                new_block_insts.push(term_id);
            }

            self.blocks.get_mut(&block_id).unwrap().insts = new_block_insts;
        }

        // update block params with new ids
        for block in self.blocks.values_mut() {
            block.params = block
                .params
                .iter()
                .map(|&p| id_map.get(&p).copied().unwrap_or(p))
                .collect();
        }

        self.values = new_values;
        self.insts = new_insts;
        self.next_value = next_id;
    }

    pub fn recompute_uses(&mut self) {
        for v in self.values.values_mut() {
            v.uses.clear();
        }

        for (inst_id, inst) in &self.insts {
            for (idx, &op) in inst.operands.iter().enumerate() {
                let Some(val) = self.values.get_mut(&op) else {
                    continue;
                };

                val.uses.push(Use {
                    inst: *inst_id,
                    index: idx as u32,
                });
            }
        }
    }

    pub fn recompute_cfg(&mut self) {
        for b in self.blocks.values_mut() {
            b.succs.clear();
            b.preds.clear();
        }

        let block_ids: Vec<BlockId> = self.blocks.keys().copied().collect();

        for bid in block_ids {
            let term = match self.blocks[&bid].term {
                Some(t) => t,
                None => continue,
            };

            let inst = &self.insts[&term];

            match inst.kind {
                InstKind::Jump(target) => {
                    self.add_edge(bid, target);
                }

                InstKind::JumpIf {
                    then_block,
                    else_block,
                } => {
                    self.add_edge(bid, then_block);
                    self.add_edge(bid, else_block);
                }

                InstKind::Ret => {}
                _ => {}
            }
        }
    }

    pub fn reconstruct(&mut self) {
        self.reconstruct_values();
        self.recompute_cfg();
        self.recompute_uses();
    }

    fn add_edge(&mut self, from: BlockId, to: BlockId) {
        self.blocks.get_mut(&from).unwrap().succs.push(to);
        self.blocks.get_mut(&to).unwrap().preds.push(from);
    }

    pub fn remove_block(&mut self, block: BlockId) {
        let Some(b) = self.blocks.get(&block).cloned() else {
            return;
        };

        for inst_id in b.insts.iter().copied() {
            self.remove_inst(inst_id);
        }

        for param in b.params.iter() {
            if let Some(val) = self.values.remove(param) {
                debug_assert!(
                    val.uses.is_empty(),
                    "removing block with live uses of parameter value"
                );
            }
        }

        for pred in &b.preds {
            if let Some(pb) = self.blocks.get_mut(pred) {
                pb.succs.retain(|&s| s != block);
            }
        }

        for succ in &b.succs {
            if let Some(sb) = self.blocks.get_mut(succ) {
                sb.preds.retain(|&p| p != block);
            }
        }

        self.blocks.remove(&block);
    }

    pub fn remove_inst(&mut self, inst_id: InstId) {
        if let Some(inst) = self.insts.remove(&inst_id) {
            if let Some(res) = inst.result {
                self.values.remove(&res);
            }

            for op in inst.operands {
                if let Some(v) = self.values.get_mut(&op) {
                    v.uses.retain(|u| u.inst != inst_id);
                }
            }

            if let Some(block) = self.blocks.get_mut(&inst.parent) {
                block.insts.retain(|&i| i != inst_id);
            }
        }
    }

    pub fn replace_inst(&mut self, id: InstId, to: Inst) -> bool {
        let old = self.insts.get(&id).cloned();
        if old.is_none() {
            return false;
        }
        let old = old.unwrap();

        let old_cost = old.kind.get_cost();
        let to_cost = to.kind.get_cost();

        if to_cost > old_cost {
            log::warn!("replacement is more expensive");
            log::warn!("{:?}({}) -> {:?}({})", old.kind, old_cost, to.kind, to_cost);
        }

        self.insts.insert(id, to.clone());

        for (i, &op) in old.operands.iter().enumerate() {
            if let Some(v) = self.values.get_mut(&op) {
                v.uses.retain(|u| !(u.inst == id && u.index == i as u32));
            }
        }

        for (i, &op) in to.operands.iter().enumerate() {
            self.add_use(op, id, i as u32);
        }

        true
    }

    pub fn remove_inst_uses(&mut self, inst_id: InstId) {
        let inst = match self.insts.get(&inst_id) {
            Some(i) => i,
            None => return,
        };

        for (i, &op) in inst.operands.iter().enumerate() {
            if let Some(v) = self.values.get_mut(&op) {
                v.uses
                    .retain(|u| !(u.inst == inst_id && u.index == i as u32));
            }
        }
    }

    // pub fn replace_value(&mut self, from: ValueId, to: ValueId) {
    //     let uses = self.values[&from].uses.clone();
    //     for u in uses {
    //         let inst = self.insts.get_mut(&u.inst).unwrap();
    //         inst.operands[u.index as usize] = to;
    //         self.values.get_mut(&to).unwrap().uses.push(u);
    //     }
    //     self.values.get_mut(&from).unwrap().uses.clear();
    // }

    pub fn get_values(&self) -> &HashMap<ValueId, Value> {
        &self.values
    }

    pub fn get_insts(&self) -> &HashMap<InstId, Inst> {
        &self.insts
    }

    pub fn replace_value(&mut self, from: ValueId, to: ValueId) {
        let uses = self
            .values
            .get(&from)
            .map(|v| v.uses.clone())
            .unwrap_or_default();

        for u in uses {
            if let Some(inst) = self.insts.get_mut(&u.inst) {
                inst.operands[u.index as usize] = to;
            }

            if let Some(v) = self.values.get_mut(&to) {
                v.uses.push(u);
            }
        }

        if let Some(v) = self.values.get_mut(&from) {
            v.uses.clear();
        }
    }
}

impl FunctionDef {
    pub fn dump_dot(&self, module: &Module) -> String {
        let mut s = String::new();

        s.push_str("digraph SSA {\n");
        s.push_str("  node [shape=box, fontname=\"monospace\"];\n");

        for (bid, block) in &self.blocks {
            let mut label = format!("B{}:\\l", bid);

            if !block.params.is_empty() {
                label.push_str("  params: ");
                for p in &block.params {
                    label.push_str(&format!("%{}:{} ", p, self.values[p].ty));
                }
                label.push_str("\\l");
            }

            for inst_id in &block.insts {
                let inst = &self.insts[inst_id];

                if let Some(res) = inst.result {
                    label.push_str(&format!("  %{} = ", res));
                } else {
                    label.push_str("  ");
                }

                label.push_str(&self.fmt_inst(module, self, inst));
                label.push_str("\\l");
            }

            s.push_str(&format!("  B{} [label=\"{}\"];\n", bid, label));
        }

        for (bid, block) in &self.blocks {
            for succ in &block.succs {
                s.push_str(&format!("  B{} -> B{};\n", bid, succ));
            }
        }

        s.push_str("}\n");
        s
    }

    fn fmt_args(args: &[ValueId]) -> String {
        args.iter()
            .map(|v| format!("%{}", v))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn fmt_inst(&self, module: &Module, func: &FunctionDef, inst: &Inst) -> String {
        let result_ty = inst.result.map(|v| func.get_type(v)).unwrap_or(Type::Void);
        match &inst.kind {
            InstKind::IConst(x) => {
                let val = inst.result.unwrap();
                let ty = func.get_type(val);
                format!("const.{} ${}", ty, x)
            }
            InstKind::FConst(x) => {
                let val = inst.result.unwrap();
                let ty = func.get_type(val);
                format!("const.{} ${}", ty, x)
            }
            InstKind::Add => format!(
                "add.{} %{}, %{}",
                result_ty, inst.operands[0], inst.operands[1]
            ),
            InstKind::Sub => format!(
                "sub.{} %{}, %{}",
                result_ty, inst.operands[0], inst.operands[1]
            ),
            InstKind::Mul => format!(
                "mul.{} %{}, %{}",
                result_ty, inst.operands[0], inst.operands[1]
            ),
            InstKind::Div { signed } => format!(
                "{}div.{} %{}, %{}",
                if !*signed { "u" } else { "s" },
                result_ty,
                inst.operands[0],
                inst.operands[1]
            ),
            InstKind::Rem { signed } => format!(
                "{}rem.{} %{}, %{}",
                if !*signed { "u" } else { "s" },
                result_ty,
                inst.operands[0],
                inst.operands[1]
            ),
            InstKind::FAdd => format!(
                "fadd.{} %{}, %{}",
                result_ty, inst.operands[0], inst.operands[1]
            ),
            InstKind::FSub => format!(
                "fsub.{} %{}, %{}",
                result_ty, inst.operands[0], inst.operands[1]
            ),
            InstKind::FMul => format!(
                "fmul.{} %{}, %{}",
                result_ty, inst.operands[0], inst.operands[1]
            ),
            InstKind::FDiv => format!(
                "fdiv.{} %{}, %{}",
                result_ty, inst.operands[0], inst.operands[1]
            ),
            InstKind::FRem => format!(
                "frem.{} %{}, %{}",
                result_ty, inst.operands[0], inst.operands[1]
            ),
            InstKind::And => format!(
                "and.{} %{}, %{}",
                result_ty, inst.operands[0], inst.operands[1]
            ),
            InstKind::Or => format!(
                "or.{} %{}, %{}",
                result_ty, inst.operands[0], inst.operands[1]
            ),
            InstKind::Xor => format!(
                "xor.{} %{}, %{}",
                result_ty, inst.operands[0], inst.operands[1]
            ),
            InstKind::LShl => format!(
                "lshl.{} %{}, %{}",
                result_ty, inst.operands[0], inst.operands[1]
            ),
            InstKind::LShr => format!(
                "lshr.{} %{}, %{}",
                result_ty, inst.operands[0], inst.operands[1]
            ),
            InstKind::AShr => format!(
                "ashr.{} %{}, %{}",
                result_ty, inst.operands[0], inst.operands[1]
            ),

            InstKind::Cmp(k) => format!("cmp.{} %{}, %{}", k, inst.operands[0], inst.operands[1]),

            InstKind::Load { volatile } => format!(
                "{}load.{} %{}",
                if *volatile { "v" } else { "" },
                result_ty,
                inst.operands[0]
            ),
            InstKind::Store { volatile } => format!(
                "{}store %{}, %{}",
                if *volatile { "v" } else { "" },
                inst.operands[0],
                inst.operands[1]
            ),

            InstKind::NAlloca => format!("alloca {}", inst.operands[0]),
            InstKind::Alloca(ty) => format!("alloca.{}", ty),

            InstKind::ElementPtr => {
                format!("elemptr %{}, %{}", inst.operands[0], inst.operands[1])
            }

            InstKind::Call(fid) => {
                let name = &module.get_function(*fid).unwrap().name;
                format!("call {}({})", name, Self::fmt_args(&inst.operands))
            }

            InstKind::Cast(k) => format!("cast.{} %{}", k, inst.operands[0]),

            InstKind::Jump(bb) => {
                if inst.operands.is_empty() {
                    format!("jmp B{}", bb)
                } else {
                    let args = Self::fmt_args(&inst.operands);
                    format!("jmp B{}({})", bb, args)
                }
            }

            InstKind::JumpIf {
                then_block,
                else_block,
            } => {
                let (cond, then_params, else_params) = self.get_jumpif_params(inst).unwrap();

                let then_str = if then_params.is_empty() {
                    format!("B{}", then_block)
                } else {
                    format!("B{}({})", then_block, Self::fmt_args(then_params))
                };

                let else_str = if else_params.is_empty() {
                    format!("B{}", else_block)
                } else {
                    format!("B{}({})", else_block, Self::fmt_args(else_params))
                };

                format!("jmpif %{} {}, {}", cond, then_str, else_str)
            }

            InstKind::Ret => {
                if inst.operands.is_empty() {
                    "ret".to_string()
                } else {
                    format!("ret %{}", inst.operands[0])
                }
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct Function {
    pub name: String,
    pub signature: FunctionSignature,
    pub linkage: Linkage,
    pub calling_convention: CallingConvention,

    definition: Option<FunctionDef>,
}

impl Function {
    pub fn definition(
        name: &str,
        signature: FunctionSignature,
        linkage: Linkage,
        calling_convention: CallingConvention,
        def: FunctionDef,
    ) -> Self {
        let mut decl = Self::declaration(name, signature, linkage, calling_convention);
        decl.definition = Some(def);
        decl
    }

    pub fn declaration(
        name: &str,
        signature: FunctionSignature,
        linkage: Linkage,
        calling_convention: CallingConvention,
    ) -> Self {
        Self {
            name: name.to_string(),
            definition: None,
            signature,
            linkage,
            calling_convention,
        }
    }

    pub fn get_definition(&self) -> Option<&FunctionDef> {
        self.definition.as_ref()
    }

    pub fn get_definition_mut(&mut self) -> Option<&mut FunctionDef> {
        self.definition.as_mut()
    }
}

#[derive(Debug, Default)]
pub struct Module {
    pub name: String,
    pub functions: HashMap<FuncId, Function>,

    next_function: FuncId,
}

impl Module {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            functions: HashMap::new(),
            next_function: 0,
        }
    }

    pub fn add_function(&mut self, func: Function) -> FuncId {
        let id = self.next_function;
        self.next_function += 1;

        self.functions.insert(id, func);
        id
    }

    pub fn declare_function(
        &mut self,
        name: &str,
        signature: FunctionSignature,
        linkage: Linkage,
        calling_convention: CallingConvention,
    ) -> FuncId {
        let func = Function::declaration(name, signature, linkage, calling_convention);
        self.add_function(func)
    }

    pub fn define_function(&mut self, id: FuncId, def: FunctionDef) -> Result<(), String> {
        let func = self
            .functions
            .get_mut(&id)
            .ok_or_else(|| format!("function {} not found", id))?;

        if func.definition.is_some() {
            return Err(format!("function {} already defined", id));
        }

        func.definition = Some(def);
        Ok(())
    }

    pub fn get_function(&self, id: FuncId) -> Option<&Function> {
        self.functions.get(&id)
    }

    pub fn get_function_mut(&mut self, id: FuncId) -> Option<&mut Function> {
        self.functions.get_mut(&id)
    }

    pub fn lookup_function(&self, name: &str) -> Option<FuncId> {
        self.functions
            .iter()
            .find(|(_, f)| f.name == name)
            .map(|(id, _)| *id)
    }

    pub fn iter_functions(&self) -> impl Iterator<Item = (FuncId, &Function)> {
        self.functions.iter().map(|(id, f)| (*id, f))
    }

    pub fn iter_functions_mut(&mut self) -> impl Iterator<Item = (FuncId, &mut Function)> {
        self.functions.iter_mut().map(|(id, f)| (*id, f))
    }
}

impl Module {
    pub fn dump_dot(&self) -> String {
        let mut s = String::new();

        s.push_str("digraph ModuleSSA {\n");
        s.push_str("  node [shape=box, fontname=\"monospace\"];\n");
        s.push_str("  compound=true;\n\n");

        for (fid, func) in &self.functions {
            let def = match func.get_definition() {
                Some(d) => d,
                None => continue,
            };

            s.push_str(&format!("  subgraph cluster_f{} {{\n", fid));
            s.push_str(&format!(
                "    label=\"[{}] {}\";\n",
                func.calling_convention, func.name
            ));
            s.push_str("    style=rounded;\n\n");

            for (bid, block) in &def.blocks {
                let mut label = if block.params.is_empty() {
                    format!("B{}:\\l", bid)
                } else {
                    let params = block
                        .params
                        .iter()
                        .map(|p| format!("%{}:{}", p, def.values[p].ty))
                        .collect::<Vec<_>>()
                        .join(", ");

                    format!("B{}({}):\\l", bid, params)
                };

                for inst_id in &block.insts {
                    let inst = &def.insts[inst_id];

                    if let Some(res) = inst.result {
                        label.push_str(&format!("  %{} = ", res));
                    } else {
                        label.push_str("  ");
                    }

                    label.push_str(&def.fmt_inst(self, def, inst));
                    label.push_str("\\l");
                }

                s.push_str(&format!("    f{}_b{} [label=\"{}\"];\n", fid, bid, label));
            }

            s.push('\n');

            for (bid, block) in &def.blocks {
                for succ in &block.succs {
                    s.push_str(&format!("    f{}_b{} -> f{}_b{};\n", fid, bid, fid, succ));
                }
            }

            s.push_str("  }\n\n");
        }

        s.push_str("}\n");
        s
    }
}
