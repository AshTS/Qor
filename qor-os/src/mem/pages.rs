use libutils::sync::InitThreadMarker;
use libutils::sync::NoInterruptMarker;

use crate::mem::error::GlobalPageAllocatorError;
use crate::mem::Page;
use crate::mem::PAGE_SIZE;

/// Internal Global Kernel Page Allocator Data
struct GlobalPageAllocatorData {
    bitmap: core::ptr::Unique<u64>,
    count: usize,
    start_addr: core::ptr::NonNull<Page>,
}

/// Global Page Allocator Object
pub struct GlobalPageAllocator {
    data: libutils::sync::Mutex<Option<GlobalPageAllocatorData>>,
}

/// A `Box` like structure for continuous ranges of kernel pages
pub struct KernelPageBox<T> {
    ptr: core::ptr::Unique<T>,
    length: usize,
}

const ALLOCATED: bool = true;
const FREE: bool = false;

impl GlobalPageAllocatorData {
    /// Construct an offset and mask from an index into the bitmask
    fn offset_and_mask(index: usize) -> (usize, u64) {
        (index / 64, 1 << (index % 64))
    }

    /// Check if the page at a given index is allocated
    fn is_page_allocated(&self, index: usize) -> bool {
        // Make sure that we only accept requests within the bounds of the available space
        assert!(index < self.count);

        // Get the offset into the pages and the mask for the page
        let (offset, mask) = Self::offset_and_mask(index);

        // Safety: Since we have a reference to the data, it is safe for us to acquire a reference to the bitmap since we own it.
        let flag = unsafe { self.bitmap.as_ptr().add(offset).as_ref() }.unwrap();

        *flag & mask > 0
    }

    /// Allocate the page at the given index
    fn allocate_page(&mut self, index: usize) -> Result<(), ()> {
        // Make sure that we only accept requests within the bounds of the available space
        assert!(index < self.count);

        // Get the offset into the pages and the mask for the page
        let (offset, mask) = Self::offset_and_mask(index);

        // Safety: Since we have a mutable reference to the data, it is safe for us to acquire a mutable reference to the bitmap since we own it.
        let flag = unsafe { self.bitmap.as_ptr().add(offset).as_mut() }.unwrap();

        if *flag & mask > 0 {
            Err(())
        } else {
            *flag |= mask;
            Ok(())
        }
    }

    /// Free the page at the given index, returning the previous state of the page, true if allocated, false if not
    fn free_page(&mut self, index: usize) -> bool {
        // Make sure that we only accept requests within the bounds of the available space
        assert!(index < self.count);

        // Get the offset into the pages and the mask for the page
        let (offset, mask) = Self::offset_and_mask(index);

        // Safety: Since we have a mutable reference to the data, it is safe for us to acquire a mutable reference to the bitmap since we own it.
        let flag = unsafe { self.bitmap.as_ptr().add(offset).as_mut() }.unwrap();

        // Store the result before masking off the flag
        let result = *flag & mask > 0;

        *flag &= !mask;

        result
    }
}

impl GlobalPageAllocator {
    /// Construct a new Global Page Allocator object, this would be stored statically, and accessed only in single threaded contexts
    pub const fn new() -> Self {
        Self {
            data: libutils::sync::Mutex::new(None),
        }
    }

    /// Initialize the Global Page Allocator, this must be done when interrupts are disabled as we are constructing a global state
    pub fn initialize(
        &self,
        init_thread: InitThreadMarker,
        _no_interrupts: NoInterruptMarker,
    ) -> Result<(), GlobalPageAllocatorError> {
        // First, we get the start and end of the heap
        let heap_start = unsafe { crate::asm::HEAP_START };
        let heap_end = unsafe { crate::asm::HEAP_END };
        let heap_size = heap_end - heap_start;

        // Next, we can determine how many pages of memory we have available
        let heap_page_count = heap_size / PAGE_SIZE;
        kdebugln!(
            init_thread,
            KernelPageTable,
            "Initializing Page Heap with {} pages ({} KiB)",
            heap_page_count,
            heap_page_count * PAGE_SIZE
        );

        // Now, we need the bitmap to denote page allocations, we will construct this in the first couple of pages we have been given
        let bits_per_page = PAGE_SIZE * 8;
        let pages_for_bitmap = (heap_page_count + bits_per_page - 1) / bits_per_page;

        // Make sure we actually have the space for the bitmap (this is only a problem for extremely low memory systems)
        if pages_for_bitmap >= heap_page_count {
            return Err(GlobalPageAllocatorError::BitmapSpaceError);
        }

        // Get the address of the bitmap and remainder of the heap
        let bitmap_address = heap_start as *mut u64;
        let page_heap_address = (heap_start + PAGE_SIZE * pages_for_bitmap) as *mut Page;

        // Remove these pages from the final count
        let final_heap_page_count = heap_page_count - pages_for_bitmap;

        // Fill the bitmap with zeros
        // Safety: We constructed this pointer from the start of the heap, so we know the memory is not addressed anywhere else, furthermore, we have the `InitThreadMarker`, so we know that the access cannot alias
        unsafe {
            for i in 0..(pages_for_bitmap * PAGE_SIZE) {
                bitmap_address.add(i).write_volatile(0);
            }
        }

        // Safety: We know this will not deadlock since we are in an interrupt free context
        *self.data.spin_lock() = Some(GlobalPageAllocatorData {
            bitmap: core::ptr::Unique::new(bitmap_address).expect("Qor Kernel Requires a Heap"),
            count: final_heap_page_count,
            start_addr: core::ptr::NonNull::new(page_heap_address)
                .expect("Qor Kernel Requires Non Null Heap"),
        });

        Ok(())
    }

    /// Allocate `page_count` pages from the global page allocator table, this must be done with interrupts disabled as we are mutating a global state
    pub fn allocate_pages_raw(
        &self,
        _no_interrupts: NoInterruptMarker,
        page_count: usize,
    ) -> Result<(*mut Page, usize), GlobalPageAllocatorError> {
        if let Some(data) = &mut *self.data.spin_lock() {
            // We loop over the indexes
            let mut index = 0;
            while index <= data.count - page_count {
                let mut viable = true;
                for add in 0..page_count {
                    viable = viable && (data.is_page_allocated(index + add) == FREE);
                    if !viable {
                        index += add + 1;
                        break;
                    }
                }

                if viable {
                    for add in 0..page_count {
                        assert!(data.allocate_page(index + add).is_ok());
                    }

                    // Safety: Index can't be greater than the number of pages given to the allocator, so this pointer addition is safe
                    let start_pointer = unsafe { data.start_addr.as_ptr().add(index) };

                    kdebugln!(unsafe KernelPageTable, "Allocating {} pages at {:?}", page_count, start_pointer);

                    return Ok((start_pointer, page_count));
                }
            }

            Err(GlobalPageAllocatorError::OutOfMemory)
        } else {
            Err(GlobalPageAllocatorError::NotInitialized)
        }
    }

    /// Statically allocate a block of pages, returns an `&'static mut [T]`, which cannot be safely freed. See `allocate_static_pages`.
    pub fn allocate_static_pages_data<T: Copy>(
        &self,
        no_interrupts: NoInterruptMarker,
        page_count: usize,
        fill: T,
    ) -> Result<&'static mut [T], GlobalPageAllocatorError> {
        // We must verify that the type `T` fits within the allocated space
        assert!(core::mem::size_of::<T>() <= PAGE_SIZE * page_count);

        let (pointer, length) = self.allocate_pages_raw(no_interrupts, page_count)?;

        let pointer = pointer as *mut T;
        let length = length * PAGE_SIZE / core::mem::size_of::<T>();

        // Safety: We know that such a reference can be created as the pointer is gaurenteed to have been properly allocated, and we cannot give out another reference to this memory until this one has been returned. Becuase the memory is allocated on a global heap, we know it must have a static lifetime.
        let slice = unsafe { core::slice::from_raw_parts_mut(pointer, length) };

        // Fill the slice with the value we have been given to fill it with
        slice.fill(fill);

        Ok(slice)
    }

    /// Statically allocate a block of pages, returns an `&'static mut [Page]`, which cannot be safely freed. This is because there is no mechanism for handling a drop on a mutable reference like that. If a block of pages that is able to be freed is desired, use `allocate_pages` to produce a `KernelPageBox` which allows the pages to be freed when the box is dropped. This must be done with interrupts disabled as we are mutating a global state.
    pub fn allocate_static_pages(
        &self,
        no_interrupts: NoInterruptMarker,
        page_count: usize,
    ) -> Result<&'static mut [Page], GlobalPageAllocatorError> {
        let (pointer, length) = self.allocate_pages_raw(no_interrupts, page_count)?;

        // Safety: We know that such a reference can be created as the pointer is gaurenteed to have been properly allocated, and we cannot give out another reference to this memory until this one has been returned. Becuase the memory is allocated on a global heap, we know it must have a static lifetime.
        let slice = unsafe { core::slice::from_raw_parts_mut(pointer, length) };

        Ok(slice)
    }

    /// Statically allocate a block of memory appropriately sized to be stored in pages
    pub fn allocate_static<T>(
        &self,
        no_interrupts: NoInterruptMarker,
        data: T,
    ) -> Result<&'static mut T, GlobalPageAllocatorError> {
        let page_count = (core::mem::size_of::<T>() + PAGE_SIZE - 1) / PAGE_SIZE;

        let (pointer, _) = self.allocate_pages_raw(no_interrupts, page_count)?;

        let pointer = pointer as *mut T;

        // Safety: We know that such a reference can be created as the pointer is gaurenteed to have been properly allocated, and we cannot give out another reference to this memory until this one has been returned. Becuase the memory is allocated on a global heap, we know it must have a static lifetime.
        unsafe { pointer.write(data) };
        Ok(unsafe { pointer.as_mut().unwrap() })
    }

    /// Allocate a `KernelPageBox` for a block of pages. This must be done with interrupts disabled as we are mutating a global state
    pub fn allocate_pages<T>(
        &self,
        no_interrupts: NoInterruptMarker,
        page_count: usize,
        data: T,
    ) -> Result<KernelPageBox<T>, GlobalPageAllocatorError> {
        let (ptr, length) = self.allocate_pages_raw(no_interrupts, page_count)?;

        Ok(unsafe { KernelPageBox::from_raw(ptr as *mut T, length, data) })
    }

    /// Allocate kernel pages of the proper length for the data given.
    pub fn allocate<T>(
        &self,
        no_interrupts: NoInterruptMarker,
        data: T,
    ) -> Result<KernelPageBox<T>, GlobalPageAllocatorError> {
        self.allocate_pages(
            no_interrupts,
            (core::mem::size_of::<T>() + PAGE_SIZE - 1) / PAGE_SIZE,
            data,
        )
    }

    /// Free a block of pages back to the page allocator table, this must be done with interrupts disabled as we are mutating a global state.
    ///
    /// # Safety:
    ///
    /// The `start_pointer` must point to a valid, allocated range of memory which is `page_count` pages long. Note that all future references to that memory becomes invalid.
    pub unsafe fn free_pages_unchecked(
        &self,
        _no_interrupts: NoInterruptMarker,
        start_pointer: *mut Page,
        page_count: usize,
    ) -> Result<(), GlobalPageAllocatorError> {
        kdebugln!(unsafe KernelPageTable, "Freeing {} pages at {:?}", page_count, start_pointer);

        if let Some(data) = &mut *self.data.spin_lock() {
            let index = (start_pointer as usize - data.start_addr.as_ptr() as usize) / PAGE_SIZE;

            for add in 0..page_count {
                if data.free_page(index + add) == FREE {
                    return Err(GlobalPageAllocatorError::MemoryNotAllocated(
                        start_pointer,
                        page_count,
                    ));
                }
            }

            Ok(())
        } else {
            Err(GlobalPageAllocatorError::NotInitialized)
        }
    }
}

impl<T> KernelPageBox<T> {
    /// Construct a new kernel page box from its constituent parts. This is effectively just a slice with a drop implementation. It takes a pointer to the data contained and the number of pages.
    ///
    /// Safety: The pointer must point to a valid, allocated range of the proper number of pages
    pub unsafe fn from_raw(ptr: *mut T, length: usize, data: T) -> Self {
        // Ensure that the proper size is present
        assert!(core::mem::size_of::<T>() <= PAGE_SIZE * length);

        ptr.write(data);

        Self {
            ptr: core::ptr::Unique::new(ptr).unwrap(),
            length,
        }
    }

    /// Create a new kernel page box by calling allocate on the global page allocator. Note that this creates a new no_interrupt context, if many pieces of data must be allocated, try to do so explicitly in a larger no_interrupt context.
    pub fn new(data: T) -> Result<Self, GlobalPageAllocatorError> {
        libutils::sync::no_interrupts_supervisor(|no_interrupts| {
            crate::mem::PAGE_ALLOCATOR.allocate(no_interrupts, data)
        })
    }

    /// Free the memory stored in a KernelPageBox
    pub fn free(self, _no_interrupts: NoInterruptMarker) {
        drop(self);
    }

    /// Get the raw pointer
    pub fn raw(&self) -> *const T {
        self.ptr.as_ptr()
    }

    /// Get the internal slice
    pub fn get(&self) -> &T {
        // Safety: We can dereference the contained pointer as we know that we are the sole owners of that pointer
        unsafe { self.ptr.as_ref() }
    }

    /// Get a mutable reference to the internal slice
    pub fn get_mut(&mut self) -> &mut T {
        // Safety: We can dereference the contained pointer as we know that we are the sole owners of that pointer
        unsafe { self.ptr.as_mut() }
    }

    /// Leak the memory used for the page table
    pub fn leak(mut self) -> &'static mut T {
        self.length = 0;
        // Safety: We can dereference the contained pointer as we know that we are the sole owners of that pointer
        unsafe { &mut *self.ptr.as_ptr() }
    }
}

impl<T> core::ops::Deref for KernelPageBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> core::ops::DerefMut for KernelPageBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

impl<T> core::ops::Drop for KernelPageBox<T> {
    fn drop(&mut self) {
        if self.length != 0 {
            let _ = unsafe { self.ptr.as_ptr().read() };
        

            // Drop the pages allocated
            libutils::sync::no_interrupts_supervisor(|no_interrupts| unsafe {
                crate::mem::PAGE_ALLOCATOR
                    .free_pages_unchecked(no_interrupts, self.ptr.as_ptr() as *mut Page, self.length)
                    .expect("Failed to free memory from `KernelPageBox`");
            })
        }
    }
}
