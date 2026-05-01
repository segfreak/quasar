use std::{fs::File, io};

use memmap2::Mmap;
use quald::elf::Elf;

pub fn map_file(path: &str) -> io::Result<Mmap> {
    let file = File::open(path)?;
    unsafe { Mmap::map(&file) }
}

fn main() -> std::io::Result<()> {
    let mmap = map_file("a.out")?;
    let elf = Elf::parse(&mmap)?;

    println!("entry: 0x{:x}", elf.ehdr.e_entry);

    for seg in elf.find_load_segments() {
        println!(
            "LOAD offset=0x{:x} vaddr=0x{:x} size=0x{:x}",
            seg.p_offset, seg.p_vaddr, seg.p_filesz
        );
    }

    Ok(())
}
