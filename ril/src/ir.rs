use mirssa::ir::CmpKind;
use quasar::{target::*, *};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VReg {
    pub id: u32,
    pub ty: Type,
}

impl VReg {
    pub fn new(id: u32, ty: Type) -> Self {
        Self { id, ty }
    }
}

impl std::fmt::Display for VReg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}", self.id)
    }
}

pub type InstId = u32;
pub type BlockId = u32;
pub type FuncId = u32;

#[derive(Debug, Clone)]
pub enum Operand {
    VReg(VReg),
    Immediate(i64),
}

impl std::fmt::Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Immediate(imm) => write!(f, "{}", imm),
            Operand::VReg(reg) => write!(f, "{}", reg),
        }
    }
}

impl Operand {
    pub fn imm(x: i64) -> Self {
        Self::Immediate(x)
    }

    pub fn reg(id: u32, ty: Type) -> Self {
        Self::VReg(VReg { id, ty })
    }
}

#[derive(Debug, Clone)]
pub enum Inst {
    Mov {
        dst: VReg,
        src: Operand,
    },
    Add {
        dst: VReg,
        a: Operand,
        b: Operand,
    },
    Sub {
        dst: VReg,
        a: Operand,
        b: Operand,
    },
    Mul {
        dst: VReg,
        a: Operand,
        b: Operand,
    },
    SDiv {
        dst: VReg,
        a: Operand,
        b: Operand,
    },
    UDiv {
        dst: VReg,
        a: Operand,
        b: Operand,
    },
    Cmp {
        a: Operand,
        b: Operand,
    },
    Jmp {
        target: BlockId,
    },
    JmpIf {
        cond: CmpKind,
        t: BlockId,
        f: BlockId,
    },
    Load {
        dst: VReg,
        ptr: Operand,
    },
    Store {
        ptr: Operand,
        val: Operand,
    },
    Call {
        dst: Option<VReg>,
        func: FuncId,
        args: Vec<Operand>,
    },
    Ret {
        val: Option<Operand>,
    },
}

impl Inst {
    pub fn get_def(&self) -> Option<&VReg> {
        match self {
            Inst::Mov { dst, .. } => Some(dst),

            Inst::Add { dst, .. }
            | Inst::Sub { dst, .. }
            | Inst::Mul { dst, .. }
            | Inst::SDiv { dst, .. }
            | Inst::UDiv { dst, .. }
            | Inst::Load { dst, .. } => Some(dst),

            Inst::Call { dst: Some(d), .. } => Some(d),

            _ => None,
        }
    }

    pub fn get_uses(&self) -> Vec<VReg> {
        let mut out = Vec::new();

        let mut push = |op: &Operand| {
            if let Operand::VReg(r) = op {
                out.push(*r);
            }
        };

        match self {
            Inst::Mov { src, .. } => {
                push(src);
            }

            Inst::Add { a, b, .. }
            | Inst::Sub { a, b, .. }
            | Inst::Mul { a, b, .. }
            | Inst::SDiv { a, b, .. }
            | Inst::UDiv { a, b, .. } => {
                push(a);
                push(b);
            }

            Inst::Cmp { a, b, .. } => {
                push(a);
                push(b);
            }

            Inst::Load { ptr, .. } => {
                push(ptr);
            }

            Inst::Store { ptr, val } => {
                push(ptr);
                push(val);
            }

            Inst::Call { args, .. } => {
                for a in args {
                    push(a);
                }
            }

            Inst::Ret { val } => {
                if let Some(v) = val {
                    push(v);
                }
            }

            Inst::Jmp { .. } | Inst::JmpIf { .. } => {}
        }

        out
    }
}

#[derive(Debug, Default, Clone)]
pub struct Block {
    pub insts: Vec<Inst>,
    pub succs: Vec<BlockId>,
    pub preds: Vec<BlockId>,
}

#[derive(Debug, Default, Clone)]
pub struct FunctionDef {
    pub blocks: Vec<Block>,
    pub inst_map: HashMap<InstId, (BlockId, usize)>,

    pub entry: BlockId,
    pub next_block: BlockId,
    pub next_inst: InstId,
}

impl FunctionDef {
    pub fn new() -> Self {
        let mut f = Self::default();
        let entry = f.new_block();
        f.entry = entry;
        f
    }

    pub fn new_block(&mut self) -> BlockId {
        let id = self.next_block;
        self.next_block += 1;

        if id as usize == self.blocks.len() {
            self.blocks.push(Block::default());
        } else {
            self.blocks.insert(id as usize, Block::default());
        }

        id
    }

    pub fn lookup_block(&self, b: BlockId) -> Option<&Block> {
        self.blocks.get(b as usize)
    }

    pub fn append_inst(&mut self, block: BlockId, inst: Inst) -> InstId {
        let inst_id = self.next_inst;
        self.next_inst += 1;

        let block_ref = self.blocks.get_mut(block as usize).unwrap();

        let idx = block_ref.insts.len();
        block_ref.insts.push(inst);

        self.inst_map.insert(inst_id, (block, idx));

        inst_id
    }

    pub fn remove_inst(&mut self, inst_id: InstId) {
        let (block, idx) = match self.inst_map.remove(&inst_id) {
            Some(v) => v,
            None => return,
        };

        let block_ref = &mut self.blocks[block as usize];

        let last_idx = block_ref.insts.len() - 1;
        block_ref.insts.swap(idx, last_idx);

        block_ref.insts.pop();

        if idx != last_idx {
            let moved_inst_id = self
                .inst_map
                .iter()
                .find(|(_, (b, i))| *b == block && *i == last_idx)
                .map(|(id, _)| *id);

            if let Some(moved_id) = moved_inst_id {
                self.inst_map.insert(moved_id, (block, idx));
            }
        }
    }

    pub fn replace_inst(&mut self, id: InstId, to: Inst) -> bool {
        let (block, idx) = match self.inst_map.get(&id).copied() {
            Some(v) => v,
            None => return false,
        };

        let block_ref = &mut self.blocks[block as usize];
        block_ref.insts[idx] = to;

        true
    }

    pub fn make_mov(&mut self, block: BlockId, dst: VReg, src: Operand) -> InstId {
        self.append_inst(block, Inst::Mov { dst, src })
    }

    pub fn make_add(&mut self, block: BlockId, dst: VReg, a: Operand, b: Operand) -> InstId {
        self.append_inst(block, Inst::Add { dst, a, b })
    }

    pub fn make_sub(&mut self, block: BlockId, dst: VReg, a: Operand, b: Operand) -> InstId {
        self.append_inst(block, Inst::Sub { dst, a, b })
    }

    pub fn make_mul(&mut self, block: BlockId, dst: VReg, a: Operand, b: Operand) -> InstId {
        self.append_inst(block, Inst::Mul { dst, a, b })
    }

    pub fn make_sdiv(&mut self, block: BlockId, dst: VReg, a: Operand, b: Operand) -> InstId {
        self.append_inst(block, Inst::SDiv { dst, a, b })
    }

    pub fn make_udiv(&mut self, block: BlockId, dst: VReg, a: Operand, b: Operand) -> InstId {
        self.append_inst(block, Inst::UDiv { dst, a, b })
    }

    pub fn make_cmp(&mut self, block: BlockId, a: Operand, b: Operand) -> InstId {
        self.append_inst(block, Inst::Cmp { a, b })
    }

    pub fn make_load(&mut self, block: BlockId, dst: VReg, ptr: Operand) -> InstId {
        self.append_inst(block, Inst::Load { dst, ptr })
    }

    pub fn make_store(&mut self, block: BlockId, ptr: Operand, val: Operand) -> InstId {
        self.append_inst(block, Inst::Store { ptr, val })
    }

    pub fn make_jmp(&mut self, block: BlockId, target: BlockId) -> InstId {
        let inst = Inst::Jmp { target };

        self.blocks[block as usize].succs.push(target);

        self.append_inst(block, inst)
    }

    pub fn make_jmpif(&mut self, block: BlockId, cond: CmpKind, t: BlockId, f: BlockId) -> InstId {
        let inst = Inst::JmpIf { cond, t, f };

        let b = &mut self.blocks[block as usize];
        b.succs.push(t);
        b.succs.push(f);

        self.append_inst(block, inst)
    }

    pub fn make_call(
        &mut self,
        block: BlockId,
        dst: Option<VReg>,
        func: FuncId,
        args: Vec<Operand>,
    ) -> InstId {
        self.append_inst(block, Inst::Call { dst, func, args })
    }

    pub fn make_ret(&mut self, block: BlockId, val: Option<Operand>) -> InstId {
        self.append_inst(block, Inst::Ret { val })
    }

    pub fn cfg(&mut self) {
        for block in &mut self.blocks {
            block.preds.clear();
        }

        // rebuild from succs
        let len = self.blocks.len();

        for bid in 0..len {
            let succs = self.blocks[bid].succs.clone();

            for &succ in &succs {
                self.blocks[succ as usize].preds.push(bid as BlockId);
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

impl FunctionDef {
    #[allow(unused)]
    fn fmt_args(args: &[Operand]) -> String {
        args.iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<_>>()
            .join(", ")
    }

    #[allow(unused)]
    fn fmt_inst(&self, module: &Module, func: &FunctionDef, inst: &Inst) -> String {
        match inst {
            Inst::Add { a, b, .. } => format!("add {}, {}", a, b),
            Inst::Sub { a, b, .. } => format!("sub {}, {}", a, b),
            Inst::Mul { a, b, .. } => format!("mul {}, {}", a, b),
            Inst::SDiv { a, b, .. } => format!("sdiv {}, {}", a, b),
            Inst::UDiv { a, b, .. } => format!("udiv {}, {}", a, b),

            Inst::Ret { val } => {
                if let Some(v) = val {
                    format!("ret {}", v)
                } else {
                    "ret".into()
                }
            }

            _ => todo!(),
        }
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

            // subgraph per function
            s.push_str(&format!("  subgraph cluster_f{} {{\n", fid));
            s.push_str(&format!(
                "    label=\"[{}] {}\";\n",
                func.calling_convention, func.name
            ));
            s.push_str("    style=rounded;\n\n");

            // blocks
            for (bid, block) in def.blocks.iter().enumerate() {
                let mut label = format!("B{}:\\l", bid);

                for inst in &block.insts {
                    if let Some(def) = inst.get_def() {
                        label.push_str(&format!("  {} = ", def));
                    } else {
                        label.push_str("  ");
                    }

                    label.push_str(&def.fmt_inst(self, def, inst));
                    label.push_str("\\l");
                }

                s.push_str(&format!("    f{}_b{} [label=\"{}\"];\n", fid, bid, label));
            }

            s.push('\n');

            for (bid, block) in def.blocks.iter().enumerate() {
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
