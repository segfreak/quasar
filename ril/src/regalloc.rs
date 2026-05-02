use enum_display::EnumDisplay;
use quasar::*;

use crate::{ir::VReg, live::*};

#[derive(Debug, EnumDisplay, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RegClass {
    /// general purpose class
    #[display("general_purpose")]
    General,

    /// xmm class (floats, vectors)
    #[display("xmm")]
    Xmm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Reg {
    /// reg identifier
    pub id: u32,
    /// type of register
    ///  rax: i64
    ///  eax: i32
    ///  etc
    pub ty: Type,
    /// class of register
    pub class: RegClass,
}

impl Reg {
    pub fn new(id: u32, ty: Type, class: RegClass) -> Self {
        Self { id, ty, class }
    }
}

impl std::fmt::Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r{}", self.id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegSet {
    gpr: Vec<Reg>,
    xmm: Vec<Reg>,
}

impl RegSet {
    pub fn new(regs: Vec<Reg>) -> Self {
        let mut regs = regs.clone();
        regs.sort_by_key(|r| r.id);

        let mut gpr = Vec::new();
        let mut xmm = Vec::new();

        for r in regs {
            match r.class {
                RegClass::General => gpr.push(r),
                RegClass::Xmm => xmm.push(r),
            }
        }

        Self { gpr, xmm }
    }

    pub fn alloc(&mut self, class: RegClass) -> Option<Reg> {
        match class {
            RegClass::General => self.gpr.pop(),
            RegClass::Xmm => self.xmm.pop(),
        }
    }

    pub fn free(&mut self, reg: Reg) {
        match reg.class {
            RegClass::General => self.gpr.push(reg),
            RegClass::Xmm => self.xmm.push(reg),
        }
    }

    pub fn len_class(&self, class: RegClass) -> usize {
        match class {
            RegClass::General => self.gpr.len(),
            RegClass::Xmm => self.xmm.len(),
        }
    }

    pub fn is_empty_class(&self, class: RegClass) -> bool {
        match class {
            RegClass::General => self.gpr.is_empty(),
            RegClass::Xmm => self.xmm.is_empty(),
        }
    }

    pub fn len(&self) -> usize {
        self.len_class(RegClass::General) + self.len_class(RegClass::Xmm)
    }

    pub fn is_empty(&self) -> bool {
        self.is_empty_class(RegClass::General) && self.is_empty_class(RegClass::Xmm)
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
    Register(Reg),
    Spill(StackSlot),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegAlloc {
    intervals: Vec<Interval>,
    active: Vec<Active>,
    /// registered physical registers
    /// dont touch by hands
    phys_regs: Vec<Reg>,
    free_regs: RegSet,
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
    reg: Reg,
}

/// class of register for type
fn class_of(ty: Type) -> RegClass {
    match ty {
        Type::F32 | Type::F64 => RegClass::Xmm,
        _ => RegClass::General,
    }
}

impl RegAlloc {
    pub fn new(mut intervals: Vec<Interval>, regs: Vec<Reg>) -> Self {
        intervals.sort_by_key(|i| i.start);

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
        }
    }

    fn expire_old(&mut self, current: u32) {
        while let Some(a) = self.active.first() {
            if a.interval.end < current {
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

            let class = class_of(i.vreg.ty);

            if let Some(reg) = self.free_regs.alloc(class) {
                log::debug!(
                    "alloc {} {}:{} live({}..{}) => r{}:{}",
                    class,
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
                    "spill {}:{} live({}..{}) => {} (no register left)",
                    i.vreg.id,
                    i.vreg.ty,
                    i.start,
                    i.end,
                    spill
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
