use libutils::sync::{InitThreadMarker, NoInterruptMarker};

use super::PageCount;

/// Allocation Chunk Metadata
#[derive(Debug, Clone, Copy)]
pub struct AllocationChunk {
    next: Option<core::ptr::Unique<Option<AllocationChunk>>>,
    ptr: usize,
    size: usize,
    free: bool,
}

/// Allocator object, wraps an array of chunks along with a root chunk
pub struct Allocator {
    chunks: &'static mut [Option<AllocationChunk>],
    root: Option<AllocationChunk>,
}

/// Structure to hold the kernel heap allocator
pub struct GlobalAllocator {
    allocator: core::cell::UnsafeCell<Option<Allocator>>,
}

impl AllocationChunk {
    /// Construct a new allocation chunk for a given size at the given pointer
    pub fn new(ptr: usize, size: usize) -> Self {
        Self {
            next: None,
            ptr,
            size,
            free: true,
        }
    }

    /// Walk the Allocation Chunks in order
    pub fn in_order<F: Fn(&mut Self)>(&mut self, f: &F) {
        f(self);

        if let Some(mut next) = self.next {
            unsafe { next.as_mut() }.unwrap().in_order(f);
        }
    }

    /// Split the current chunk in two
    pub fn split(&mut self, size: usize, other: &mut Option<AllocationChunk>) {
        assert!(size <= self.size);

        if size < self.size {
            let other_value = Self {
                next: self.next,
                free: true,
                size: self.size - size,
                ptr: self.ptr + size,
            };

            let _ = other.insert(other_value);

            self.size = size;
            self.free = false;

            self.next =
                Some(core::ptr::Unique::new(other as *mut Option<AllocationChunk>).unwrap());
        } else {
            self.free = false;
        }
    }

    /// Check if the current chunk contains a pointer
    pub fn contains_ptr(&self, ptr: usize) -> bool {
        self.ptr <= ptr && ptr < self.ptr + self.size
    }

    /// Attempt to combine this chunk with the next one
    pub fn attempt_combine(&mut self) {
        if !self.free {
            return;
        }

        if let Some(mut next) = self.next {
            let next_option = unsafe { next.as_mut() };
            let mut new_next = None;
            if let Some(next) = next_option {
                if next.free {
                    self.size += next.size;
                    new_next = Some(next.next);
                }
            }

            if let Some(new_next) = new_next {
                self.next = new_next;
                *next_option = None;
            }
        }
    }

    /// Check if this chunk has the proper alignment and size, returning the eventually needed size if it workss
    pub fn check_layout(&self, layout: &alloc::alloc::Layout) -> Option<usize> {
        let overlap = self.ptr & (layout.align() - 1);
        let extra = (layout.align() - overlap) % layout.align();

        let total_size = layout.size() + extra;

        if total_size <= self.size {
            Some(total_size)
        } else {
            None
        }
    }
}

impl core::fmt::Display for AllocationChunk {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "<{:#x} byte{} {} at {:#x}>",
            self.size,
            if self.size > 1 { "s" } else { "" },
            if self.free { "free" } else { "allocated" },
            self.ptr
        )
    }
}

impl Allocator {
    /// Construct a new allocator, takes a slice of allocator chunks, a pointer to the memory and a size
    ///
    /// # Safety
    /// The pointer and size must be valid and correct
    pub unsafe fn new(
        chunks: &'static mut [Option<AllocationChunk>],
        pointer: usize,
        size: usize,
    ) -> Self {
        Self {
            chunks,
            root: Some(AllocationChunk::new(pointer, size)),
        }
    }

    /// Search for a free allocator chunk
    pub fn find_free_chunk(&mut self) -> Option<usize> {
        for (i, v) in self.chunks.iter_mut().enumerate() {
            if v.is_none() {
                return Some(i);
            }
        }

        None
    }

    /// Allocate a chunk of memory
    pub fn allocate_memory(&mut self, layout: alloc::alloc::Layout) -> Option<usize> {
        let chunk_index = self
            .find_free_chunk()
            .expect("Unable to find another free chunk");
        let mut walking_reference = &mut self.root;

        loop {
            if let Some(walking) = &mut walking_reference {
                if walking.free {
                    if let Some(eventual_size) = walking.check_layout(&layout) {
                        // Construct the final pointer
                        let overlap = walking.ptr & (layout.align() - 1);
                        let extra = (layout.align() - overlap) % layout.align();

                        let ptr = Some(walking.ptr + extra);

                        // Allocate the space

                        walking.split(eventual_size, &mut self.chunks[chunk_index]);

                        return ptr;
                    }
                }

                if let Some(next) = walking.next {
                    walking_reference = unsafe { next.as_ptr().as_mut().unwrap() };
                } else {
                    panic!("Unable to allocate {:?}", layout);
                }
            } else {
                unreachable!()
            }
        }
    }

    /// Free a chunk of memory
    pub fn free_memory(&mut self, ptr: usize) {
        let mut walking_reference = &mut self.root;

        while let Some(walking) = walking_reference {
            let found = walking.free == false && walking.contains_ptr(ptr);

            if found {
                walking.free = true;
                walking.attempt_combine();
                break;
            }

            if walking.free {
                walking.attempt_combine();
            }

            if let Some(next) = walking.next {
                walking_reference = unsafe { next.as_ptr().as_mut().unwrap() };
            } else {
                panic!("The address {:x} is not allocated", ptr);
            }
        }
    }

    /// Attempt to clean up the memory
    pub fn clean_up(&mut self) {
        let mut walking_reference = &mut self.root;

        while let Some(walking) = walking_reference {
            if walking.free {
                walking.attempt_combine();
            }

            if let Some(next) = walking.next {
                walking_reference = unsafe { next.as_ptr().as_mut().unwrap() };
            } else {
                break;
            }
        }
    }
}

impl core::fmt::Display for Allocator {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut walking_reference = &self.root;

        while let Some(walking) = walking_reference {
            writeln!(f, "{}", walking)?;

            if let Some(next) = walking.next {
                walking_reference = unsafe { next.as_ptr().as_ref().unwrap() };
            } else {
                break;
            }
        }

        Ok(())
    }
}

impl GlobalAllocator {
    /// Construct a new, empty global allocator
    pub const fn new() -> Self {
        Self {
            allocator: core::cell::UnsafeCell::new(None),
        }
    }

    /// Initialize the allocator
    pub fn initialize(&self, _init_thread: InitThreadMarker, no_interrupts: NoInterruptMarker, pages: PageCount) {
        // Allocate the proper number of pages
        let entries = crate::mem::PAGE_ALLOCATOR
            .allocate_static(no_interrupts, [None; 4096])
            .expect("Unable to allocate allocation entries");

        let pointer = crate::mem::PAGE_ALLOCATOR
            .allocate_static_pages(no_interrupts, pages)
            .expect("Unable to allocate space for heap")
            .as_ptr();

        unsafe {
            self.allocator
                .get()
                .as_mut()
                .unwrap()
                .insert(Allocator::new(
                    entries,
                    pointer as usize,
                    crate::mem::PAGE_SIZE * pages.raw(),
                ))
        };
    }
}

unsafe impl core::alloc::GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        // Get a critical section, this is not technically enough, we also need to make sure we are in the proper thread, for now we will ignore this, but in future, each thread needs to be given its own allocator
        libutils::sync::no_interrupts_supervisor(|_| {
            if let Some(allocator) = self.allocator.get().as_mut().unwrap() {
                allocator.allocate_memory(layout).unwrap() as *mut u8
            } else {
                panic!("Global Allocator Not Initialized")
            }
        })
    }

    unsafe fn dealloc(&self, data_ptr: *mut u8, _layout: core::alloc::Layout) {
        // Get a critical section, this is not technically enough, we also need to make sure we are in the proper thread, for now we will ignore this, but in future, each thread needs to be given its own allocator
        libutils::sync::no_interrupts_supervisor(|_| {
            if let Some(allocator) = self.allocator.get().as_mut().unwrap() {
                allocator.free_memory(data_ptr as usize)
            } else {
                panic!("Global Allocator Not Initialized")
            }
        })
    }
}

unsafe impl Sync for GlobalAllocator {}
unsafe impl Send for GlobalAllocator {}

/// Allocation error handler
#[alloc_error_handler]
pub fn alloc_error(l: core::alloc::Layout) -> ! {
    panic!(
        "Allocator failed to allocate {} bytes with {}-byte alignment.",
        l.size(),
        l.align()
    );
}

// Assign a new global allocator
#[global_allocator]
pub static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator::new();
