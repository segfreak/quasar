use quasar::*;

use crate::{ir::VReg, live::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysReg {
    /// reg identifier
    pub id: u32,
    /// type of register
    ///  rax: i64
    ///  eax: i32
    ///  etc
    pub ty: Type,
}

impl PhysReg {
    pub fn new(id: u32, ty: Type) -> Self {
        Self { id, ty }
    }
}

impl std::fmt::Display for PhysReg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r{}", self.id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StackSlot {
    /// stack offset
    ///  amd64: rsp - {offset}
    ///     because stack is growth down
    pub offset: usize,
    /// type of stack slot
    pub ty: Type,
    /// align of stack slot
    pub align: usize,
}

impl std::fmt::Display for StackSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "stack@{}", self.offset)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Slot {
    Register(PhysReg),
    Spill(StackSlot),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegAlloc {
    intervals: Vec<Interval>,
    active: Vec<Active>,
    /// registered physical registers
    /// dont touch by hands
    phys_regs: Vec<PhysReg>,
    free_regs: Vec<PhysReg>,
    result: RegAllocResult,
    next_stack_offset: usize,
    spills_counter: usize,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct RegAllocResult(pub HashMap<VReg, Slot>);

impl std::fmt::Display for RegAllocResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut items: Vec<_> = self.0.iter().collect();
        items.sort_by_key(|(v, _)| v.id);

        write!(f, "{{ ")?;

        for (i, (v, slot)) in items.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }

            match slot {
                Slot::Register(r) => {
                    write!(f, "{}: {}.{}", v, r, r.ty)?;
                }
                Slot::Spill(s) => {
                    write!(f, "{}: {}.{}", v, s, s.ty)?;
                }
            }
        }

        write!(f, " }}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Active {
    interval: Interval,
    reg: PhysReg,
}

impl RegAlloc {
    pub fn new(mut intervals: Vec<Interval>, regs: Vec<PhysReg>) -> Self {
        intervals.sort_by_key(|i| i.start);

        log::debug!("intervals: {:?}", intervals);
        log::debug!("phys regs: {:?}", regs);

        Self {
            intervals,
            active: Vec::new(),
            phys_regs: regs.clone(),
            free_regs: regs,
            result: RegAllocResult::default(),
            next_stack_offset: 0,
            spills_counter: 0,
        }
    }

    fn expire_old(&mut self, current: u32) {
        while let Some(a) = self.active.first() {
            if a.interval.end < current {
                let a = self.active.remove(0);
                self.free_regs.push(a.reg);
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
    fn make_spill(&mut self, ty: Type) -> StackSlot {
        self.spills_counter += 1;
        let size = ty.get_size();
        let align = size.max(8);
        self.next_stack_offset = (self.next_stack_offset + align - 1) / align * align;
        let offset = self.next_stack_offset;
        self.next_stack_offset += size;
        StackSlot { offset, ty, align }
    }

    pub fn run(mut self) -> RegAllocResult {
        for i in self.intervals.clone().iter() {
            self.expire_old(i.start);

            if let Some(reg) = self.free_regs.pop() {
                if reg.ty == i.vreg.ty {
                    log::debug!(
                        "alloc v{}:{} live({}..{}) => r{}:{}",
                        i.vreg.id,
                        i.vreg.ty,
                        i.start,
                        i.end,
                        reg.id,
                        reg.ty
                    );
                    self.result.0.insert(i.vreg, Slot::Register(reg));
                    self.add_active(Active { interval: *i, reg });
                } else {
                    let spill = self.make_spill(i.vreg.ty);
                    log::info!(
                        "spill v{}:{} live({}..{}) => stack@{} (no register left for {})",
                        i.vreg.id,
                        i.vreg.ty,
                        i.start,
                        i.end,
                        spill.offset,
                        i.vreg.ty,
                    );
                    self.result.0.insert(i.vreg, Slot::Spill(spill));
                }
            } else {
                let spill = self.make_spill(i.vreg.ty);
                log::info!(
                    "spill v{}:{} live({}..{}) => stack@{} (no register left)",
                    i.vreg.id,
                    i.vreg.ty,
                    i.start,
                    i.end,
                    spill.offset
                );
                self.result.0.insert(i.vreg, Slot::Spill(spill));
            }
        }

        if self.spills_counter > 5 {
            log::warn!(
                "high spill rate: spills={}, active={}, phys_regs={}, free_regs={}, next_stack={}",
                self.spills_counter,
                self.active.len(),
                self.phys_regs.len(),
                self.free_regs.len(),
                self.next_stack_offset,
            );
        }

        self.result
    }
}
