use std::collections::{HashMap, HashSet};

use crate::ssa::*;

type DomSet = HashSet<BlockId>;

pub struct Dominance {
    pub dom: HashMap<BlockId, DomSet>,
    pub idom: HashMap<BlockId, BlockId>,
}

impl Dominance {
    pub fn build(func: &FunctionDef) -> Self {
        compute_dominance(func)
    }
}

fn intersect(a: &DomSet, b: &DomSet) -> DomSet {
    a.intersection(b).copied().collect()
}

fn compute_dominance(func: &FunctionDef) -> Dominance {
    let mut dom: HashMap<BlockId, DomSet> = HashMap::new();

    let blocks: Vec<BlockId> = func.blocks.keys().copied().collect();
    let entry = func.entry;

    let all: DomSet = blocks.iter().copied().collect();

    for &b in &blocks {
        dom.insert(b, all.clone());
    }

    dom.insert(entry, {
        let mut s = HashSet::new();
        s.insert(entry);
        s
    });

    let mut changed = true;

    while changed {
        changed = false;

        for &b in &blocks {
            if b == entry {
                continue;
            }

            let preds = &func.blocks[&b].preds;

            if preds.is_empty() {
                continue;
            }

            let mut new_dom = all.clone();

            for &p in preds {
                if let Some(pd) = dom.get(&p) {
                    new_dom = intersect(&new_dom, pd);
                }
            }

            new_dom.insert(b);

            if new_dom != dom[&b] {
                dom.insert(b, new_dom);
                changed = true;
            }
        }
    }

    let idom = compute_idom(&dom, &blocks, entry);

    Dominance { dom, idom }
}

fn compute_idom(
    dom: &HashMap<BlockId, DomSet>,
    blocks: &[BlockId],
    entry: BlockId,
) -> HashMap<BlockId, BlockId> {
    let mut idom = HashMap::new();

    for &b in blocks {
        if b == entry {
            continue;
        }

        let doms = &dom[&b];

        let candidates: Vec<BlockId> = doms.iter().copied().filter(|x| *x != b).collect();

        if candidates.is_empty() {
            continue;
        }

        let mut best = candidates[0];

        for &c in &candidates {
            if dominates(dom, best, c) {
                best = c;
            }
        }

        idom.insert(b, best);
    }

    idom
}

fn dominates(dom: &HashMap<BlockId, DomSet>, a: BlockId, b: BlockId) -> bool {
    dom[&b].contains(&a)
}

pub struct DomTree {
    pub idom: HashMap<BlockId, BlockId>,
    pub children: HashMap<BlockId, Vec<BlockId>>,
}

impl DomTree {
    pub fn build(idom: &HashMap<BlockId, BlockId>) -> DomTree {
        let mut children: HashMap<BlockId, Vec<BlockId>> = HashMap::new();

        for (&b, &p) in idom {
            children.entry(p).or_default().push(b);
        }

        DomTree {
            idom: idom.clone(),
            children,
        }
    }
}
