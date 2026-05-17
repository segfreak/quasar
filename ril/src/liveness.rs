use crate::{cfg::CFG, ir::*, usedef::usedef_of_block};
use quasar::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockLiveness {
    pub live_in: HashMap<BlockId, HashSet<VReg>>,
    pub live_out: HashMap<BlockId, HashSet<VReg>>,
}

pub fn compute_block_liveness(cfg: &CFG) -> BlockLiveness {
    let mut usedefs = HashMap::new();
    for (i, b) in cfg.block_iter().enumerate() {
        usedefs.insert(i as BlockId, usedef_of_block(b));
    }

    let mut live_in: HashMap<BlockId, HashSet<VReg>> = HashMap::new();
    let mut live_out: HashMap<BlockId, HashSet<VReg>> = HashMap::new();

    for (i, _b) in cfg.block_iter().enumerate() {
        live_in.insert(i as BlockId, HashSet::new());
        live_out.insert(i as BlockId, HashSet::new());
    }

    loop {
        let mut changed = false;

        for (id, block) in cfg.block_iter().rev().enumerate() {
            let id = id as BlockId;

            // live_out = union live_in of successors
            let mut new_out = HashSet::new();
            for succ in &block.succs {
                new_out.extend(live_in[succ].iter().copied());
            }

            // live_in = use ∪ (live_out - def)
            let ud = &usedefs[&id];
            let mut new_in = ud.use_set.clone();

            for v in &new_out {
                if !ud.def_set.contains(v) {
                    new_in.insert(*v);
                }
            }

            if new_out != live_out[&(id as BlockId)] {
                live_out.insert(id, new_out);
                changed = true;
            }

            if new_in != live_in[&(id as BlockId)] {
                live_in.insert(id, new_in);
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }

    BlockLiveness { live_in, live_out }
}

pub fn inst_liveness(
    block: &Block,
    live_out_block: &HashSet<VReg>,
) -> HashMap<InstId, HashSet<VReg>> {
    let mut live = live_out_block.clone();
    let mut result = HashMap::new();

    for (id, inst) in block.insts.iter().enumerate().rev() {
        result.insert(id as InstId, live.clone());

        if let Some(def) = inst.get_def() {
            live.remove(def);
        }

        for u in inst.get_uses() {
            live.insert(u);
        }
    }

    result
}
