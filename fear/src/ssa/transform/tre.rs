use crate::ssa::*;

// tail recursion elimination
pub fn tre(module: &mut Module, fid: FuncId) -> bool {
    let (entry_block, entry_params, ret_ty) = {
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
            def.blocks[&def.entry].params.clone(),
            func.signature.returns,
        )
    };

    let entry_param_count = entry_params.len();

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

    let def = module
        .get_function_mut(fid)
        .unwrap()
        .get_definition_mut()
        .unwrap();

    let loop_header = def.new_block();

    let loop_header_params: Vec<ValueId> = entry_params
        .iter()
        .map(|&p| {
            let ty = def.values[&p].ty;
            def.add_block_param(loop_header, ty)
        })
        .collect();

    for (&old_param, &new_param) in entry_params.iter().zip(loop_header_params.iter()) {
        def.replace_uses(old_param, new_param);
    }

    let entry_insts: Vec<InstId> = def.blocks[&entry_block].insts.clone();
    let entry_term = def.blocks[&entry_block].term;

    for &inst_id in &entry_insts {
        def.insts.get_mut(&inst_id).unwrap().parent = loop_header;
    }
    def.blocks.get_mut(&loop_header).unwrap().insts = entry_insts;
    def.blocks.get_mut(&entry_block).unwrap().insts = Vec::new();

    def.blocks.get_mut(&loop_header).unwrap().term = entry_term;
    def.blocks.get_mut(&entry_block).unwrap().term = None;

    let entry_succs = def.blocks[&entry_block].succs.clone();
    def.blocks.get_mut(&loop_header).unwrap().succs = entry_succs.clone();
    def.blocks.get_mut(&entry_block).unwrap().succs = vec![loop_header];

    for succ in &entry_succs {
        let preds = &mut def.blocks.get_mut(succ).unwrap().preds;
        for p in preds.iter_mut() {
            if *p == entry_block {
                *p = loop_header;
            }
        }
    }

    def.blocks.get_mut(&loop_header).unwrap().preds = vec![entry_block];

    let entry_param_ids = def.blocks[&entry_block].params.clone();
    def.make_jump(entry_block, loop_header, entry_param_ids);

    for (call_id, term_id, bid) in candidates {
        let call_args = def.insts[&call_id].operands.clone();

        def.remove_inst(call_id);

        let new_term = Inst {
            kind: InstKind::Jump(loop_header),
            operands: call_args,
            parent: bid,
            result: None,
        };

        def.replace_inst(term_id, new_term);

        {
            let block = def.blocks.get_mut(&bid).unwrap();
            block.term = Some(term_id);

            if !block.succs.contains(&loop_header) {
                block.succs.push(loop_header);
            }
        }

        {
            let header = def.blocks.get_mut(&loop_header).unwrap();
            if !header.preds.contains(&bid) {
                header.preds.push(bid);
            }
        }
    }

    true
}
