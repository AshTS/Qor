use crate::{*, process::init};

use alloc::boxed::Box;
use alloc::vec::Vec;

/// Test Kernel Page Grained Allocator - Allocate and free 4096 pages
#[test_case]
pub fn test_kernel_page_allocator_allocate_all()
{
    let initial_pages = super::allocated_kernel_pages();

    // Pages to test
    let page_count = 4096;

    // The first address in the allocator
    let first = super::kpalloc(1, "Test").unwrap();

    // Allocate every page
    for _ in 0..(page_count - 1)
    {
        super::kpalloc(1, "Test").unwrap();
    }

    // Free every page
    for i in 0..page_count
    {
        super::kpfree(first + super::PAGE_SIZE * i, 1).unwrap();
    }

    // Assert that all of the pages are free
    assert_eq!(super::allocated_kernel_pages(), initial_pages);
}

/// Test Kernel Page Grained Allocator - Ensure Zero Alloc
#[test_case]
pub fn test_kernel_page_allocator_zalloc()
{
    let initial_pages = super::allocated_kernel_pages();

    // Pages to test
    let page_count = 4096;

    // The first address in the allocator
    let first = super::kpzalloc(1, "Test").unwrap();

    // Ensure the first page is zero allocated
    if unsafe { (first as *mut [u8; super::PAGE_SIZE]).read() } != [0; super::PAGE_SIZE]
    {
        panic!("Page 0x{:x} is not zero initialized", first);
    }

    // Allocate every page
    for _ in 0..(page_count - 1)
    {
        let ptr = super::kpzalloc(1, "Test").unwrap();

        // Ensure the pages are zero allocated
        if unsafe { (ptr as *mut [u8; super::PAGE_SIZE]).read() } != [0; super::PAGE_SIZE]
        {
            panic!("Page 0x{:x} is not zero initialized", ptr);
        }
    }

    // Free every page
    for i in 0..page_count
    {
        super::kpfree(first + super::PAGE_SIZE * i, 1).unwrap();
    }

    // Assert that all of the pages are free
    assert_eq!(super::allocated_kernel_pages(), initial_pages);
}

/// Test Kernel Page Grained Allocator - Ensure Unique Allocations
#[test_case]
pub fn test_kernel_page_allocator_no_overwrite()
{
    let initial_pages = super::allocated_kernel_pages();

    // Pages to test
    let page_count = 256;

    // The first address in the allocator
    let first = super::kpzalloc(1, "Test").unwrap();

    // Allocate every page
    for _ in 0..(page_count - 1)
    {
        super::kpzalloc(1, "Test").unwrap();
    }

    // Go over every page
    for i in 0..page_count
    {
        let this_ptr = first + i * super::PAGE_SIZE;

        // Overwrite this page with 0xFF
        unsafe { (this_ptr as *mut [u8; super::PAGE_SIZE]).write([0xFF; super::PAGE_SIZE]) }

        // Loop over all other pages
        for j in 0..page_count
        {
            // Skip this page
            if j == i {continue;}

            let ptr = first + j * super::PAGE_SIZE;

            // Ensure the other pages have not been overwritten
            if unsafe { (ptr as *mut [u8; super::PAGE_SIZE]).read() } != [0; super::PAGE_SIZE]
            {
                panic!("Page 0x{:x} is not zero initialized", ptr);
            }
        }

        // Correct the page by writing it with zeros again
        unsafe { (this_ptr as *mut [u8; super::PAGE_SIZE]).write([0; super::PAGE_SIZE]) }
    }

    // Free every page
    for i in 0..page_count
    {
        super::kpfree(first + super::PAGE_SIZE * i, 1).unwrap();
    }

    // Assert that all of the pages are free
    assert_eq!(super::allocated_kernel_pages(), initial_pages);
}

/// Test Kernel Byte Grained Allocator - Test Simple Allocation
#[test_case]
pub fn test_kernel_byte_allocator_simple()
{
    // Initialize a small global allocator
    super::alloc::init_kernel_global_allocator(2);

    // Attempt to allocate a box
    let b = Box::leak(Box::new(42usize)) as *mut usize;

    unsafe { b.write_volatile(24); }

    // Attempt to free the box
    unsafe { Box::from_raw(b) };
}

/// Test Kernel Byte Grained Allocator - Test Multiple Allocation
#[test_case]
pub fn test_kernel_byte_allocator_multiple()
{
    // Initialize a small global allocator
    super::alloc::init_kernel_global_allocator(2);

    let mut v = Vec::new();

    for _ in 0..16
    {
        v.push(Box::leak(Box::new(42usize)) as *mut usize);
    }

    for ptr in &v
    {
        unsafe { ptr.write(5); }
    }

    for ptr in v
    {
        unsafe { Box::from_raw(ptr) };
    }
}