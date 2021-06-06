use crate::*;

// Mirror the definitions in syscalls.h
/*
    #define PROT_READ 1
    #define PROT_WRITE 2
    #define PROT_EXEC 4
*/

const PROT_READ: usize = 1;
const PROT_WRITE: usize = 2;
const PROT_EXEC: usize = 4;

/// mmap Syscall
pub fn syscall_mmap(proc: &mut super::Process, _start_ptr: usize, length: usize, prot: usize, _flags: usize, _fd: usize, _offset: usize) -> usize
{
    let mut mem_flags = mem::mmu::PageTableEntryFlags::user();

    if prot & PROT_EXEC > 0
    {
        mem_flags = mem_flags | mem::mmu::PageTableEntryFlags::executable() | mem::mmu::PageTableEntryFlags::accessed();
    }

    if prot & PROT_WRITE > 0
    {
        mem_flags = mem_flags | mem::mmu::PageTableEntryFlags::writable() | mem::mmu::PageTableEntryFlags::dirty();
    }

    if prot & PROT_READ > 0
    {
        mem_flags = mem_flags | mem::mmu::PageTableEntryFlags::readable() | mem::mmu::PageTableEntryFlags::accessed();
    }

    proc.map(length, mem_flags)
}