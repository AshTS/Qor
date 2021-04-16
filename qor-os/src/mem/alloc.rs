//! Byte Grain Allocator
use crate::*;

// Pointer to the start of the kernel heap
static KERNEL_HEAP_POINTER: core::sync::atomic::AtomicPtr<AllocatorHead> = core::sync::atomic::AtomicPtr::new(0 as *mut AllocatorHead);

#[repr(C)]
/// Node for the allocator linked list
struct Node
{
    ptr: *mut u8,
    next: Option<core::ptr::NonNull<Node>>,
    size: u32,
    flags: u32 // Bit 0: Is Valid ()
}

impl Node
{
    pub fn new(ptr: *mut u8, next: Option<core::ptr::NonNull<Node>>, size: u32, flags: u32) -> Self
    {
        Self
        {
            ptr, next, size, flags
        }
    }

    pub fn is_taken(&self) -> bool
    {
        self.flags & 2 > 0
    }
}

impl core::fmt::Display for Node
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result
    {
        write!(f, "<Node ptr: 0x{:x}, next: {:?}, size: {}, flags: 0b{:b}>", 
        self.ptr as usize, self.next, self.size, self.flags)
    }
}

#[repr(C)]
struct AllocatorHead
{
    head_node: &'static mut Node,
    num_nodes: usize,
    max_nodes: usize,
}

impl AllocatorHead
{
    /// Create a new Allocator Head
    pub fn new(table_pages: usize, memory_pages: usize) -> Self
    {
        // Allocate the proper number of tables
        // Safety: This is safe because the address was retrieved from the kernel page allocator
        let head_node = unsafe { (mem::kpalloc(table_pages) as *mut Node).as_mut() }.unwrap();
        let mem_head = mem::kpalloc(memory_pages);

        // Create the head node
        let node = Node::new(mem_head, None, (memory_pages * 4096) as u32, 1);

        // Calculate the maximum number of nodes
        let max_nodes = table_pages * 4096 / core::mem::size_of::<Node>();

        // Insert the head node
        *head_node = node;

        Self
        {
            head_node,
            num_nodes: 1,
            max_nodes
        }
    }

    /// Display the node table
    pub fn display_table(&self)
    {
        for i in 0..self.num_nodes
        {
            let ptr = unsafe { (self.head_node as *const Node).add(i).as_ref() }.unwrap();

            kdebugln!(MemoryAllocation, "Node {:2}: {} -> {}", i, ptr,
                if let Some(n) = ptr.next
                {
                    ((unsafe { n.as_ref() } as *const Node as usize - self.head_node as *const Node as usize) / core::mem::size_of::<Node>() )as isize
                }
                else
                {
                    -1 as isize
                }
        );
        }
    }

    /// Add a node to the table
    pub fn add_node(&mut self, node: Node) -> &mut Node
    {
        if self.num_nodes == self.max_nodes
        {
            panic!("Cannot add anymore nodes to the allocated table");
        }

        for i in 0..self.num_nodes
        {
            let ptr = unsafe { (self.head_node as *mut Node).add(i).as_mut() }.unwrap();

            if ptr.flags & 1 == 0
            {
                *ptr = node;
                return ptr;
            }
        }

        let ptr = unsafe { (self.head_node as *mut Node).add(self.num_nodes).as_mut() }.unwrap(); 
    
        *ptr = node;
        self.num_nodes += 1;

        ptr
    }

    /// Allocate some space with the given layout
    pub fn allocate(&mut self, layout: core::alloc::Layout) -> *mut u8
    {
        kdebugln!(MemoryAllocation, "Allocating {} bytes with an alignment of {} bytes", layout.size(), layout.align());

        let size = layout.size();
        let align = layout.align();

        let mut head_pointer = unsafe { Box::from_raw(self.head_node) };

        let final_size;

        loop
        {
            if !head_pointer.is_taken() && head_pointer.size >= size as u32
            {
                let end = head_pointer.ptr as usize + head_pointer.size as usize;
                let canidate_ptr = end - size;
                
                let align_size_inc = canidate_ptr % align;
                let this_size = size + align_size_inc;

                if head_pointer.size >= this_size as u32
                {
                    final_size = this_size;
                    break;
                }
            }

            let next = if let Some(next) = head_pointer.next
            {
                next
            }
            else
            {
                panic!("Out of Memory");
            };

            head_pointer = unsafe { Box::from_raw(next.as_ptr()) };
            
        }

        kdebugln!(MemoryAllocation, "Found! {}", final_size);

        let ptr = head_pointer.ptr as usize + head_pointer.size as usize - final_size;

        head_pointer.size -= final_size as u32;

        kdebugln!(MemoryAllocation, "Found memory at 0x{:x}", ptr);

        let next = self.add_node(Node::new(ptr as *mut u8, head_pointer.next, final_size as u32, 3));
        head_pointer.next = core::ptr::NonNull::new(next as *mut Node);

        Box::leak(head_pointer);

        ptr as *mut u8
    }

    /// Walk the table and combine any adjacent nodes which are connected
    fn combine(&mut self)
    {
        let mut head_pointer = unsafe { Box::from_raw(self.head_node) };

        loop
        {
            let mut next = if let Some(next) = head_pointer.next
            {
                unsafe { Box::from_raw(next.as_ptr()) }
            }
            else
            {
                Box::leak(head_pointer);
                return;
            };

            if head_pointer.ptr as usize + head_pointer.size as usize == next.ptr as usize
            {
                if !head_pointer.is_taken() && !next.is_taken()
                {
                    next.flags &= !1;
                    head_pointer.size += next.size;
                    head_pointer.next = next.next;

                    Box::leak(next);

                    continue;
                }
            }

            Box::leak(head_pointer);
            head_pointer = next;
        }
    }

    /// Deallocate some space with the given layout
    pub fn deallocate(&mut self, ptr: *mut u8, layout: core::alloc::Layout)
    {
        kdebugln!(MemoryAllocation, "Dellocating {} bytes with an alignment of {} bytes at 0x{:x}", layout.size(), layout.align(), ptr as usize);

        let mut head_pointer = unsafe { Box::from_raw(self.head_node) };

        loop
        {
            if head_pointer.is_taken() && head_pointer.ptr == ptr
            {
                // Clear the taken flag (freeing the memory)
                head_pointer.flags &= !2;
                Box::leak(head_pointer);
                self.combine();
                return; 
            }

            let next = if let Some(next) = head_pointer.next
            {
                next
            }
            else
            {
                panic!("Attempting to free unallocated space");
            };

            Box::leak(head_pointer);
            head_pointer = unsafe { Box::from_raw(next.as_ptr()) };
        }
    }
}

pub fn init_kernel_heap(num_pages: usize)
{
    kprintln!("Initializing Kernel Heap with {} pages", num_pages);

    // Initialize the space for the allocator head
    let alloc_head = mem::kpalloc(1) as *mut AllocatorHead;

    // Insert a new allocator
    // Safety: This is safe because the address was retrieved form the kernel page allocator
    unsafe { *alloc_head = AllocatorHead::new(4, num_pages) };

    KERNEL_HEAP_POINTER.store(alloc_head, core::sync::atomic::Ordering::SeqCst);
}

pub fn display_heap_table()
{
    let ptr = KERNEL_HEAP_POINTER.load(core::sync::atomic::Ordering::SeqCst);

    if ptr.is_null()
    {
        panic!("Cannot display heap table without the Kernel Heap Initialized");
    }

    unsafe { ptr.as_ref() }.unwrap().display_table();
}

struct GlobalAllocator;

unsafe impl core::alloc::GlobalAlloc for GlobalAllocator
{
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8
    {
        let ptr = KERNEL_HEAP_POINTER.load(core::sync::atomic::Ordering::SeqCst);

        if ptr.is_null()
        {
            panic!("Cannot allocate without the Kernel Heap Initialized");
        }

        ptr.as_mut().unwrap().allocate(layout)
    }

    unsafe fn dealloc(&self, data_ptr: *mut u8, layout: core::alloc::Layout)
    {
        let ptr = KERNEL_HEAP_POINTER.load(core::sync::atomic::Ordering::SeqCst);

        if ptr.is_null()
        {
            panic!("Cannot deallocate without the Kernel Heap Initialized");
        }

        ptr.as_mut().unwrap().deallocate(data_ptr, layout)
    }
}

#[alloc_error_handler]
pub fn alloc_error(l: core::alloc::Layout) -> ! {
	panic!(
	       "Allocator failed to allocate {} bytes with {}-byte alignment.",
	       l.size(),
	       l.align()
	);
}

#[global_allocator]
static GA: GlobalAllocator = GlobalAllocator {};
