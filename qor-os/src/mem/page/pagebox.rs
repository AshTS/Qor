use super::Page;

/// Box like structure which stores a sequential range of pages allocated by
/// the kernel. The space is deallocated when this object is dropped.
/// 
/// We give the guarantee that the pointers making up the page range are
/// properly aligned, in the right order, and represent pages allocated by the
/// global bitmap page allocator.
pub struct KernelPageSequence {
    pointer_range: core::ops::Range<*mut Page>,
}

impl KernelPageSequence {
    /// Construct a `KernelPageSequence` from a raw pointer range.
    /// 
    /// # Safety
    /// The range given must be made up of pointers which are properly aligned
    /// to page boundaries, are in the right order, and represent a range of
    /// pages allocated by the kernel page allocator.
    pub const unsafe fn from_raw(range: core::ops::Range<*mut Page>) -> Self {
        Self {
            pointer_range: range
        }
    }

    /// Get the start of the page sequence as a pointer
    pub fn as_ptr(&self) -> *mut Page {
        self.pointer_range.start
    }

    /// Get the number of pages in the allocation
    pub fn page_count(&self) -> usize {
        // Safety: The pointer range must be in the correct order and aligned
        // by the guarantee of the `pointer_range` value.
        unsafe { self.pointer_range.end.sub_ptr(self.pointer_range.start) }
    }

    /// Leak the allocated sequence so it never gets deallocated, returning a
    /// static mutable reference to the sequence of pages.
    pub fn leak(sequence: Self) -> &'static mut [Page] {
        // Safety: This assumes that the pointer range is a valid pointer range
        // allocated by the Kernel Page Bitmap Allocator, as guaranteed by the
        // requirements of the `pointer_range` value
        let slice = unsafe { core::slice::from_mut_ptr_range(sequence.pointer_range.clone()) };

        slice
    }

    /// Calling this function invalidates the backing memory of the sequence,
    /// and is only to be used as an implementation detail of the drop and leak
    /// functions.
    /// 
    /// # Safety
    /// This function must never be called before a use of the backing memory
    /// occurs as we invalidate that memory here.
    unsafe fn free_raw(&self) {
        // Safety: This assumes that the pointer range is a valid pointer range
        // allocated by the Kernel Page Bitmap Allocator, as guaranteed by the
        // requirements of the `pointer_range` value
        unsafe { crate::mem::BITMAP_ALLOC.free_pages(self.pointer_range.start, self.page_count()).unwrap() }
    }
}

impl core::ops::Drop for KernelPageSequence {
    fn drop(&mut self) {
        // Safety: We are in the drop implementation, so we can be assured this
        // object will never be used again.
        unsafe { self.free_raw() }
    }
}

#[cfg(test)]
mod test {
    use super::super::BITMAP_ALLOC;

    #[test_case]
    fn many_page_sequence_allocations() {
        for _ in 0..2048 {
            let sequence = BITMAP_ALLOC.allocate_page_sequence(63).expect("Able to allocate page sequence");
            core::hint::black_box(sequence);
        }
    }
}

#[cfg(test)]
pub mod sync_test {
    use crate::harts;

    use super::super::BITMAP_ALLOC;

    pub fn many_cores_page_seq_allocations() {
        harts::machine_mode_sync();

        for _ in 0..2048 {
            let sequence = BITMAP_ALLOC.allocate_page_sequence(16).expect("Able to allocate page sequence");
            core::hint::black_box(sequence);
        }
    }
}