use crate::*;

use super::heap::kzalloc;
use super::pagetable::{Table, Entry, EntryBits};
use super::pages::PAGE_SIZE;

/// Allocate a 512 entry page table
pub fn alloc_table() -> &'static mut Table
{
    let table_addr = kzalloc(1) as usize;
    let page = table_addr / PAGE_SIZE;

    // Safety: Because the page is allocated via the kalloc
    unsafe { Table::new(page) }
}

#[repr(usize)]
#[derive(Debug, Clone, Copy)]
/// Levels for the MMU Mapper (4KiB, 2MiB, 1GiB)
pub enum MMUPageLevel
{
    Level4KiB = 0,
    Level2MiB = 1,
    Level1GiB = 2
}

pub fn map(root: &mut Table, virt_addr: usize, phys_addr: usize, settings: usize, level: MMUPageLevel)
{
    kdebugln!("Mapping 0x{:x} -> 0x{:x} settings: 0b{:0b}, level: {:?}", virt_addr, phys_addr, settings, level);

    let level = level as usize;

    if settings & 0xe == 0
    {
        panic!("Cannot map with none of RWX set (0x{:x} -> 0x{:x})", virt_addr, phys_addr);
    }

    let vpn = [
        (virt_addr >> 12) & ((1 << 9) - 1),
        (virt_addr >> 21) & ((1 << 9) - 1),
        (virt_addr >> 30) & ((1 << 9) - 1)
    ];


    let ppn = [
        (phys_addr >> 12) & ((1 << 9) - 1),
        (phys_addr >> 21) & ((1 << 9) - 1),
        (phys_addr >> 30) & ((1 << 26) - 1)
    ];

    let mut walking = &mut root[vpn[2]];

    for current_level in (level..2).rev()
    {
        if !walking.is_valid()
        {
            let page_addr = super::heap::kzalloc(1);

            walking.set_data(
                (page_addr as usize >> 2) as u64 | EntryBits::Valid as u64,
            );
        }

        let entry = ((walking.get_data() & !0x3ff) << 2) as *mut Entry;
	    walking = unsafe { entry.add(vpn[current_level]).as_mut().unwrap() };
    }

    let entry = (ppn[2] << 28) as usize |
		        	(ppn[1] << 19) as usize | 
		        	(ppn[0] << 10) as usize | 
		        	settings |                    
		        	EntryBits::Valid as usize;

    walking.set_data(entry as u64);
}

pub fn unmap()
{

}

/// Map a virtual address to a physical address (So user space programs can
/// share a pointer to a kernel space program)
pub fn virt_to_phys(root: &Table, virt_addr: usize) -> Option<usize>
{
    let vpn = [
        (virt_addr >> 12) & ((1 << 9) - 1),
        (virt_addr >> 21) & ((1 << 9) - 1),
        (virt_addr >> 30) & ((1 << 9) - 1)
    ];

    let mut ptr = &root[vpn[2]];

    for i in (0..=2).rev()
    {
        if !ptr.is_valid()
        {
            // The MMU would page fault here
            break;
        }
        else if ptr.is_leaf()
        {
            let ppn = ptr.get_ppn();

            let offset_mask =  (1 << (12 + 9 * i)) - 1;
            let physical_mask = !((1 << (9 * i)) - 1);
            
            let offset = virt_addr &offset_mask;
            let phys_offset = ppn & physical_mask;

            return Some(offset | (phys_offset << (12 + 9 * i)));
        }
        else
        {
            let entry = ((ptr.get_data() & !0x3ff) << 2) as *const Entry;
            // Safety: Assuming the Page Tables are properly initialized, this
            // will be safe
            ptr = unsafe { entry.add(vpn[i - 1]).as_ref().unwrap() };
        }
    }

    None
}