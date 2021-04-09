use crate::*;

use super::heap::{kzalloc, kfree};
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

/// Unmap a virtual address
pub fn unmap(root: &mut Table, virt_addr: usize, level: MMUPageLevel)
{
    kdebugln!("Unmapping virtual address 0x{:x} with level {:?}", virt_addr, level);

    let vpn = [
        (virt_addr >> 12) & ((1 << 9) - 1),
        (virt_addr >> 21) & ((1 << 9) - 1),
        (virt_addr >> 30) & ((1 << 9) - 1)
    ];

    let mut last_addr = None;
    let mut ptr = &mut root[vpn[2]];
    for i in (level as usize..=2).rev()
    {
        if !ptr.is_valid()
        {
            // If we hit an invalid before we get to the bottom, we must have
            // tried to unmap an already unmapped address, we will panic here
            // because it means the memory manager for the kernel messed up some
            // how.
            panic!("Cannot unmap virtual address 0x{:x} level {:?}, hit an invalid entry at level {}", virt_addr, level, i);
        }
        else if ptr.is_leaf()
        {
            // Invalidate the leaf and return
            ptr.set_bit(EntryBits::Valid, false);
            return;
        }
        else
        {
            let entry = ((ptr.get_data() & !0x3ff) << 2) as *mut Entry;
            // Safety: Assuming the Page Tables are properly initialized, this
            // will be safe
            ptr = unsafe { entry.add(vpn[i - 1]).as_mut().unwrap() };
            last_addr = Some( (entry as usize >> 12) << 12 );
        }
    }

    // Unmap the table
    // Safety: Because this address was retrieved from the page table, it should point to valid memory (hopefully)
    unsafe { unmap_table(last_addr.unwrap() as *mut Table ) }

    // Invalidate the last node
    ptr.set_bit(EntryBits::Valid, false);
}

/// Unmap a table and all valid paths below it
/// Safety: As long as the pointer is a valid pointer to a page table, this
/// will be safe
unsafe fn unmap_table(root: *mut Table)
{
    // Safety: The function would be unsafe if this call were with a null
    // pointer, making this possible panic redundant
    let root_ref = root.as_mut().unwrap();

    for i in 0..512
    {
        let entry = &mut root_ref[i];

        if entry.is_valid() && !entry.is_leaf()
        {
            unmap_table((entry.get_ppn() << 12) as *mut Table);
        }
    }

    // Safety: Same as the function
    kfree(root as *mut u8, 1);
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