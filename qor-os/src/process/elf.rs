//! ELF File Loader

use crate::*;

use fs::fstrait::Filesystem;

use alloc::vec::Vec;
use libutils::paths::PathBuffer;

use super::process::Process;

/// Elf Loading Error
#[derive(Debug, Clone)]
pub enum ElfLoadError
{
    ReadError(fs::structures::FilesystemError),
    NotAnELF,
    BadFormat(String)
}

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
}


/// Load a file from a file interface and convert it to a process
pub fn load_elf(interface: &mut fs::vfs::FilesystemInterface, path: PathBuffer, args: Vec<String>) -> Result<Process, ElfLoadError>
{
    kdebugln!(Elf, "Loading ELF File `{}`", path);

    // Open the file
    let index = interface.path_to_inode(path).map_err(|e| ElfLoadError::ReadError(e))?;
    let file_data = interface.read_inode(index).map_err(|e| ElfLoadError::ReadError(e))?;

    // Verify it is an elf file
    if file_data[0..4] != [0x7F, 'E' as u8, 'L' as u8, 'F' as u8]
    {
        return Err(ElfLoadError::NotAnELF)
    }

    // Get an instance of the Elf Header
    let elf_header = unsafe { (file_data.as_ptr() as *const ElfHeader).read() };

    // Verify the elf is a 64 bit elf file
    if elf_header.ident_class != 2
    {
        return Err(ElfLoadError::BadFormat(String::from("ELF File is a 32-bit ELF File")));
    }

    // Verify the elf is a risc-v elf file
    if elf_header.e_machine != 0xF3
    {
        return Err(ElfLoadError::BadFormat(String::from("ELF File is not a RISCV ELF File")));
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

        segments.push(
            Segment
            {
                vaddr: header.vaddr,
                flags,
                msize: header.memsz,
                fsize: header.filesz,
                f_offset: header.off
            }
        )
    }

    // Allocate a new page table
    let table = unsafe { (mem::kpzalloc(1, "ELF Page Table").unwrap() as *mut mem::mmu::PageTable).as_mut().unwrap() };

    // Map the segments
    for segment in segments
    {
        let poff = segment.f_offset & (mem::PAGE_SIZE - 1);

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

    // Allocate space for four pages of stack space
    let stack_space = mem::kpzalloc(4, "ELF Stack Space").unwrap();

    // Map the stack space
    for i in 0..4
    {
        table.map(0x2_0000_0000 + mem::PAGE_SIZE * i,
            stack_space + mem::PAGE_SIZE * i,
            mem::mmu::PageTableEntryFlags::user() | mem::mmu::PageTableEntryFlags::readable() | mem::mmu::PageTableEntryFlags::executable() | mem::mmu::PageTableEntryFlags::writable() | mem::mmu::PageTableEntryFlags::dirty() | mem::mmu::PageTableEntryFlags::accessed(),
            0);
    }

    let mut proc = Process::from_components(
        elf_header.e_entry as usize, 
        table as *mut mem::mmu::PageTable, 
        4, 0x2_0000_0000);

    let mut full_arguments = vec![path.as_str().to_string()];

    full_arguments.extend_from_slice(&args);
    let mut raw_args = Vec::new();

    for arg in &mut full_arguments
    {
        arg.push('\0');

        raw_args.push(arg.as_bytes());
    }

    proc.set_arguments(&raw_args, &[]);

    proc.data.fill_command_line_args(full_arguments);

    Ok(proc)
}