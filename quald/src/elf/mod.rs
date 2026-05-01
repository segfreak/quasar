use std::io;
use std::mem::size_of;
use std::slice;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64Ehdr {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64Phdr {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

pub struct Elf<'a> {
    pub data: &'a [u8],
    pub ehdr: &'a Elf64Ehdr,
}

impl<'a> Elf<'a> {
    #[inline(always)]
    pub fn parse(data: &'a [u8]) -> io::Result<Self> {
        if data.len() < size_of::<Elf64Ehdr>() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "too small"));
        }

        let ehdr = unsafe { &*(data.as_ptr() as *const Elf64Ehdr) };

        if &ehdr.e_ident[0..4] != b"\x7FELF" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "not ELF"));
        }

        Ok(Self { data, ehdr })
    }

    #[inline(always)]
    pub fn phdrs(&self) -> &[Elf64Phdr] {
        let phoff = self.ehdr.e_phoff as usize;
        let count = self.ehdr.e_phnum as usize;

        let ptr = unsafe { self.data.as_ptr().add(phoff) as *const Elf64Phdr };

        unsafe { slice::from_raw_parts(ptr, count) }
    }

    #[inline(always)]
    pub fn find_load_segments(&self) -> impl Iterator<Item = &Elf64Phdr> {
        self.phdrs().iter().filter(|p| p.p_type == 1) // PT_LOAD
    }
}
