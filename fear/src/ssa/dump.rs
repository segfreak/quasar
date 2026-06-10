use crate::ssa::*;
use crate::types::Type;

impl FunctionDef {
    fn fmt_args(args: &[ValueId]) -> String {
        args.iter()
            .map(|v| format!("%{}", v))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn fmt_inst(&self, module: &Module, inst: &Inst) -> String {
        let ty = inst
            .result
            .map(|v| self.get_type_of(v))
            .unwrap_or(Type::Void);

        match &inst.kind {
            InstKind::IConst(x) => {
                format!(
                    "const.{:<6} {:<12} # hex: 0x{:X}, bits: 0b{:b}",
                    ty, x, x, x
                )
            }

            InstKind::FConst(x) => {
                format!(
                    "const.{:<6} {:<12} # hex: 0x{:X}, bits: 0b{:b}",
                    ty,
                    f64::from_bits(*x),
                    x,
                    x
                )
            }

            InstKind::Add => {
                format!(
                    "add.{:<6} {:<8} {:<8}",
                    ty,
                    format!("%{}", inst.operands[0]),
                    format!("%{}", inst.operands[1])
                )
            }

            InstKind::Sub => {
                format!(
                    "sub.{:<6} {:<8} {:<8}",
                    ty,
                    format!("%{}", inst.operands[0]),
                    format!("%{}", inst.operands[1])
                )
            }

            InstKind::Mul => {
                format!(
                    "mul.{:<6} {:<8} {:<8}",
                    ty,
                    format!("%{}", inst.operands[0]),
                    format!("%{}", inst.operands[1])
                )
            }

            InstKind::Div { signed } => {
                format!(
                    "{}div.{:<6} {:<8} {:<8}",
                    if !*signed { "u" } else { "s" },
                    ty,
                    format!("%{}", inst.operands[0]),
                    format!("%{}", inst.operands[1])
                )
            }

            InstKind::Rem { signed } => {
                format!(
                    "{}rem.{:<6} {:<8} {:<8}",
                    if !*signed { "u" } else { "s" },
                    ty,
                    format!("%{}", inst.operands[0]),
                    format!("%{}", inst.operands[1])
                )
            }

            InstKind::FAdd => {
                format!(
                    "add.{:<6} {:<8} {:<8}",
                    ty,
                    format!("%{}", inst.operands[0]),
                    format!("%{}", inst.operands[1])
                )
            }

            InstKind::FSub => {
                format!(
                    "sub.{:<6} {:<8} {:<8}",
                    ty,
                    format!("%{}", inst.operands[0]),
                    format!("%{}", inst.operands[1])
                )
            }

            InstKind::FMul => {
                format!(
                    "mul.{:<6} {:<8} {:<8}",
                    ty,
                    format!("%{}", inst.operands[0]),
                    format!("%{}", inst.operands[1])
                )
            }

            InstKind::FDiv => {
                format!(
                    "div.{:<6} {:<8} {:<8}",
                    ty,
                    format!("%{}", inst.operands[0]),
                    format!("%{}", inst.operands[1])
                )
            }

            InstKind::FRem => {
                format!(
                    "rem.{:<6} {:<8} {:<8}",
                    ty,
                    format!("%{}", inst.operands[0]),
                    format!("%{}", inst.operands[1])
                )
            }

            InstKind::Not => {
                format!("not.{:<6} {:<8}", ty, format!("%{}", inst.operands[0]))
            }

            InstKind::And => {
                format!(
                    "and.{:<6} {:<8} {:<8}",
                    ty,
                    format!("%{}", inst.operands[0]),
                    format!("%{}", inst.operands[1])
                )
            }

            InstKind::Or => {
                format!(
                    "or.{:<6} {:<8} {:<8}",
                    ty,
                    format!("%{}", inst.operands[0]),
                    format!("%{}", inst.operands[1])
                )
            }

            InstKind::Xor => {
                format!(
                    "xor.{:<6} {:<8} {:<8}",
                    ty,
                    format!("%{}", inst.operands[0]),
                    format!("%{}", inst.operands[1])
                )
            }

            InstKind::LShl => {
                format!(
                    "lshl.{:<6} {:<8} {:<8}",
                    ty,
                    format!("%{}", inst.operands[0]),
                    format!("%{}", inst.operands[1])
                )
            }

            InstKind::LShr => {
                format!(
                    "lshr.{:<6} {:<8} {:<8}",
                    ty,
                    format!("%{}", inst.operands[0]),
                    format!("%{}", inst.operands[1])
                )
            }

            InstKind::AShr => {
                format!(
                    "ashr.{:<6} {:<8} {:<8}",
                    ty,
                    format!("%{}", inst.operands[0]),
                    format!("%{}", inst.operands[1])
                )
            }

            InstKind::Cmp(k) => {
                format!(
                    "icmp.{:<6} {:<4} {:<8} {:<8}",
                    ty,
                    k,
                    format!("%{}", inst.operands[0]),
                    format!("%{}", inst.operands[1])
                )
            }

            InstKind::FCmp(k) => {
                format!(
                    "fcmp.{:<6} {:<4} {:<8} {:<8}",
                    ty,
                    k,
                    format!("%{}", inst.operands[0]),
                    format!("%{}", inst.operands[1])
                )
            }

            InstKind::Load { volatile } => {
                format!(
                    "{}load.{:<6} {:<8}",
                    if *volatile { "volatile " } else { "" },
                    ty,
                    format!("%{}", inst.operands[0])
                )
            }

            InstKind::Store { volatile } => {
                format!(
                    "{}store.{:<6} {:<8} {:<8}",
                    if *volatile { "volatile " } else { "" },
                    self.get_type_of(inst.operands[1]),
                    format!("%{}", inst.operands[0]),
                    format!("%{}", inst.operands[1]),
                )
            }

            InstKind::Alloca(t) => format!("alloca     {}", t),
            InstKind::NAlloca(t, size) => format!("nalloca    {} {}", t, size),

            InstKind::PtrOffset => {
                format!(
                    "ptroffset  {:<8} {:<8}",
                    format!("%{}", inst.operands[0]),
                    format!("%{}", inst.operands[1])
                )
            }

            InstKind::ElementPtr(ty) => {
                format!(
                    "elementptr.{:<6} {:<8} {:<8}",
                    ty,
                    format!("%{}", inst.operands[0]),
                    format!("%{}", inst.operands[1])
                )
            }

            InstKind::Call(fid) => {
                let name = &module.get_function(*fid).unwrap().name;
                format!("call       {}({})", name, Self::fmt_args(&inst.operands))
            }

            InstKind::Cast(k) => {
                format!("cast.{:<6} {:<8}", k, format!("%{}", inst.operands[0]))
            }

            InstKind::Jump(bb) => {
                if inst.operands.is_empty() {
                    format!("jmp        B{}", bb)
                } else {
                    let args = Self::fmt_args(&inst.operands);
                    format!("jmp        B{}({})", bb, args)
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
                    format!("ret.{:<6}", Type::Void)
                } else {
                    let op = inst.operands[0];
                    format!("ret.{:<6} %{}", self.get_type_of(op), op)
                }
            }

            InstKind::Undef => "undef".into(),
        }
    }

    pub fn dump(&self) -> String {
        let mut s = String::new();
        let m = Module::new("__unnamed");

        let blocks = self.compute_rpo();
        for bid in &blocks {
            let block = &self.blocks[bid];
            let is_entry = self.entry == *bid;
            if block.params.is_empty() || /* the entry block has no parameters of its own, only function parameters */ is_entry
            {
                s.push_str(&format!("B{}:", bid));
            } else {
                let bparams = block
                    .params
                    .iter()
                    .map(|p| format!("%{}: {}", p, self.values[p].ty))
                    .collect::<Vec<_>>()
                    .join(", ");

                s.push_str(&format!("B{}({}):", bid, bparams));
            }
            if is_entry {
                s.push_str(" __entry__\n");
            } else {
                s.push('\n');
            }

            for inst_id in &block.insts {
                let inst = &self.insts[inst_id];

                if let Some(res) = inst.result {
                    s.push_str(&format!("  %{} = ", res));
                } else {
                    s.push_str("  ");
                }

                s.push_str(&self.fmt_inst(&m, inst));
                s.push('\n');
            }
        }

        s
    }
}

impl Module {
    pub fn dump(&self) -> String {
        let mut s = String::new();

        s.push_str(&format!("# module \"{}\"\n\n", self.name));

        for func in self.functions.values() {
            let def = match func.get_definition() {
                Some(d) => d,
                None => {
                    let params = func
                        .signature
                        .params
                        .iter()
                        .map(|p| format!("{}", p))
                        .collect::<Vec<_>>()
                        .join(", ");

                    s.push_str(&format!(
                        "__abi({}) declare {} {}({}) -> {}\n",
                        func.calling_convention,
                        func.linkage,
                        func.name,
                        params,
                        func.signature.returns
                    ));

                    continue;
                }
            };

            let params = def
                .get_params()
                .iter()
                .map(|p| format!("%{}: {}", p, def.values[p].ty))
                .collect::<Vec<_>>()
                .join(", ");

            s.push_str(&format!(
                "__abi({}) define {} {}({}) -> {} {{\n",
                func.calling_convention, func.linkage, func.name, params, func.signature.returns
            ));

            let blocks = def.compute_rpo();
            for bid in &blocks {
                let block = &def.blocks[bid];
                let is_entry = def.entry == *bid;
                if block.params.is_empty() || /* the entry block has no parameters of its own, only function parameters */ is_entry
                {
                    s.push_str(&format!("B{}:", bid));
                } else {
                    let bparams = block
                        .params
                        .iter()
                        .map(|p| format!("%{}: {}", p, def.values[p].ty))
                        .collect::<Vec<_>>()
                        .join(", ");

                    s.push_str(&format!("B{}({}):", bid, bparams));
                }
                if is_entry {
                    s.push_str(" __entry__\n");
                } else {
                    s.push('\n');
                }

                for inst_id in &block.insts {
                    let inst = &def.insts[inst_id];

                    if let Some(res) = inst.result {
                        s.push_str(&format!("  %{} = ", res));
                    } else {
                        s.push_str("  ");
                    }

                    s.push_str(&def.fmt_inst(self, inst));
                    s.push('\n');
                }
            }

            s.push_str("}\n\n");
        }

        s
    }

    pub fn dump_dot(&self) -> String {
        let mut s = String::new();

        s.push_str(&format!("digraph {} {{\n", self.name));
        s.push_str("  node [shape=box, fontname=\"monospace\"];\n");
        s.push_str("  compound=true;\n\n");

        for (fid, func) in &self.functions {
            let def = match func.get_definition() {
                Some(d) => d,
                None => continue,
            };

            s.push_str(&format!("  subgraph cluster_f{} {{\n", fid));
            s.push_str(&format!(
                "    label=\"__abi({}) {}\";\n",
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

                    label.push_str(&def.fmt_inst(self, inst));
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
