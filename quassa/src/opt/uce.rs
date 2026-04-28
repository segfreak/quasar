use crate::ir::*;

use crate::prelude::HashSet;

pub fn uce(func: &mut FunctionDef) -> bool {
    let mut changed = false;

    let mut reachable = HashSet::new();
    let mut stack = vec![func.entry];

    while let Some(b) = stack.pop() {
        if !reachable.insert(b) {
            continue;
        }

        if let Some(block) = func.blocks.get(&b) {
            for &s in &block.succs {
                stack.push(s);
            }
        }
    }

    let before = func.blocks.len();

    func.blocks.retain(|b, _| reachable.contains(b));

    let after = func.blocks.len();

    if after != before {
        changed = true;
    }

    changed
}
