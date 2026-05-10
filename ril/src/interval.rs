use crate::ir::*;
use quasar::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Interval {
    pub vreg: VReg,
    pub start: InstId,
    pub end: InstId,
}

pub fn build_live_intervals(func: &FunctionDef) -> Vec<Interval> {
    let mut start: HashMap<VReg, InstId> = HashMap::new();
    let mut end: HashMap<VReg, InstId> = HashMap::new();

    let mut idx = 0u32;

    for block in &func.blocks {
        for inst in &block.insts {
            if let Some(def) = inst.get_def() {
                start
                    .entry(*def)
                    .and_modify(|s| *s = (*s).min(idx))
                    .or_insert(idx);
            }

            for u in inst.get_uses() {
                start
                    .entry(u)
                    .and_modify(|s| *s = (*s).min(idx))
                    .or_insert(idx);

                end.insert(u, idx);
            }

            idx += 1;
        }
    }

    start
        .into_iter()
        .map(|(v, s)| Interval {
            vreg: v,
            start: s,
            end: *end.get(&v).unwrap_or(&s),
        })
        .collect()
}
