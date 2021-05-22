//! Memory Allocation Handling

// Includes
pub mod lds;
pub mod page;

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
pub fn kpalloc(count: usize) -> Result<usize, page::KernelPageAllocationError>
{
    // Ensure the global kernel page allocator was initialized
    if unsafe { GLOBAL_KERNEL_PAGE_ALLOCATOR.is_null() }
    {
        panic!("Cannot use kpalloc before the global kerbnel page allocator is initialized");
    }
    
    // Safety: The above ensured it was initialized, and the only method of
    // initialization is through the proper initializer
    unsafe
    {
        // Panic Safety: This is safe because a null would have been caught
        // above
        GLOBAL_KERNEL_PAGE_ALLOCATOR.as_mut().unwrap().alloc_pages(count)
    }
}

/// Allocate consecutive pages from the kernel
pub fn kpzalloc(count: usize) -> Result<usize, page::KernelPageAllocationError>
{
    let ptr = kpalloc(count)? as *mut u128;

    for i in 0..count * PAGE_SIZE / core::mem::size_of::<u128>()
    {
        // Safety: kpalloc will return only a valid pointer or an error
        unsafe { ptr.add(i).write(0) };
    }

    Ok(ptr as usize)
}

/// Free consecutive pages from the kernel
pub fn kpfree(addr: usize, count: usize) -> Result<(), page::KernelPageAllocationError>
{
    // Ensure the global kernel page allocator was initialized
    if unsafe { GLOBAL_KERNEL_PAGE_ALLOCATOR.is_null() }
    {
        panic!("Cannot use kpalloc before the global kerbnel page allocator is initialized");
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