use crate::*;

use super::heap::{kzalloc, kfree};
use super::pagetable::{Table, Entry, EntryBits};
use super::pages::PAGE_SIZE;

use core::{ptr::null_mut, sync::atomic::{AtomicPtr, AtomicBool}};

// Global page table pointer
static GLOBAL_PAGE_TABLE_POINTER: AtomicPtr<Table> = AtomicPtr::new(null_mut());

// Flag set to true iff the page table has been initialized
static PAGE_TABLE_INITIALIZED: AtomicBool = AtomicBool::new(false);

// TODO: This is not the safest way to go about this, after all, this does allow
// multi thread access, I will fix this when I find a better lock
/// Get a mutable reference to the global page table
fn global_page_table() -> &'static mut Table
{
    if !PAGE_TABLE_INITIALIZED.load(core::sync::atomic::Ordering::Relaxed)
    {
        panic!("Global page table has not yet been initialized");
    }

    // Safety: This is safe because we check the PAGE_TABLE_INITIALIZED flag
    unsafe { GLOBAL_PAGE_TABLE_POINTER.load(core::sync::atomic::Ordering::SeqCst).as_mut().unwrap() }
}

/// Allocate a 512 entry page table
fn alloc_table() -> &'static mut Table
{
    let table_addr = kzalloc(1) as usize;
    let page = table_addr / PAGE_SIZE;

    // Safety: Because the page is allocated via the kalloc
    unsafe { Table::new(page) }
}

/// Initialize the global page table
pub fn init_global_page_table()
{
    let table = alloc_table();

    let ptr = table as *mut Table;

    GLOBAL_PAGE_TABLE_POINTER.store(ptr, core::sync::atomic::Ordering::SeqCst);
    PAGE_TABLE_INITIALIZED.store(true, core::sync::atomic::Ordering::SeqCst);

    kprintln!("Global Page Table Initialized");
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

// TODO: Make sure all of the uses of the settings in these functions are the
// right number of bits

/// Map a virtual address to a physical address
fn inner_map(root: &mut Table, virt_addr: usize, phys_addr: usize, settings: usize, level: MMUPageLevel)
{
    kdebugln!(MemoryMapping, "Mapping 0x{:x} -> 0x{:x} settings: 0b{:0b}, level: {:?}", virt_addr, phys_addr, settings, level);

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
fn inner_unmap(root: &mut Table, virt_addr: usize, level: MMUPageLevel)
{
    kdebugln!(MemoryMapping, "Unmapping virtual address 0x{:x} with level {:?}", virt_addr, level);

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
fn inner_virt_to_phys(root: &Table, virt_addr: usize) -> Option<usize>
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

/// Allocate some number of pages in virtual memory
fn inner_kvalloc(root: &mut Table, virt_addr: usize, num_pages: usize, settings: usize)
{
    kdebugln!(PageMapping, "Allocating {} pages of virtual memory at 0x{:x} with settings 0b{:b}", num_pages, virt_addr, settings);

    // Allocate the space
    let ptr = kzalloc(num_pages);

    // Convert pointers to usizes
    let mut virt_addr_usize = virt_addr & !(4096 - 1);
    let mut phys_addr_usize = ptr as usize & !(4096 - 1);

    // Map all of the pages
    for _ in 0..num_pages
    {
        inner_map(root, virt_addr_usize, phys_addr_usize, settings, MMUPageLevel::Level4KiB);
    
        virt_addr_usize += PAGE_SIZE;
        phys_addr_usize += PAGE_SIZE;
    }
}

/// Free some number of virtual memory pages
fn inner_kvfree(root: &mut Table, virt_addr: usize, num_pages: usize)
{
    kdebugln!(PageMapping, "Freeing {} pages of virtual memory at 0x{:x}", num_pages, virt_addr);

    let mut virt_addr_usize = virt_addr & !(4096 - 1);

    for _ in 0..num_pages
    {
        let phys = inner_virt_to_phys(root, virt_addr_usize);

        if phys.is_none()
        {
            panic!("Attempting to free unmapped virtual memory 0x{:x}", virt_addr_usize & !(4096 - 1));
        }

        inner_unmap(root, virt_addr_usize, MMUPageLevel::Level4KiB);

        // Safety: Because this was found from the memory map and the only way
        // to get an entry in the memory map is to allocate space for it, and
        // the only way to remove an entry would cause the check above to fail,
        // this is safe
        unsafe { kfree(phys.unwrap() as *mut u8, 1) };

        virt_addr_usize += PAGE_SIZE;
    }
}

/// Identity map some range of addresses
fn inner_idmap(root: &mut Table, addr_start: usize, addr_end: usize, settings: usize)
{
    // Align to page boundaries
    let mut index = addr_start & !PAGE_SIZE;
    let aligned_end = addr_end & !PAGE_SIZE;

    // Map all of the pages
    while index <= aligned_end
    {
        inner_map(root, index, index, settings, MMUPageLevel::Level4KiB);

        index += PAGE_SIZE;
    }
}

// =============================================================================
// Public Interfaces for MMU functions
// =============================================================================

/// Map a virtual address to a physical address
pub fn map(virt_addr: usize, phys_addr: usize, settings: usize, level: MMUPageLevel)
{
    inner_map(global_page_table(), virt_addr, phys_addr, settings, level)
}

/// Unmap a virtual address
pub fn unmap(virt_addr: usize, level: MMUPageLevel)
{
    inner_unmap(global_page_table(), virt_addr, level)
}

/// Map a virtual address to a physical address (So user space programs can
/// share a pointer to a kernel space program)
pub fn virt_to_phys(virt_addr: usize) -> Option<usize>
{
    inner_virt_to_phys(global_page_table(), virt_addr)
}

/// Allocate some number of pages in virtual memory
pub fn kvalloc(virt_addr: usize, num_pages: usize, settings: usize)
{
    inner_kvalloc(global_page_table(), virt_addr, num_pages, settings)
}

/// Free some number of virtual memory pages
pub fn kvfree(virt_addr: usize, num_pages: usize)
{
    inner_kvfree(global_page_table(), virt_addr, num_pages)
}

/// Identity map some range of addresses
pub fn idmap(addr_start: usize, addr_end: usize, settings: usize)
{
    inner_idmap(global_page_table(), addr_start, addr_end, settings)
}
