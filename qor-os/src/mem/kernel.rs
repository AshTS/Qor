use crate::*;

use super::addrs;

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

    super::mmu::idmap(addrs::text_start(), addrs::text_end(), super::EntryBits::ReadExecute as usize);
    super::mmu::idmap(addrs::rodata_start(), addrs::rodata_end(), super::EntryBits::ReadExecute as usize);
    super::mmu::idmap(addrs::data_start(), addrs::data_end(), super::EntryBits::ReadWrite as usize);
    super::mmu::idmap(addrs::bss_start(), addrs::bss_end(), super::EntryBits::ReadWrite as usize);
    super::mmu::idmap(addrs::stack_start(), addrs::stack_end(), super::EntryBits::ReadWrite as usize);
    super::mmu::idmap(addrs::heap_start(), addrs::heap_end(), super::EntryBits::ReadWrite as usize);

    kprintln!("Kernel Mapped");
}