use std::{
    collections::{hash_map, HashMap, HashSet},
    hash::{Hash, Hasher},
};

use crate::types::{
    CallingConvention, CastKind, FloatCmp, FunctionSignature, IntCmp, Linkage, Type,
};

pub type ValueId = u32;
pub type InstId = u32;
pub type BlockId = u32;
pub type FuncId = u32;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Value {
    ty: Type,
    def: InstId,
    uses: Vec<Use>,
}

impl Value {
    pub fn new(ty: Type, def: InstId, uses: &[Use]) -> Self {
        Self {
            ty,
            def,
            uses: uses.into(),
        }
    }

    /// Get type of value
    pub fn get_type(&self) -> Type {
        self.ty
    }

    /// Get instruction id who defines this value
    pub fn get_def(&self) -> InstId {
        self.def
    }

    /// Get value uses
    pub fn get_uses(&self) -> &[Use] {
        &self.uses
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Use {
    inst: InstId,
    index: u32,
}

impl Use {
    /// Get user instruction
    pub fn get_inst(&self) -> InstId {
        self.inst
    }

    /// Get user operand index
    pub fn get_index(&self) -> u32 {
        self.index
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Inst {
    pub kind: InstKind,
    pub operands: Vec<ValueId>,
    pub parent: BlockId,
    pub result: Option<ValueId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    Neg,

    FAdd,
    FSub,
    FMul,
    FDiv,
    FRem,
    FNeg,

    Not,
    And,
    Or,
    Xor,
    LShl,
    LShr,
    AShr,
    /// cmp {kind} {lhs}, {rhs}
    Cmp(IntCmp),
    /// fcmp {kind} {lhs}, {rhs}
    FCmp(FloatCmp),
    /// alloca {type}
    Alloca(Type),
    /// nalloca {type} {len}
    NAlloca(Type, usize),
    /// load {ptr}
    Load {
        volatile: bool,
    },
    /// store {ptr}, {value}
    Store {
        volatile: bool,
    },

    /// ptroffset {base}, {offset}
    /// byte addressed
    ///   (uint8_t*)base + offset
    PtrOffset,

    /// elementptr {ty} {base}, {offset}
    /// element addressed
    ///   (ty*)base + offset
    ///
    /// base = nalloca i64 2
    /// first   = elementptr i32 base, 0     ! base + 0x00
    /// second  = elementptr i32 base, 1     ! base + 0x04
    ElementPtr(/* addressation unit */ Type),

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

    Undef,

    /// select {cond} {then_value}, {else_value}
    Select,
}

impl InstKind {
    pub fn operand_count(&self) -> usize {
        use InstKind::*;

        match self {
            IConst(_) | FConst(_) => 0,
            Undef => 0,

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
            Cmp(_) | FCmp(_) => 2,

            Alloca(_) | NAlloca(_, _) => 0,
            Load { .. } => 1,
            Store { .. } => 2,
            PtrOffset | ElementPtr(_) => 2,

            Cast(_) => 1,
            Ret => 1,

            Not | Neg | FNeg => 1,

            Select => 3,

            // context depended
            Call(_) | Jump(_) | JumpIf { .. } => usize::MAX,
        }
    }

    pub fn get_cost(&self) -> u8 {
        match self {
            Self::Undef => 0,     // Purely a compiler placeholder, emits zero instructions.
            Self::IConst(_) => 0, // Almost always encoded as an immediate or zeroed via XOR.
            Self::FConst(_) => 1, // May force a constant pool load on RISC architectures.
            Self::Jump(_) => 1,   // Unconditional branch; hardware front-end swallows this easily.

            Self::Add | Self::Sub | Self::Neg => 2,
            Self::Not | Self::And | Self::Or | Self::Xor => 2,
            Self::LShl | Self::LShr | Self::AShr => 2,
            Self::Cmp(_) => 2,

            Self::Alloca(_) | Self::NAlloca(_, _) => 2, // Just bumping the stack pointer (sub sp, imm).
            Self::JumpIf { .. } => 4, // Conditional branch; penalized for potential mispredictions.
            Self::Ret => 4, // Return; hits the Return Stack Buffer, but still costs a bit.

            // Address arithmetic. x86 has LEA, but ARM/RISC-V might need an extra shift + add chain.
            Self::PtrOffset | Self::ElementPtr(_) => 6,

            Self::Select => 6, // Low-cost on x86 (cmov) and ARM (csel), but expands to branches on basic RISC-V.
            Self::Cast(_) => 6, // Sign/zero extensions or float-to-int conversions.

            Self::FAdd | Self::FSub => 8,
            Self::FNeg => 1,
            Self::FCmp(_) => 8,

            Self::Mul => 12, // Integer multiply is fully pipelined nowadays, but takes 3-5 cycles.
            Self::FMul => 16, // Floating-point multiplication.

            // Memory ops. Assuming L1 cache hit as best case, but we heavily penalize them
            // to force the optimizer to keep variables in registers.
            Self::Store { .. } => 15, // Hidden by store buffers, but still occupies execution slots.
            Self::Load { .. } => 25, // Loads are blocking; later instructions usually have to wait for data.

            Self::FDiv | Self::FRem => 60, // Floating-point division is never cheap.

            // Integer division is a massive pain across all architectures.
            // Non-pipelined, iterative microcode. On some RISC-V/ARM cores, this can take up to 60+ cycles.
            Self::Div { .. } | Self::Rem { .. } => 120,

            Self::Call(_) => 200,
        }
    }

    pub fn is_alloca(&self) -> bool {
        matches!(self, Self::Alloca(_) | Self::NAlloca(_, _))
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Block {
    pub params: Vec<ValueId>,
    pub insts: Vec<InstId>,
    pub term: Option<InstId>,
    pub preds: Vec<BlockId>,
    pub succs: Vec<BlockId>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FunctionDef {
    blocks: HashMap<BlockId, Block>,
    insts: HashMap<InstId, Inst>,
    values: HashMap<ValueId, Value>,
    entry: BlockId,

    next_block: BlockId,
    next_inst: InstId,
    next_value: ValueId,
}

impl Hash for FunctionDef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.next_block.hash(state);
        self.next_inst.hash(state);
        self.next_value.hash(state);

        let mut blocks: Vec<_> = self.blocks.iter().collect();
        blocks.sort_by_key(|(id, _)| *id);

        for (id, block) in blocks {
            id.hash(state);
            block.hash(state);
        }

        let mut values: Vec<_> = self.values.iter().collect();
        values.sort_by_key(|(id, _)| *id);

        for (id, value) in values {
            id.hash(state);
            value.hash(state);
        }

        self.entry.hash(state);
    }
}

impl FunctionDef {
    pub fn new() -> Self {
        let mut f = Self::default();
        let entry = f.create_block();
        f.entry = entry;
        f
    }

    pub fn get_hash(&self) -> u64 {
        let mut state = crate::DefaultHasher::default();
        self.hash(&mut state);
        state.finish()
    }

    pub fn get_entry(&self) -> BlockId {
        self.entry
    }

    pub fn get_entry_mut(&mut self) -> &mut BlockId {
        &mut self.entry
    }

    pub fn get_blocks(&self) -> &HashMap<BlockId, Block> {
        &self.blocks
    }

    pub fn get_blocks_mut(&mut self) -> &mut HashMap<BlockId, Block> {
        &mut self.blocks
    }

    pub fn get_block(&self, block: BlockId) -> Option<&Block> {
        self.blocks.get(&block)
    }

    pub fn get_block_mut(&mut self, block: BlockId) -> Option<&mut Block> {
        self.blocks.get_mut(&block)
    }

    pub fn get_entry_block(&self) -> &Block {
        self.get_block(self.get_entry()).unwrap()
    }

    pub fn get_block_params(&self, block: BlockId) -> &[ValueId] {
        &self.get_block(block).unwrap().params
    }

    pub fn get_params(&self) -> &[ValueId] {
        self.get_block_params(self.get_entry())
    }

    pub fn add_block_param(&mut self, block: BlockId, ty: Type) -> ValueId {
        let v = self.create_value_without_def(ty);
        self.blocks.get_mut(&block).unwrap().params.push(v);
        v
    }

    pub fn add_param(&mut self, ty: Type) -> ValueId {
        self.add_block_param(self.entry, ty)
    }

    pub fn get_block_ids(&self) -> Vec<BlockId> {
        self.get_blocks().keys().cloned().collect()
    }

    pub fn get_inst_ids(&self) -> Vec<InstId> {
        self.get_insts().keys().cloned().collect()
    }

    pub fn compute_rpo(&self) -> Vec<BlockId> {
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

    pub fn create_block(&mut self) -> BlockId {
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

    fn create_value(&mut self, ty: Type, def: InstId) -> ValueId {
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

    fn create_value_without_def(&mut self, ty: Type) -> ValueId {
        self.create_value(ty, InstId::MAX)
    }

    pub fn add_use(&mut self, value: ValueId, inst: InstId, index: u32) {
        self.values
            .get_mut(&value)
            .unwrap()
            .uses
            .push(Use { inst, index });
    }

    /// used in builder.rs
    pub(super) fn append_inst_base(
        &mut self,
        block: BlockId,
        kind: InstKind,
        result_ty: Type,
        operands: Vec<ValueId>,
    ) -> (InstId, Option<ValueId>) {
        let inst_id = self.next_inst;
        self.next_inst += 1;

        let result = if !result_ty.is_void() {
            Some(self.create_value(result_ty, inst_id))
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

    /// used in builder.rs
    pub(super) fn append_inst(
        &mut self,
        block: BlockId,
        kind: InstKind,
        result_ty: Type,
        operands: Vec<ValueId>,
    ) -> ValueId {
        let (_, val) = self.append_inst_base(block, kind, result_ty, operands);
        val.unwrap_or(ValueId::MAX)
    }

    pub fn get_type_of(&self, v: ValueId) -> Type {
        self.values.get(&v).map(|v| v.ty).unwrap_or(Type::Void)
    }

    pub fn get_value_def_in(&self, v: ValueId) -> Option<BlockId> {
        let val = &self.values[&v];
        if val.def == InstId::MAX {
            None
        } else {
            Some(self.insts[&val.def].parent)
        }
    }

    pub fn get_value_def(&self, v: ValueId) -> Option<InstId> {
        let val = self.values.get(&v)?;
        if val.def == InstId::MAX {
            None
        } else {
            Some(val.def)
        }
    }

    pub fn get_int_const(&self, v: ValueId) -> Option<i64> {
        let val = &self.values[&v];
        if val.def == InstId::MAX {
            return None;
        }

        match self.insts[&val.def].kind {
            InstKind::IConst(x) => Some(x),
            _ => None,
        }
    }

    pub fn get_float_const_bits(&self, v: ValueId) -> Option<u64> {
        let val = &self.values[&v];
        if val.def == InstId::MAX {
            return None;
        }

        match self.insts[&val.def].kind {
            InstKind::FConst(x) => Some(x),
            _ => None,
        }
    }

    pub fn get_float_const(&self, v: ValueId) -> Option<f64> {
        self.get_float_const_bits(v).map(f64::from_bits)
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

            let t = self.get_block_params(then_block).len();
            let e = self.get_block_params(else_block).len();

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

    pub fn has_value(&self, v: ValueId) -> bool {
        self.values.contains_key(&v)
    }

    /// used in builder.rs
    pub(super) fn set_terminator(&mut self, block: BlockId, inst: InstId) {
        let blk = self.blocks.get_mut(&block).expect("block not exists");
        blk.term = Some(inst);
    }

    fn reassign_values(&mut self) {
        let mut id_map: HashMap<ValueId, ValueId> = HashMap::new();
        let mut next_id: ValueId = 0;

        let block_ids = self.compute_rpo();

        // value ids order:
        //   - entry block params
        //   - block params
        //   - constants
        //   - normal instructions
        let entry = self.entry;

        for &param in &self.blocks[&entry].params {
            id_map.insert(param, next_id);
            next_id += 1;
        }

        for &block_id in &block_ids {
            let block = &self.blocks[&block_id];

            if block_id != entry {
                for &param in &block.params {
                    id_map.insert(param, next_id);
                    next_id += 1;
                }
            }

            for &inst_id in &block.insts {
                let Some(inst) = self.insts.get(&inst_id) else {
                    continue;
                };
                #[allow(clippy::collapsible_if)]
                if matches!(inst.kind, InstKind::IConst(_) | InstKind::FConst(_)) {
                    if let Some(result) = inst.result {
                        if let hash_map::Entry::Vacant(e) = id_map.entry(result) {
                            e.insert(next_id);
                            next_id += 1;
                        }
                    }
                }
            }

            for &inst_id in &block.insts {
                let Some(inst) = self.insts.get(&inst_id) else {
                    continue;
                };

                #[allow(clippy::collapsible_if)]
                if !matches!(inst.kind, InstKind::IConst(_) | InstKind::FConst(_))
                    && !inst.kind.is_terminator()
                {
                    if let Some(result) = inst.result {
                        if let std::collections::hash_map::Entry::Vacant(e) = id_map.entry(result) {
                            e.insert(next_id);
                            next_id += 1;
                        }
                    }
                }
            }
        }

        for old_id in self.values.keys().copied().collect::<Vec<_>>() {
            if let hash_map::Entry::Vacant(e) = id_map.entry(old_id) {
                e.insert(next_id);
                next_id += 1;
            }
        }

        //
        // rebuild
        //
        let mut new_values = HashMap::<ValueId, Value>::new();

        for (old_id, value) in &self.values {
            let Some(&new_id) = id_map.get(old_id) else {
                continue;
            };

            new_values.insert(new_id, value.clone());
        }

        //
        // rewrite
        //
        let mut new_insts = HashMap::<InstId, Inst>::new();

        for (&inst_id, inst) in &self.insts {
            let mut inst = inst.clone();

            inst.operands = inst
                .operands
                .iter()
                .map(|v| id_map.get(v).copied().unwrap_or(*v))
                .collect();

            if let Some(result) = inst.result {
                inst.result = id_map.get(&result).copied();
            }

            new_insts.insert(inst_id, inst);
        }

        for &block_id in &block_ids {
            let old_insts = self.blocks[&block_id].insts.clone();

            let mut consts = Vec::new();
            let mut others = Vec::new();

            for inst_id in old_insts {
                let Some(inst) = new_insts.get(&inst_id) else {
                    continue;
                };

                if matches!(inst.kind, InstKind::IConst(_) | InstKind::FConst(_)) {
                    consts.push(inst_id);
                } else if !inst.kind.is_terminator() {
                    others.push(inst_id);
                }
            }

            let mut reordered = Vec::new();
            reordered.extend(consts);
            reordered.extend(others);

            if let Some(term) = self.blocks[&block_id].term {
                reordered.push(term);
            }

            self.blocks.get_mut(&block_id).unwrap().insts = reordered;
        }

        for block in self.blocks.values_mut() {
            block.params = block
                .params
                .iter()
                .map(|v| id_map.get(v).copied().unwrap_or(*v))
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

    pub fn recompute_control_flow(&mut self) {
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
        self.reassign_values();
        self.recompute_uses();
        self.recompute_control_flow();
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
        log::trace!("removing inst {}", inst_id);

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

    pub fn get_values(&self) -> &HashMap<ValueId, Value> {
        &self.values
    }

    pub fn get_value(&self, v: ValueId) -> Option<&Value> {
        self.get_values().get(&v)
    }

    pub fn get_values_mut(&mut self) -> &mut HashMap<ValueId, Value> {
        &mut self.values
    }

    pub fn get_value_mut(&mut self, v: ValueId) -> Option<&mut Value> {
        self.get_values_mut().get_mut(&v)
    }

    pub fn get_insts(&self) -> &HashMap<InstId, Inst> {
        &self.insts
    }

    pub fn get_inst(&self, id: InstId) -> Option<&Inst> {
        self.insts.get(&id)
    }

    pub fn get_insts_mut(&mut self) -> &mut HashMap<InstId, Inst> {
        &mut self.insts
    }

    pub fn get_inst_mut(&mut self, id: InstId) -> Option<&mut Inst> {
        self.insts.get_mut(&id)
    }

    pub fn replace_uses(&mut self, from: ValueId, to: ValueId) {
        log::trace!("replacing %{} => %{}", from, to);

        let uses = self.get_uses(from).to_vec();
        for u in uses {
            if let Some(inst) = self.get_inst_mut(u.inst) {
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

    pub fn clear_uses(&mut self, val: ValueId) {
        if let Some(v) = self.values.get_mut(&val) {
            v.uses.clear();
        }
    }

    pub fn uses_iter(&self, v: ValueId) -> impl Iterator<Item = Use> + '_ {
        self.values.get(&v).into_iter().flat_map(|v| v.uses.clone())
    }

    pub fn get_uses(&self, v: ValueId) -> &[Use] {
        match self.values.get(&v) {
            Some(v) => &v.uses,
            None => &[],
        }
    }

    pub fn users_iter(&self, v: ValueId) -> impl Iterator<Item = InstId> + '_ {
        self.values
            .get(&v)
            .into_iter()
            .flat_map(|val| val.uses.iter().map(|u| u.inst))
    }

    pub fn get_users(&self, v: ValueId) -> Vec<InstId> {
        self.users_iter(v).collect()
    }
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    pub fn define_function(
        &mut self,
        id: FuncId,
        def: impl Into<FunctionDef>,
    ) -> Result<(), String> {
        let func = self
            .functions
            .get_mut(&id)
            .ok_or_else(|| format!("function {} not found", id))?;

        if func.definition.is_some() {
            return Err(format!("function {} already defined", id));
        }

        func.definition = Some(def.into());
        Ok(())
    }

    pub fn set_definition(
        &mut self,
        id: FuncId,
        def: impl Into<FunctionDef>,
    ) -> Result<(), String> {
        let func = self
            .functions
            .get_mut(&id)
            .ok_or_else(|| format!("function {} not found", id))?;

        func.definition = Some(def.into());
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
