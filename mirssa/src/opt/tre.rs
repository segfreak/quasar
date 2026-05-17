use crate::ir::*;

pub fn tre(module: &mut Module, fid: FuncId) -> bool {
    let (entry_block, entry_param_count, ret_ty) = {
        let func = match module.get_function(fid) {
            Some(f) => f,
            None => return false,
        };
        let def = match func.get_definition() {
            Some(d) => d,
            None => return false,
        };
        (
            def.entry,
            def.blocks[&def.entry].params.len(),
            func.signature.returns,
        )
    };

    let candidates: Vec<(InstId, InstId, BlockId)> = {
        let func = module.get_function(fid).unwrap();
        let def = func.get_definition().unwrap();

        let mut found = Vec::new();

        for (&bid, block) in &def.blocks {
            let term_id = match block.term {
                Some(t) => t,
                None => continue,
            };

            let term = &def.insts[&term_id];
            if !matches!(term.kind, InstKind::Ret) {
                continue;
            }

            let preceding_id = block.insts.iter().rev().nth(1).copied();

            let call_id = match preceding_id {
                Some(id) => id,
                None => continue,
            };

            let call_inst = &def.insts[&call_id];

            if !matches!(call_inst.kind, InstKind::Call(callee) if callee == fid) {
                continue;
            }

            let tail = match term.operands.first() {
                None => ret_ty.is_void(),
                Some(&ret_val) => call_inst.result == Some(ret_val),
            };

            if !tail {
                continue;
            }

            if call_inst.operands.len() != entry_param_count {
                continue;
            }

            found.push((call_id, term_id, bid));
        }

        found
    };

    if candidates.is_empty() {
        return false;
    }

    for (call_id, term_id, bid) in candidates {
        let def = module
            .get_function_mut(fid)
            .unwrap()
            .get_definition_mut()
            .unwrap();

        let call_args = def.insts[&call_id].operands.clone();

        def.remove_inst(call_id);

        let new_term = Inst {
            kind: InstKind::Jump(entry_block),
            operands: call_args.clone(),
            parent: bid,
            result: None,
        };

        def.replace_inst(term_id, new_term);

        {
            let block = def.blocks.get_mut(&bid).unwrap();
            block.term = Some(term_id);

            if !block.succs.contains(&entry_block) {
                block.succs.push(entry_block);
            }
        }

        {
            let entry = def.blocks.get_mut(&entry_block).unwrap();
            if !entry.preds.contains(&bid) {
                entry.preds.push(bid);
            }
        }
    }

    true
}
