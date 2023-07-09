use core::{ops::Range, sync::atomic::AtomicU64};

use super::{Page, PAGE_SIZE};

pub struct KernelPageBitmapAllocator {
    bitmap: atomic::Atomic<&'static [core::sync::atomic::AtomicU64]>,
    start_addr: core::sync::atomic::AtomicPtr<Page>,
    count: core::sync::atomic::AtomicUsize
}

impl KernelPageBitmapAllocator {
    /// Construct a new allocator which points to a null region of memory.
    pub const fn new() -> Self {
        Self {
            bitmap: atomic::Atomic::new(&[]),
            start_addr: core::sync::atomic::AtomicPtr::new(core::ptr::null_mut()),
            count: core::sync::atomic::AtomicUsize::new(0)
        }
    }

    /// Initialize a `KernelPageBitmapAllocator` to point to a new region of memory.
    /// 
    /// # Safety
    /// The page range given must point to a valid, unused region of memory.
    /// Additionally, all previously mapped regions are no longer able to be
    /// freed.
    pub unsafe fn initialize(&self, page_range: Range<*const Page>) {
        // Get the number of pages in the range
        let page_count = page_range.end.sub_ptr(page_range.start);

        // Get the size of the bitmap
        let bitmap_page_count = (page_count + 8 * PAGE_SIZE - 1) / (8 * PAGE_SIZE);
        let remaining_page_count = page_count - bitmap_page_count;

        // Get the starting address of the allocation area
        let allocation_start_address = page_range.start.add(bitmap_page_count) as *mut Page;

        // Construct the bitmap
        let bitmap: &[AtomicU64]  = 
            core::mem::transmute(core::slice::from_raw_parts(page_range.start, remaining_page_count));

        // Fill the bitmap with zeroes to denote free pages
        for bitmap_entry in bitmap {
            bitmap_entry.store(0, core::sync::atomic::Ordering::Relaxed);
        }

        // Update the allocator
        self.bitmap.store(bitmap, atomic::Ordering::Release);
        self.start_addr.store(allocation_start_address, core::sync::atomic::Ordering::Release);
        self.count.store(page_count - bitmap_page_count, core::sync::atomic::Ordering::Release);

        core::sync::atomic::fence(core::sync::atomic::Ordering::Acquire);
    }

    /// Initialize a `KernelPageBitmapAllocator` to point to a new region of
    /// memory given by a mutable reference to a static buffer.
    pub fn initialize_with_buffer(&self, buffer: &'static mut [Page]) {
        unsafe { self.initialize(buffer.as_ptr_range()) };
    }

    /// Construct the mask for the given number of pages at the given index
    /// (the index is required so the mask is clipped at the right number of
    /// bits to stay within a bitmap entry).
    fn make_mask_for(index: usize, page_count: usize) -> Option<u64> {
        if page_count < 64 {
            Some(((0b1 << page_count) - 1) << (index % 64))
        }
        else {
            None
        }
    }

    /// Allocates the pages within a single bitmap entry, returning true if the
    /// allocation was successful, if the allocation was not successful, the
    /// attempted allocation is undone.
    fn mark_as_allocated_within_one(&self, index: usize, page_count: usize) -> bool {
        if page_count > 64 { return false; }
        if index + page_count > self.count.load(core::sync::atomic::Ordering::Relaxed) {
            return false;
        }

        // Produce the mask to use to update the bitmap entry
        if let Some(mask) = Self::make_mask_for(index, page_count) {
            let read = self.bitmap.load(atomic::Ordering::Relaxed)[index / 64].fetch_or(mask, core::sync::atomic::Ordering::AcqRel);

            // If the allocation failed, and someone got to that page first
            if read & mask == 0 {
                true
            }
            else {
                // Then we modify the mask to reflect the bits that couldn't be allocated
                let mask_take_away_failed = mask ^ (read & mask);

                // Then we clear those bits
                self.bitmap.load(atomic::Ordering::Relaxed)[index / 64].fetch_and(!mask_take_away_failed, core::sync::atomic::Ordering::Relaxed);
                
                false
            }
        }
        else {
            false
        }
    }

    /// Frees the pages within a single bitmap entry, returning true if
    /// successful.
    fn mark_as_free_within_one(&self, index: usize, page_count: usize) -> bool {
        if page_count > 64 { return false; }
        if index + page_count > self.count.load(core::sync::atomic::Ordering::Relaxed) {
            return false;
        }

        // Produce the mask to use to update the bitmap entry
        if let Some(mask) = Self::make_mask_for(index, page_count) {
            self.bitmap.load(atomic::Ordering::Relaxed)[index / 64].fetch_and(!mask, core::sync::atomic::Ordering::AcqRel);
            true
        }
        else {
            false
        }
    }

    /// Allocates the pages in the bitmap starting at the given index,
    /// returning a boolean if the allocation was successful.
    fn mark_pages_as_allocated(&self, index: usize, page_count: usize) -> bool {
        if index % 64 + page_count < 64 {
            self.mark_as_allocated_within_one(index, page_count)
        }
        else {
            todo!()
        }
    }

    /// Frees the pages in the bitmap starting at the given index, returning a
    /// boolean if it successfully freed all of the pages.
    fn mark_pages_as_free(&self, index: usize, page_count: usize) -> bool {
        if index % 64 + page_count < 64 {
            self.mark_as_free_within_one(index, page_count)
        }
        else {
            todo!()
        }
    }

    /// Get the pointer corresponding to the given index
    /// 
    /// # Safety
    /// If the index is out of bounds of the number of allocated pages, we will
    /// get back a pointer to unusable memory.
    unsafe fn pointer_for_index(&self, index: usize) -> *mut Page {
        self.start_addr.load(core::sync::atomic::Ordering::Relaxed).add(index)
    }


    /// Ensure the page referenced by the given pointer is properly mapped and
    /// returns the index of that page within the bitmap.
    /// 
    /// # Errors
    /// Returns an error if the page being requested is either not mapped, not
    /// aligned or not present within the bitmap.
    fn ensure_mapped_index(&self, ptr: *mut Page) -> Result<usize, KernelPageBitmapAllocatorError> {
        let start_ptr = self.start_addr.load(core::sync::atomic::Ordering::Relaxed);
        let mapped_count = self.count.load(core::sync::atomic::Ordering::Relaxed);

        if start_ptr.addr() % PAGE_SIZE != 0 {
            return Err(KernelPageBitmapAllocatorError::UnalignedPage { address: ptr.addr() });
        }

        if start_ptr <= ptr {
            let index = unsafe { ptr.sub_ptr(start_ptr) };
            if index >= mapped_count {
                Err(KernelPageBitmapAllocatorError::PageNotMapped { address: ptr.addr() })
            }
            else {
                Ok(index)
            }
        }
        else {
            Err(KernelPageBitmapAllocatorError::PageNotMapped { address: ptr.addr() })
        }
    }

    /// Allocate a number of continuous pages.
    /// 
    /// # Errors
    /// If no suitable location is able to be found, an `OutOfMemoryError` is
    /// returned.
    pub fn allocate_pages(&self, count: usize) -> Result<*mut Page, KernelPageBitmapAllocatorError> {
        let count_up_to = 64 - count;

        for entry_index in 0..self.bitmap.load(atomic::Ordering::Relaxed).len() {
            for bit in 0..count_up_to {
                if self.mark_pages_as_allocated(entry_index * 64 + bit, count) {
                    return Ok(unsafe { self.pointer_for_index(entry_index * 64 + bit) });
                }
            }
        }

        Err(KernelPageBitmapAllocatorError::OutOfMemory { requested: count })
    }

    /// Free a consecutive region of pages.
    /// 
    /// # Errors
    /// Returns an error if the location given is unaligned or not within the
    /// mapped region. Also returns an error if the entire region of pages is
    /// not within the bounds of the mapped area.
    /// 
    /// # Safety
    /// The location and count must reflect an allocated region from this
    /// allocator.
    pub unsafe fn free_pages(&self, location: *mut Page, count: usize) -> Result<(), KernelPageBitmapAllocatorError> {
        // Ensure first page is valid and get the index
        let index = self.ensure_mapped_index(location)?;

        // Ensure the last page is valid
        self.ensure_mapped_index(unsafe { location.add(count.max(1) - 1) })?;

        // Mark the pages as free
        self.mark_pages_as_free(index, count);

        Ok(())
    }

    pub fn dump(&self) {
        kprintln!(unsafe "----------------");
        for v in self.bitmap.load(atomic::Ordering::Relaxed) {
            kprintln!(unsafe "{:#066b}", v.load(core::sync::atomic::Ordering::Relaxed))
        }
        kprintln!(unsafe "----------------");
    }
}

/// Potential errors which can be returned when attempting an allocation or
/// free via a `KernelPageBitmapAllocator`.
#[derive(Debug, Clone, Copy)]
pub enum KernelPageBitmapAllocatorError {
    OutOfMemory{requested: usize},
    PageNotMapped{address: usize},
    UnalignedPage{address: usize}
}

impl core::fmt::Display for KernelPageBitmapAllocatorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            KernelPageBitmapAllocatorError::OutOfMemory { requested } => {
                write!(f, "KernelPageBitmapAllocatorError::OutOfMemory - Cannot allocate {} page{}", requested, if *requested > 1 { "s" } else { "" })
            },
            KernelPageBitmapAllocatorError::PageNotMapped { address } => {
                write!(f, "KernelPageBitmapAllocatorError::PageNotMapped - Cannot free page at {address:#x}, which is outside of mapped region")
            },
            KernelPageBitmapAllocatorError::UnalignedPage { address } => {
                write!(f, "KernelPageBitmapAllocatorError::UnalignedPage - Cannot free page at {address:#x}, which is not aligned to a page boundary")
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::mem::{PAGE_SIZE, Page};

    use super::KernelPageBitmapAllocator;

    static mut SIX_PAGES: [Page; 6] = [Page([0; PAGE_SIZE]); 6];

    #[test_case]
    fn allocate_to_empty() {
        let allocator = KernelPageBitmapAllocator::new();

        assert!(allocator.allocate_pages(4).is_err());
        assert!(allocator.allocate_pages(2).is_err());
        assert!(allocator.allocate_pages(1).is_err());
    }

    #[test_case]
    fn allocate_and_free_single() {
        let allocator = KernelPageBitmapAllocator::new();
        allocator.initialize_with_buffer(unsafe{ &mut SIX_PAGES });

        let first_page = allocator.allocate_pages(4).expect("Unable to initialize expected memory");

        assert!(allocator.allocate_pages(1).is_ok());
        assert!(allocator.allocate_pages(4).is_err());
        assert!(allocator.allocate_pages(2).is_err());
        unsafe { allocator.free_pages(first_page, 4).unwrap() };

        let second_page = allocator.allocate_pages(4).expect("Unable to initialize expected memory");

        assert_eq!(first_page, second_page);
    }
}

#[cfg(test)]
pub mod sync_test {
    use crate::{mem::{Page, PAGE_SIZE}, harts::{machine_mode_is_primary_hart, machine_mode_sync, CORE_COUNT}, asm::HEAP_START, asm::HEAP_END};
    use crate::harts;

    use super::KernelPageBitmapAllocator;

    static FIVE_PAGES: [Page; 5] = [Page([0; PAGE_SIZE]); 5];

    static BITMAP_ALLOC: KernelPageBitmapAllocator = unsafe { KernelPageBitmapAllocator::new() };

    const ALLOC_SIZES: &[usize] = &[8, 16, 32, 8, 16, 4, 2, 1, 32, 32, 32, 1, 1, 1, 1, 8, 8, 8, 8, 16, 1, 2, 4, 8, 16, 32, 63, 63, 32, 16, 12, 4];

    pub fn collective_test() {
        let heap_size: usize = unsafe { (crate::asm::HEAP_END as usize - crate::asm::HEAP_START as usize)/PAGE_SIZE };

        let mut alloc_list: &mut[Option<*mut Page>] = &mut [None; ALLOC_SIZES.len()];

        
        if machine_mode_is_primary_hart() {    
            unsafe { BITMAP_ALLOC.initialize(core::ops::Range { start: HEAP_START, end: HEAP_END }) }
        }

        assert!(ALLOC_SIZES.iter().map(|v| *v).sum::<usize>() * CORE_COUNT < unsafe { HEAP_END.sub_ptr(HEAP_START) });

        machine_mode_sync();

        let count = 24;

        for _ in 0..count {
            for (alloc, size) in alloc_list.iter_mut().zip(ALLOC_SIZES.iter()) {
                *alloc = Some(BITMAP_ALLOC.allocate_pages(*size).expect("Test should not overrun allocation area"));
                // BITMAP_ALLOC.dump();
            }

            for (alloc, size) in alloc_list.iter_mut().zip(ALLOC_SIZES.iter()) {
                let alloc = alloc.take().unwrap();
                assert!(unsafe { BITMAP_ALLOC.free_pages(alloc, *size) }.is_ok());
                // BITMAP_ALLOC.dump();
            }
        }

        machine_mode_sync();
    }
}