use crate::*;
use alloc::format;

pub struct ElfError
{
    pub msg: String
}

pub struct ElfParser
{

}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ElfHeader
{
    pub magic: u32,
    pub bitsize: u8,
    pub endian: u8,
    pub ident_abi_version: u8,
    pub target_platform: u8,
    pub abi_version: u8,
    pub padding: [u8; 7],
    pub obj_type: u16,
    pub machine: u16, // 0xf3 for RISC-V
    pub version: u32,
    pub entry_addr: usize,
    pub phoff: usize,
    pub shoff: usize,
    pub flags: u32,
    pub ehsize: u16,
    pub phentsize: u16,
    pub phnum: u16,
    pub shentsize: u16,
    pub shnum: u16,
    pub shstrndx: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ProgramHeader
{
    pub seg_type: u32,
    pub flags: u32,
    pub off: usize,
    pub vaddr: usize,
    pub paddr: usize,
    pub filesz: usize,
    pub memsz: usize,
    pub align: usize,
}

#[derive(Debug)]
pub struct Segment
{
    vaddr: usize,
    f_offset: usize,
    fsize: usize,
    msize: usize,
    flags: usize,
}


pub fn load_elf(buffer: &[u8]) -> Result<process::process::ProcessData, ElfError>
{
    kdebugln!(ElfParsing, "Attempting to parse elf file");

    let header = unsafe { *(buffer as *const [u8] as *const ElfHeader) };

    if header.magic != 0x464c457f
    {
        return Err(ElfError{msg: format!("File is not an Elf File")});
    }

    let prog_headers = unsafe { core::mem::transmute::<&[u8], &[ProgramHeader]>(&buffer[header.phoff..]) };

    let mut segments = Vec::new();

    for i in 0..header.phnum as usize
    {
        let phead = prog_headers[i];

        if phead.seg_type != 1
        {
            continue; // Skip any non load program headers
        }

        kdebugln!(ElfParsing, "{:x?}", phead);

        kdebug!(ElfParsing, "   Flags: ");

        let mut flags = mem::pagetable::EntryBits::User as usize;

        if phead.flags & 1 != 0
        {
            kdebug!(ElfParsing, "EXEC ");
            flags |= mem::pagetable::EntryBits::Execute as usize;
        }

        if phead.flags & 2 != 0
        {
            kdebug!(ElfParsing, "WRITE ");
            flags |= mem::pagetable::EntryBits::Write as usize;
        }

        if phead.flags & 4 != 0
        {
            kdebug!(ElfParsing, "READ ");
            flags |= mem::pagetable::EntryBits::Read as usize;
        }
        kdebugln!(ElfParsing);

        segments.push(
            Segment
            {
                vaddr: phead.vaddr,
                flags,
                msize: phead.memsz,
                fsize: phead.filesz,
                f_offset: phead.off
            }
        )
    }

    let table = unsafe { (mem::kpzalloc(1) as *mut mem::pagetable::Table).as_mut().unwrap() };

    for segment in segments
    {
        let phys_ptr = mem::kpzalloc((segment.msize + 4095) / 4096);

        let poff = segment.f_offset & (4096 - 1);

        for i in 0..segment.fsize
        {
            unsafe { phys_ptr.add(i + poff).write( buffer[segment.f_offset + i] ) }
        }
        mem::mmu::inner_map(table, segment.vaddr, phys_ptr as usize, mem::pagetable::EntryBits::UserReadWriteExecute, mem::mmu::MMUPageLevel::Level4KiB);
    }

    // Allocate space for four pages of stack space
    let stack_space = mem::kpzalloc(4);

    // Map the stack space
    for i in 0..4
    {
        mem::mmu::inner_map(table, 0x2000_0000 + 4096 * i, stack_space as usize + 4096 * i, 
            mem::pagetable::EntryBits::UserReadWrite, mem::mmu::MMUPageLevel::Level4KiB);
    }

    Ok(process::process::ProcessData::new_elf(table, 4, 0x2000_0000, header.entry_addr))
}