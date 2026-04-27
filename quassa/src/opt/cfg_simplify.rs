use crate::ir::*;

pub fn cfg_simplify(func: &mut FunctionDef) {
    let mut reachable = std::collections::HashSet::new();
    let mut stack = vec![func.entry];

    while let Some(b) = stack.pop() {
        if !reachable.insert(b) {
            continue;
        }
        for &s in &func.blocks[&b].succs {
            stack.push(s);
        }
    }

    func.blocks.retain(|b, _| reachable.contains(b));
}
