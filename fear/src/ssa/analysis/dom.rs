use std::collections::{HashMap, HashSet};

use crate::ssa::*;

pub struct Dominance {
    pub idom: HashMap<BlockId, BlockId>,
}

pub struct DomTree {
    pub idom: HashMap<BlockId, BlockId>,
    pub children: HashMap<BlockId, Vec<BlockId>>,
}

impl DomTree {
    pub fn build(idom: &HashMap<BlockId, BlockId>) -> Self {
        let mut children = HashMap::<BlockId, Vec<BlockId>>::new();

        for (&block, &parent) in idom {
            children.entry(parent).or_default().push(block);
        }

        Self {
            idom: idom.clone(),
            children,
        }
    }
}

impl Dominance {
    pub fn build(func: &FunctionDef) -> Self {
        Self {
            idom: compute_idom(func),
        }
    }
}

fn intersect(
    mut a: BlockId,
    mut b: BlockId,
    rpo_index: &HashMap<BlockId, usize>,
    idom: &HashMap<BlockId, BlockId>,
) -> BlockId {
    while a != b {
        while rpo_index[&a] > rpo_index[&b] {
            a = idom[&a];
        }

        while rpo_index[&b] > rpo_index[&a] {
            b = idom[&b];
        }
    }

    a
}

pub fn compute_idom(func: &FunctionDef) -> HashMap<BlockId, BlockId> {
    let rpo = func.compute_rpo();

    let mut rpo_index = HashMap::<BlockId, usize>::new();

    for (idx, &block) in rpo.iter().enumerate() {
        rpo_index.insert(block, idx);
    }

    let start = func.entry;

    let mut idom = HashMap::<BlockId, BlockId>::new();
    idom.insert(start, start);

    let mut changed = true;

    while changed {
        changed = false;

        for &block in rpo.iter().skip(1) {
            let preds = &func.blocks[&block].preds;

            let mut new_idom = None;

            for &pred in preds {
                if idom.contains_key(&pred) {
                    new_idom = Some(pred);
                    break;
                }
            }

            let mut new_idom = match new_idom {
                Some(v) => v,
                None => continue,
            };

            for &pred in preds {
                if pred == new_idom {
                    continue;
                }

                if !idom.contains_key(&pred) {
                    continue;
                }

                new_idom = intersect(pred, new_idom, &rpo_index, &idom);
            }

            let update = match idom.get(&block) {
                Some(old) => *old != new_idom,
                None => true,
            };

            if update {
                idom.insert(block, new_idom);
                changed = true;
            }
        }
    }

    idom.remove(&start);

    idom
}

pub fn dominates(
    a: BlockId,
    mut b: BlockId,
    idom: &HashMap<BlockId, BlockId>,
    entry: BlockId,
) -> bool {
    if a == b {
        return true;
    }

    while b != entry {
        let parent = match idom.get(&b) {
            Some(v) => *v,
            None => break,
        };

        if parent == a {
            return true;
        }

        b = parent;
    }

    false
}

pub fn compute_df(
    func: &FunctionDef,
    idom: &HashMap<BlockId, BlockId>,
) -> HashMap<BlockId, HashSet<BlockId>> {
    let mut df = HashMap::<BlockId, HashSet<BlockId>>::new();

    for &block in func.blocks.keys() {
        df.insert(block, HashSet::new());
    }

    for (&block, bb) in &func.blocks {
        if bb.preds.len() < 2 {
            continue;
        }

        let idom_block = match idom.get(&block) {
            Some(v) => *v,
            None => continue,
        };

        for &pred in &bb.preds {
            let mut runner = pred;

            while runner != idom_block {
                df.entry(runner).or_default().insert(block);

                let next = match idom.get(&runner) {
                    Some(v) => *v,
                    None => break,
                };

                runner = next;
            }
        }
    }

    df
}
