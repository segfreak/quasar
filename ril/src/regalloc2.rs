use crate::interval::Interval;
use crate::ir::VReg;
use crate::regalloc::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RegAlloc2 {
    intervals: Vec<Interval>,
    active: Vec<Active>,
    phys_regs: Vec<Reg>,
    free_regs: RegSet,
    result: RegAllocResult,
    next_stack_offset: usize,
    spills_counter: usize,

    vreg_weights: HashMap<VReg, f32>,
}

impl RegAlloc2 {
    pub fn new(mut intervals: Vec<Interval>, regs: Vec<Reg>) -> Self {
        intervals.sort_by_key(|i| i.start);

        let mut weights = HashMap::new();
        let mut priority = HashMap::new();

        for interval in &intervals {
            let length = (interval.end - interval.start) as f32;
            let weight = 1.0 / (length + 1.0);
            weights.insert(interval.vreg, weight);

            let priority_val = length;
            priority.insert(interval.vreg, priority_val);
        }

        log::debug!("intervals: {:?}", intervals);
        log::debug!("phys regs: {:?}", regs);

        Self {
            intervals,
            active: Vec::new(),
            phys_regs: regs.clone(),
            free_regs: RegSet::new(regs),
            result: RegAllocResult::default(),
            next_stack_offset: 0,
            spills_counter: 0,
            vreg_weights: weights,
        }
    }

    fn expire_old(&mut self, current: u32) {
        while let Some(a) = self.active.first() {
            if a.interval.end <= current {
                let a = self.active.remove(0);
                self.free_regs.free(a.reg);
            } else {
                break;
            }
        }
    }

    fn add_active(&mut self, a: Active) {
        let pos = self
            .active
            .binary_search_by_key(&a.interval.end, |x| x.interval.end)
            .unwrap_or_else(|e| e);

        self.active.insert(pos, a);
    }

    #[allow(clippy::manual_div_ceil)]
    fn make_spill(&mut self, ty: quasar::Type) -> StackSlot {
        self.spills_counter += 1;
        let size = ty.get_size();
        let align = size.max(8);
        self.next_stack_offset = (self.next_stack_offset + align - 1) / align * align;
        let offset = self.next_stack_offset;
        self.next_stack_offset += size;
        StackSlot { offset, ty, align }
    }

    fn report_spills(&self) {
        if self.spills_counter > 5 {
            log::warn!(
                "high spill rate: spills={}, active={}, phys_regs={}, free_regs={}, next_stack={}",
                self.spills_counter,
                self.active.len(),
                self.phys_regs.len(),
                self.free_regs.len(),
                self.next_stack_offset,
            );
        } else {
            log::debug!(
                "spill stats: spills={}, active={}, stack={}",
                self.spills_counter,
                self.active.len(),
                self.next_stack_offset,
            );
        }
    }

    fn spill_cost(&self, vreg: VReg, interval: &Interval) -> f32 {
        let weight = self.vreg_weights.get(&vreg).copied().unwrap_or(1.0);
        let length = (interval.end - interval.start) as f32;
        weight * length
    }

    pub fn linear_scan(&mut self) -> RegAllocResult {
        for i in self.intervals.clone().iter() {
            self.expire_old(i.start);

            let class = class_of(i.vreg.ty);

            if let Some(reg) = self.free_regs.alloc(class) {
                self.result.0.insert(i.vreg, Slot::Register(reg));
                self.add_active(Active { interval: *i, reg });
                continue;
            }

            let mut victim_idx: Option<usize> = None;
            let mut best_cost = f32::MAX;

            for (idx, a) in self.active.iter().enumerate() {
                if a.reg.class == class {
                    let cost = self.spill_cost(a.interval.vreg, &a.interval);
                    if cost < best_cost {
                        best_cost = cost;
                        victim_idx = Some(idx);
                    }
                }
            }

            if let Some(idx) = victim_idx {
                let victim = self.active[idx];
                let current_cost = self.spill_cost(i.vreg, i);

                if victim.interval.end > i.end && best_cost > current_cost {
                    let spill = self.make_spill(victim.interval.vreg.ty);
                    self.result
                        .0
                        .insert(victim.interval.vreg, Slot::Spill(spill));

                    let reg = victim.reg;
                    self.active.remove(idx);

                    self.result.0.insert(i.vreg, Slot::Register(reg));
                    self.add_active(Active { interval: *i, reg });
                    continue;
                }
            }

            let spill = self.make_spill(i.vreg.ty);
            self.result.0.insert(i.vreg, Slot::Spill(spill));
        }

        self.report_spills();
        self.result.clone()
    }

    pub fn get_stack_frame_size(&self) -> usize {
        self.next_stack_offset
    }

    pub fn get_result(&self) -> RegAllocResult {
        self.result.clone()
    }
}

pub fn class_of(ty: quasar::Type) -> RegClass {
    match ty {
        quasar::Type::F32 | quasar::Type::F64 => RegClass::Xmm,
        _ => RegClass::General,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_improved_regalloc() {
        use quasar::Type;

        let intervals = vec![
            Interval {
                vreg: VReg::new(0, Type::I32),
                start: 0,
                end: 10,
            },
            Interval {
                vreg: VReg::new(1, Type::I32),
                start: 5,
                end: 15,
            },
        ];

        let regs = vec![
            Reg::new(0, Type::I32, RegClass::General),
            Reg::new(1, Type::I32, RegClass::General),
        ];

        let mut alloc = RegAlloc2::new(intervals, regs);
        let _result = alloc.linear_scan();

        assert!(alloc.spills_counter <= 1, "Should have minimal spills");
    }
}
