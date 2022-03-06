//! ELF File Loader

use crate::*;

use super::loading;

use alloc::vec::Vec;
use libutils::paths::PathBuffer;

use super::process::Process;

use super::stats::MemoryStats;

/// Elf Header Structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ElfHeader
{
    ident_magic: u32,
    ident_class: u8,
    ident_data: u8,
    ident_version: u8,
    ident_os_abi: u8,
    ident_abi_version: u8,
    ident_pad0: [u8; 7],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16
}

/// Elf Program Header
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

/// Segment to load into program memory
#[derive(Debug)]
pub struct Segment
{
    vaddr: usize,
    f_offset: usize,
    fsize: usize,
    msize: usize,
    flags: mem::mmu::PageTableEntryFlags,
    align: usize
}


/// Load a file from a file interface and convert it to a process
pub fn load_elf(file_data: Vec<u8>, path: PathBuffer, args: &Vec<String>, envp: &Vec<String>) -> Result<Process, loading::ProcessLoadError>
{
    kdebugln!(Elf, "Loading ELF File `{}`", path);

    let mut text_size = 0;
    let mut data_size = 0;

    // Verify it is an elf file
    if file_data[0..4] != [0x7F, 'E' as u8, 'L' as u8, 'F' as u8]
    {
        return Err(loading::ProcessLoadError::NotAnELF)
    }

    // Get an instance of the Elf Header
    let elf_header = unsafe { (file_data.as_ptr() as *const ElfHeader).read() };

    // Verify the elf is a 64 bit elf file
    if elf_header.ident_class != 2
    {
        return Err(loading::ProcessLoadError::BadFormat(String::from("ELF File is a 32-bit ELF File")));
    }

    // Verify the elf is a risc-v elf file
    if elf_header.e_machine != 0xF3
    {
        return Err(loading::ProcessLoadError::BadFormat(String::from("ELF File is not a RISCV ELF File")));
    }

    // Extract the program headers
    let prog_headers = unsafe { core::mem::transmute::<&[u8], &[ProgramHeader]>(&file_data[elf_header.e_phoff as usize..]) };

    // Segments to write to memory
    let mut segments = Vec::new();

    // Iterate over the program headers
    for i in 0..elf_header.e_phnum
    {
        let header = &prog_headers[i as usize];

        // Skip any headers which are not LOAD
        if header.seg_type != 1
        {
            continue;
        }

        kdebug!(Elf, "   Flags: ");

        let mut flags = mem::mmu::PageTableEntryFlags::user();

        if header.flags & 1 != 0
        {
            kdebug!(Elf, "EXEC ");
            flags = flags | mem::mmu::PageTableEntryFlags::executable() | mem::mmu::PageTableEntryFlags::accessed();
        }

        if header.flags & 2 != 0
        {
            kdebug!(Elf, "WRITE ");
            flags = flags | mem::mmu::PageTableEntryFlags::writable() | mem::mmu::PageTableEntryFlags::dirty();
        }

        if header.flags & 4 != 0
        {
            kdebug!(Elf, "READ ");
            flags = flags | mem::mmu::PageTableEntryFlags::readable() | mem::mmu::PageTableEntryFlags::accessed();
        }
        kdebugln!(Elf, "{} bytes", header.memsz);

        if header.flags & 1 > 0
        {
            text_size += (header.memsz + mem::PAGE_SIZE - 1) / mem::PAGE_SIZE;
        }
        else
        {
            data_size += (header.memsz + mem::PAGE_SIZE - 1) / mem::PAGE_SIZE;
        }

        segments.push(
            Segment
            {
                vaddr: header.vaddr,
                flags,
                msize: header.memsz,
                fsize: header.filesz,
                f_offset: header.off,
                align: header.align
            }
        )
    }

    // Allocate a new page table
    let table = unsafe { (mem::kpzalloc(1, "ELF Page Table").unwrap() as *mut mem::mmu::PageTable).as_mut().unwrap() };

    // Map the segments
    for segment in segments
    {
        let poff = segment.f_offset & (segment.align - 1);

        let num_pages = (segment.msize + poff + mem::PAGE_SIZE - 1) / mem::PAGE_SIZE;
        let phys_ptr = mem::kpzalloc(num_pages, "ELF Segment").unwrap() as *mut u8;

        for i in 0..segment.fsize
        {
            unsafe { phys_ptr.add(i + poff).write( file_data[segment.f_offset + i] ) }
        }

        for i in 0..num_pages
        {
            table.map(segment.vaddr + i * mem::PAGE_SIZE, phys_ptr as usize + i * mem::PAGE_SIZE, segment.flags, 0);
        }
    }

    let stack_size = 1;

    // Allocate space for four pages of stack space
    let stack_space = mem::kpzalloc(stack_size, "ELF Stack Space").unwrap();

    // Map the stack space
    for i in 0..stack_size
    {
        table.map(super::process::STACK_END - mem::PAGE_SIZE - mem::PAGE_SIZE * i,
            stack_space + mem::PAGE_SIZE * i,
            mem::mmu::PageTableEntryFlags::user() | mem::mmu::PageTableEntryFlags::readable() | mem::mmu::PageTableEntryFlags::executable() | mem::mmu::PageTableEntryFlags::writable() | mem::mmu::PageTableEntryFlags::dirty() | mem::mmu::PageTableEntryFlags::accessed(),
            0);
    }

    let mem_stats = MemoryStats::new(0, 0, text_size, data_size + stack_size);

    let mut proc = Process::from_components(
        elf_header.e_entry as usize, 
        table as *mut mem::mmu::PageTable, 
        stack_size, super::process::STACK_END - mem::PAGE_SIZE * stack_size,
        mem_stats);

    let mut full_arguments = vec![path.as_str().to_string()];
    full_arguments.extend_from_slice(&args);

    proc.set_arguments(&full_arguments, envp);

    proc.data.fill_command_line_args(full_arguments);

    Ok(proc)
}