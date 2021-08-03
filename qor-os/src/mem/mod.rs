//! Memory Allocation Handling

use crate::*;
// Includes
pub mod alloc;
pub mod lds;
pub mod mmu;
pub mod page;

// Tests
#[cfg(test)]
pub mod test;

// Page size for global use
pub use page::PAGE_SIZE;

// Global Kernel Page Allocator
static mut GLOBAL_KERNEL_PAGE_ALLOCATOR: *mut page::PageMap = 0 as *mut page::PageMap;

/// Initialize the kernel page allocator
pub fn init_kernel_page_allocator()
{
    unsafe { GLOBAL_KERNEL_PAGE_ALLOCATOR = page::PageMap::initialize(lds::heap_start(), (lds::heap_end() - lds::heap_start()) / PAGE_SIZE) };
}

/// Allocate consecutive pages from the kernel
pub fn kpalloc(count: usize, reason: &'static str) -> Result<usize, page::KernelPageAllocationError>
{
    kdebug!(MemoryAllocation, "`{}`: kpalloc({}) -> ", reason, count);

    // Ensure the global kernel page allocator was initialized
    if unsafe { GLOBAL_KERNEL_PAGE_ALLOCATOR.is_null() }
    {
        panic!("Cannot use kpalloc before the global kernel page allocator is initialized");
    }
    
    // Safety: The above ensured it was initialized, and the only method of
    // initialization is through the proper initializer
    // Panic Safety: This is safe because a null would have been caught
    // above
    let result = unsafe { GLOBAL_KERNEL_PAGE_ALLOCATOR.as_mut() }.unwrap().alloc_pages(count);
    
    if let Ok(ptr) = result
    {
        kdebugln!(MemoryAllocation, "0x{:x}", ptr);
    }
    else
    {
        kdebugln!(MemoryAllocation, "{:?}", result);
    }

    result
}

/// Allocate consecutive pages from the kernel to zero
pub fn kpzalloc(count: usize, reason: &'static str) -> Result<usize, page::KernelPageAllocationError>
{
    // Allocate the pages
    let ptr = kpalloc(count, reason)? as *mut [u8; 4096];

    // Write zeros to each page
    for i in 0..count
    {
        // Safety: The kernel will throw an error if it cannot find valid memory
        unsafe 
        {
            ptr.add(i).write([0; PAGE_SIZE]);
        }
    }

    Ok(ptr as usize)
}

/// Free consecutive pages from the kernel
pub fn kpfree(addr: usize, count: usize) -> Result<(), page::KernelPageAllocationError>
{
    kdebugln!(MemoryAllocation, "kpfree(0x{:x}, {})", addr, count);

    
    // Ensure the global kernel page allocator was initialized
    if unsafe { GLOBAL_KERNEL_PAGE_ALLOCATOR.is_null() }
    {
        panic!("Cannot use kpalloc before the global kernel page allocator is initialized");
    }
    
    // Safety: The above ensured it was initialized, and the only method of
    // initialization is through the proper initializer
    unsafe
    {
        // Panic Safety: This is safe because a null would have been caught
        // above
        GLOBAL_KERNEL_PAGE_ALLOCATOR.as_mut().unwrap().free_pages(addr, count)
    }
}

/// Get the number of allocated pages on the kernel heap
pub fn allocated_kernel_pages() -> usize
{
    // Ensure the global kernel page allocator was initialized
    if unsafe { GLOBAL_KERNEL_PAGE_ALLOCATOR.is_null() }
    {
        panic!("Cannot get the number of allocated kernel pages because the allocator is not initialized");
    }
    
    // Safety: The above ensured it was initialized, and the only method of
    // initialization is through the proper initializer
    unsafe
    {
        // Panic Safety: This is safe because a null would have been caught
        // above
        GLOBAL_KERNEL_PAGE_ALLOCATOR.as_ref().unwrap().total_alloc_pages()
    }
}

/// Get the number of allocated pages on the kernel heap
pub fn total_kernel_pages() -> usize
{
    // Ensure the global kernel page allocator was initialized
    if unsafe { GLOBAL_KERNEL_PAGE_ALLOCATOR.is_null() }
    {
        panic!("Cannot get the total number of kernel pages because the allocator is not initialized");
    }
    
    // Safety: The above ensured it was initialized, and the only method of
    // initialization is through the proper initializer
    unsafe
    {
        // Panic Safety: This is safe because a null would have been caught
        // above
        GLOBAL_KERNEL_PAGE_ALLOCATOR.as_ref().unwrap().total_pages()
    }
}

/// Identity map the kernel
pub fn identity_map_kernel()
{
    use mmu::PageTableEntryFlags;

    // Allocate a new page table
    let page_table = mmu::PageTable::allocate();

    // Identity map the segments from the linker script
    page_table.identity_map(lds::text_start(), lds::text_end(), PageTableEntryFlags::readable() | PageTableEntryFlags::executable());
    page_table.identity_map(lds::rodata_start(), lds::rodata_end(), PageTableEntryFlags::readable() | PageTableEntryFlags::executable());
    page_table.identity_map(lds::data_start(), lds::data_end(), PageTableEntryFlags::readable() | PageTableEntryFlags::writable());
    page_table.identity_map(lds::bss_start(), lds::bss_end(), PageTableEntryFlags::readable() | PageTableEntryFlags::writable());
    page_table.identity_map(lds::stack_start(), lds::stack_end(), PageTableEntryFlags::readable() | PageTableEntryFlags::writable());
    page_table.identity_map(lds::heap_start(), lds::heap_end(), PageTableEntryFlags::readable() | PageTableEntryFlags::writable());

    // Map the RTC MMIO
    page_table.identity_map(0x10_1000, 0x10_1fff, PageTableEntryFlags::readable() | PageTableEntryFlags::writable());

    // Map the CLINT MMIO
    page_table.identity_map(0x200_0000, 0x200_b000, PageTableEntryFlags::readable() | PageTableEntryFlags::writable());

    // Map the PLIC MMIO
    page_table.identity_map(0xc00_0000, 0xc00_2000, PageTableEntryFlags::readable() | PageTableEntryFlags::writable());
    page_table.identity_map(0xc20_0000, 0xc20_0000, PageTableEntryFlags::readable() | PageTableEntryFlags::writable());

    // Map the UART MMIO
    page_table.identity_map(0x1000_0000, 0x1000_0000, PageTableEntryFlags::readable() | PageTableEntryFlags::writable());

    // Map the VirtIO MMIO
    page_table.identity_map(0x1000_1000, 0x1000_8000, PageTableEntryFlags::readable() | PageTableEntryFlags::writable());

    // Write the current page to SATP
    let root_ppn = page_table as *mut mmu::PageTable as usize >> 12;
    let satp_val = 8 << 60 | root_ppn;
    
    riscv::register::satp::write(satp_val);
}