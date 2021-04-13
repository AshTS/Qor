use crate::*;

use super::{addrs, pagetable::Table};

/// Identity map the kernel in virtual address space
pub fn identity_map_kernel()
{
    kdebugln!(KernelVirtMapping, "Mapping Kernel");    

    kdebugln!(KernelVirtMapping, "TEXT:    0x{:x} - 0x{:x}", addrs::text_start(), addrs::text_end());
    kdebugln!(KernelVirtMapping, "RODATA:  0x{:x} - 0x{:x}", addrs::rodata_start(), addrs::rodata_end());
    kdebugln!(KernelVirtMapping, "DATA:    0x{:x} - 0x{:x}", addrs::data_start(), addrs::data_end());
    kdebugln!(KernelVirtMapping, "BSS:     0x{:x} - 0x{:x}", addrs::bss_start(), addrs::bss_end());
    kdebugln!(KernelVirtMapping, "STACK:   0x{:x} - 0x{:x}", addrs::stack_start(), addrs::stack_end());
    kdebugln!(KernelVirtMapping, "HEAP:    0x{:x} - 0x{:x}", addrs::heap_start(), addrs::heap_end());

    super::mmu::idmap(addrs::text_start(), addrs::text_end(), super::EntryBits::ReadExecute);
    super::mmu::idmap(addrs::rodata_start(), addrs::rodata_end(), super::EntryBits::ReadExecute);
    super::mmu::idmap(addrs::data_start(), addrs::data_end(), super::EntryBits::ReadWrite);
    super::mmu::idmap(addrs::bss_start(), addrs::bss_end(), super::EntryBits::ReadWrite);
    super::mmu::idmap(addrs::stack_start(), addrs::stack_end(), super::EntryBits::ReadWrite);
    super::mmu::idmap(addrs::heap_start(), addrs::heap_end(), super::EntryBits::ReadWrite);

    // Map the CLINT MMIO
    super::mmu::idmap(0x200_0000, 0x200_b000, super::EntryBits::ReadWrite);

    // Map the UART MMIO
    super::mmu::idmap(0x1000_0000, 0x1000_0000, super::EntryBits::ReadWrite);

    kprintln!("Kernel Mapped");
}

/// Set the MMU to point to the GPT
pub fn init_mmu()
{
    let root_ppn = super::mmu::global_page_table() as *mut Table as usize >> 12;
    let satp_val = 8 << 60 | root_ppn;
    
    riscv::register::satp::write(satp_val);
}