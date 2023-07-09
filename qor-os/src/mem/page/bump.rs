use core::{sync::atomic::Ordering, ops::Range};

use super::{Page, PAGE_SIZE};

/// Kernel space static allocator of page-scale regions of memory which cannot
/// be freed.
#[derive(Debug)]
pub struct KernelPageStaticBumpAllocator {
    pages_walking_pointer: core::sync::atomic::AtomicPtr<Page>,
    pages_end: core::sync::atomic::AtomicPtr<Page>,
    total: core::sync::atomic::AtomicUsize
}

unsafe impl Sync for KernelPageStaticBumpAllocator {}

impl KernelPageStaticBumpAllocator {
    /// Construct a KernelPageStaticBumpAllocator from a pointer range
    /// 
    /// # Safety
    /// The pointer must point to the beginning of a region of memory only
    /// available to this allocator, with a length of `count` pages.
    pub const unsafe fn new(page_range: Range<*const Page>) -> Self {
        Self {
            pages_walking_pointer: core::sync::atomic::AtomicPtr::new(page_range.start as *mut Page),
            pages_end: core::sync::atomic::AtomicPtr::new(page_range.end as *mut Page),
            total: core::sync::atomic::AtomicUsize::new(page_range.end.sub_ptr(page_range.start))
        }
    }

    /// Update the allocator to point to a new range of memory.
    /// 
    /// # Safety
    /// The pointer must point to the beginning of a region of memory only
    /// available to this allocator, with a length of `count` pages.
    pub unsafe fn update(&self, page_range: Range<*const Page>) {
        self.pages_walking_pointer.store(page_range.start as *mut Page, Ordering::Release);
        self.pages_end.store(page_range.end as *mut Page, Ordering::Release);
        self.total.store(page_range.end.sub_ptr(page_range.start), Ordering::Release);
    }

    /// Get the number of free pages.
    pub fn free(&self) -> usize {
        let current = self.pages_walking_pointer.load(Ordering::Acquire).addr();
        let end = self.pages_end.load(Ordering::Acquire).addr();

        if current >= end {
            0
        }
        else {
            (end - current) / PAGE_SIZE
        }
    }

    /// Get the total number of pages assigned to the allocator
    pub fn total(&self) -> usize {
        self.total.load(Ordering::Acquire)
    }

    /// Allocate `count` pages of memory, returning a static, mutable reference
    /// to the region of memory `count` pages long.
    /// 
    /// # Errors
    /// 
    /// If there are not enough pages remaining to allocate `count` pages of
    /// memory, a `KernelPageStaticBumpAllocatorError` is returned.
    pub fn alloc_pages(&self, count: usize) -> Result<&'static mut [Page], KernelPageStaticBumpAllocatorError> {
        // Atomically increment the pointer
        let pointer = self.pages_walking_pointer.fetch_ptr_add(count, Ordering::Release);

        // Check if the resulting region overruns the buffer
        let after_known_buffer = unsafe { pointer.add(count).addr() };

        if after_known_buffer <= self.pages_end.load(Ordering::Acquire).addr() {
            Ok(unsafe { core::slice::from_raw_parts_mut(pointer, count) })
        }
        else {
            self.pages_walking_pointer.fetch_ptr_sub(count, Ordering::Release);
            Err(KernelPageStaticBumpAllocatorError::OutOfMemoryError { requested: count, remaining: self.free(), total: self.total() })
        }
    }
}

/// Potential errors which can be returned when attempting an allocation via a
/// `KernelPageStaticBumpAllocator`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelPageStaticBumpAllocatorError {
    OutOfMemoryError{requested: usize, remaining: usize, total: usize}
}

impl core::fmt::Display for KernelPageStaticBumpAllocatorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            KernelPageStaticBumpAllocatorError::OutOfMemoryError { requested, remaining, total } => {
                write!(f, "KernelPageStaticBumpAllocatorError::OutOfMemoryError - Cannot allocate {} page{} ({}/{}) free", requested, if *requested > 1 { "s" } else { "" }, remaining, total)
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    static FIVE_PAGES: [Page; 5] = [Page([0; PAGE_SIZE]); 5];
    static EIGHT_PAGES: [Page; 8] = [Page([0; PAGE_SIZE]); 8];

    #[test_case]
    fn success() {
        let allocator = unsafe { KernelPageStaticBumpAllocator::new(FIVE_PAGES.as_ptr_range()) };

        assert!(allocator.alloc_pages(4).is_ok());
        assert!(allocator.alloc_pages(1).is_ok());
    }

    #[test_case]
    fn out_of_memory() {
        let allocator = unsafe { KernelPageStaticBumpAllocator::new(FIVE_PAGES.as_ptr_range()) };

        assert_eq!(allocator.alloc_pages(8), Err(KernelPageStaticBumpAllocatorError::OutOfMemoryError{
            requested: 8,
            remaining: 5,
            total: 5 }));
    }

    #[test_case]
    fn many_allocations() {
        let allocator = unsafe { KernelPageStaticBumpAllocator::new(EIGHT_PAGES.as_ptr_range()) };

        assert!(allocator.alloc_pages(4).is_ok());
        assert_eq!(allocator.alloc_pages(8), Err(KernelPageStaticBumpAllocatorError::OutOfMemoryError{
            requested: 8,
            remaining: 4,
            total: 8 }));
        assert!(allocator.alloc_pages(1).is_ok());
        assert_eq!(allocator.alloc_pages(4), Err(KernelPageStaticBumpAllocatorError::OutOfMemoryError{
            requested: 4,
            remaining: 3,
            total: 8 }));
        assert!(allocator.alloc_pages(1).is_ok());
        assert!(allocator.alloc_pages(2).is_ok());
        assert_eq!(allocator.alloc_pages(1), Err(KernelPageStaticBumpAllocatorError::OutOfMemoryError{
            requested: 1,
            remaining: 0,
            total: 8 }));
    }
}

#[cfg(test)]
pub mod sync_test {
    use crate::{mem::{Page, PAGE_SIZE}, harts::{machine_mode_is_primary_hart, machine_mode_sync}, asm::HEAP_START, asm::HEAP_END};
    use crate::harts;

    use super::KernelPageStaticBumpAllocator;

    static FIVE_PAGES: [Page; 5] = [Page([0; PAGE_SIZE]); 5];

    static BUMP_ALLOC: KernelPageStaticBumpAllocator = unsafe { KernelPageStaticBumpAllocator::new(FIVE_PAGES.as_ptr_range()) };
    
    pub fn collective_test() {
        let heap_size: usize = unsafe { (crate::asm::HEAP_END as usize - crate::asm::HEAP_START as usize)/PAGE_SIZE };
        if machine_mode_is_primary_hart() {    
            unsafe { BUMP_ALLOC.update(core::ops::Range { start: HEAP_START, end: HEAP_END }) }
        }

        let count = heap_size - 8;

        if machine_mode_is_primary_hart() {
            machine_mode_sync();
            for _ in 0..count - (harts::CORE_COUNT - 1) * (count / harts::CORE_COUNT) {
                assert!(BUMP_ALLOC.alloc_pages(1).is_ok());
            }
        }
        else {
            machine_mode_sync();
            for _ in 0..count / harts::CORE_COUNT {
                assert!(BUMP_ALLOC.alloc_pages(1).is_ok());
            }
        }

        machine_mode_sync();

        if machine_mode_is_primary_hart() {
            assert_eq!(BUMP_ALLOC.free(), 8);
        }
    }

    pub fn collective_test_wide_pages() {
        let heap_size: usize = unsafe { (crate::asm::HEAP_END as usize - crate::asm::HEAP_START as usize)/PAGE_SIZE };
        if machine_mode_is_primary_hart() {    
            unsafe { BUMP_ALLOC.update(core::ops::Range { start: HEAP_START, end: HEAP_END }) }
        }

        let count = heap_size - 8;

        if machine_mode_is_primary_hart() {
            machine_mode_sync();
            for _ in 0..count - 8 * (harts::CORE_COUNT - 1) * (count / harts::CORE_COUNT / 8) {
                assert!(BUMP_ALLOC.alloc_pages(1).is_ok());
            }
        }
        else {
            machine_mode_sync();
            for _ in 0..count / harts::CORE_COUNT / 8 {
                assert!(BUMP_ALLOC.alloc_pages(8).is_ok());
            }
        }

        machine_mode_sync();

        if machine_mode_is_primary_hart() {
            assert_eq!(BUMP_ALLOC.free(), 8);
        }
    }
}

/// Initialize a `KernelPageStaticBumpAllocator` to point to the heap referenced by the linker script
pub fn initialize_kernel_bump_allocator() {
    use crate::asm::*;

    // Construct a pointer range over the heap
    let start = unsafe { HEAP_START };
    let end = unsafe { HEAP_START };
    let range = core::ops::Range { start, end};

    // Initialize the allocator
    unsafe { super::BUMP_ALLOC.update(range); }
}