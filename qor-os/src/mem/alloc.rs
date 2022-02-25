//! Byte Based Kernel Allocator

use core::ptr::NonNull;

use crate::*;

// Overwrite sentinel flag
const SENTINEL: bool = false;

// Kernel Heap Pointer
static KERNEL_HEAP_POINTER: core::sync::atomic::AtomicPtr<AllocationHeader> = core::sync::atomic::AtomicPtr::new(0 as *mut AllocationHeader);

/// Memory Allocation Flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AllocationFlags(u32);

impl AllocationFlags
{
    /*
        Bit 0: Is Valid
        Bit 1: Is Free
    */

    /// Returns true iff the node is a valid node
    pub fn is_valid(&self) -> bool
    {
        self.0 & 1 > 0
    }

    /// Sets the valid node bit
    pub fn set_valid(&mut self)
    {
        self.0 |= 1;
    }

    /// Sets the invalid node bit
    pub fn set_invalid(&mut self)
    {
        self.0 &= !1;
    }

    /// Returns true iff the node is free
    pub fn is_free(&self) -> bool
    {
        self.0 & 2 > 0
    }

    /// Returns true iff the node is taken
    pub fn is_taken(&self) -> bool
    {
        !self.is_free()
    }

    /// Sets the valid node bit
    pub fn set_free(&mut self)
    {
        self.0 |= 2;
    }

    /// Sets the valid node bit
    pub fn set_taken(&mut self)
    {
        self.0 &= !2;
    }
}

impl AllocationFlags
{
    /// Free Flag
    pub const fn free() -> Self
    {
        Self(2)
    }

    /// Taken Flag
    pub const fn taken() -> Self
    {
        Self(0)
    }

    /// Valid Flag
    pub const fn valid() -> Self
    {
        Self(1)
    }

    /// Invalid Flag
    pub const fn invalid() -> Self
    {
        Self(0)
    }
}

impl core::ops::BitOr<AllocationFlags> for AllocationFlags
{
    type Output = AllocationFlags;

    fn bitor(self, rhs: AllocationFlags) -> Self::Output 
    {
        AllocationFlags(self.0 | rhs.0)
    }
}

/// Memory Allocation Node Pointer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodePtr
{
    page_ptr: core::ptr::NonNull<AllocationHeader>,
    index: usize
}

impl NodePtr
{
    pub fn new(page_ptr: core::ptr::NonNull<AllocationHeader>, index: usize) -> Self
    {
        Self
        {
            page_ptr, index
        }
    }

    pub unsafe fn dereference(&self) -> Option<&'static mut AllocationNode>
    {
        // First, dereference the Allocation Header
        let header = self.page_ptr.as_ptr().as_mut().unwrap();

        // Second, get the node
        Some(&mut header.nodes[self.index])
    }
}

/// Memory Allocation Node
#[derive(Debug)]
pub struct AllocationNode
{
    ptr: *mut u8,
    size: u64,
    next: Option<NodePtr>,
    flags: AllocationFlags
}

impl AllocationNode
{
    /// Create a new Memory Allocation Node
    pub fn new(ptr: *mut u8, next: Option<NodePtr>, size: u64, flags: AllocationFlags) -> Self
    {
        Self
        {
            ptr, next, size, flags
        }
    }

    /// Attempt to follow the next pointer
    pub fn follow_next(&self) -> Option<&mut AllocationNode>
    {
        self.next.map(|next| unsafe { next.dereference() }).flatten()
    }
}

// Number of Allocation Nodes in each Allocation Header
const ALLOCATION_NODES: usize = 100; // (super::PAGE_SIZE - 2 * core::mem::size_of::<Option<core::ptr::NonNull<AllocationHeader>>>()) / core::mem::size_of::<AllocationNode>();

/// Memory Allocation Header
pub struct AllocationHeader
{
    next_allocator: Option<core::ptr::NonNull<AllocationHeader>>,
    nodes: [AllocationNode; 100]
}

// Ensure the header is less than a page in length
static_assertions::const_assert_eq!((core::mem::size_of::<AllocationHeader>() < super::PAGE_SIZE), true);

impl AllocationHeader
{
    /// Allocate a new Allocation Header page and append it to the end of this one
    pub fn allocate_new_page(&mut self) -> &mut AllocationHeader
    {
        let mut page = unsafe { (super::kpalloc(1, "Byte Allocator Header").unwrap() as *mut AllocationHeader).as_mut() }.unwrap();

        page.next_allocator = None;

        self.next_allocator = Some(core::ptr::NonNull::new(page as *mut AllocationHeader).unwrap());

        // Fill in the nodes with invalid nodes
        for node in &mut page.nodes[..]
        {
            node.flags.set_invalid();
        }

        page
    }

    /// Allocate a new Allocation Header and return a pointer to that header
    pub fn new(starting_pages: usize, header_count: usize) -> Result<&'static mut AllocationHeader, super::page::KernelPageAllocationError>
    {
        // Allocate a new page
        let mut page = unsafe { (super::kpalloc(1, "Byte Allocator Header")? as *mut AllocationHeader).as_mut() }.unwrap();

        // Allocate the memory for the kernel
        let kernel_mem = super::kpalloc(starting_pages, "Byte Allocator Data")? as *mut u8;

        // Set this as the only allocator header
        page.next_allocator = None;

        // Fill in the nodes with invalid nodes
        for node in &mut page.nodes[1..]
        {
            node.flags.set_invalid();
        }

        // Create the first Allocation Node
        let first_node =
            AllocationNode::new(kernel_mem, None, (starting_pages * super::PAGE_SIZE) as u64,
                                AllocationFlags::free() | AllocationFlags::valid());

        // Insert the first node
        page.nodes[0] = first_node;

        let mut walking_prev = Some(core::ptr::NonNull::new(page as *mut AllocationHeader).unwrap());

        // Loop over the possible remaining nodes
        for _ in 1..header_count
        {
            let mut page = unsafe { (super::kpalloc(1, "Byte Allocator Header")? as *mut AllocationHeader).as_mut() }.unwrap();

            // Set the pointers
            page.next_allocator = None;

            unsafe { walking_prev.unwrap().as_mut().next_allocator = Some(core::ptr::NonNull::new(page as *mut AllocationHeader).unwrap()) };

            // Fill in the nodes with invalid nodes
            for node in &mut page.nodes[..]
            {
                node.flags.set_invalid();
            }

            walking_prev = Some(core::ptr::NonNull::new(page as *mut AllocationHeader).unwrap());
        }


        Ok(page)
    }

    /// Get the page pointer from an index into this header
    pub fn get_node_ptr(&self, index: usize) -> NodePtr
    {
        NodePtr::new(NonNull::from(self), index)
    }

    /// Display the node list
    pub fn display_node_list(&self)
    {
        kprintln!("Node List:");

        let mut ptr = self.get_node_ptr(0);

        loop
        {
            let node = unsafe { ptr.dereference() }.unwrap();
            kprint!("0x{:x}[{}]\t", ptr.page_ptr.as_ptr() as usize, ptr.index);
            if node.flags.is_valid()
            {
                kprint!("{} {} byte{}\t 0x{:x} - 0x{:x}",
                        if node.flags.is_taken() {"[ALLOC]"} else {"[FREE ]"},
                        node.size,
                        if node.size == 1 {""} else {"s"},
                        node.ptr as usize,
                        node.ptr as usize + node.size as usize - 1);

                kprintln!();

                if let Some(next) = node.next
                {
                    ptr = next;
                }
                else
                {
                    break;
                }
            }
            else
            {
                kprintln!("ERROR");
                break;
            }
        }
    }

    /// Get the first free node
    pub fn get_free(&mut self) -> NodePtr
    {
        for (i, node) in self.nodes.iter().enumerate()
        {
            if !node.flags.is_valid()
            {
                return self.get_node_ptr(i);
            }
        }

        // If nothing has been found, fall through to the next allocator
        if let Some(next) = self.next_allocator
        {
            return unsafe { next.as_ptr().as_mut().unwrap() }.get_free();
        }
        // Otherwise, we need to allocate another page
        kwarnln!("Allocating new page for allocation table");
        self.allocate_new_page().get_free()
    }

    /// Allocate some space with the given layout
    pub fn allocate(&mut self, layout: core::alloc::Layout) -> *mut u8
    {
        kdebugln!(ByteMemoryAllocation, "Allocating {} bytes with an alignment of {} bytes", layout.size(), layout.align());

        let mut node = unsafe { self.get_node_ptr(0).dereference().unwrap() };

        let next_free = self.get_free();

        let size = 
        if SENTINEL
        {
            layout.size() * 2
        }
        else
        {
            layout.size()
        };
    
        loop
        {
            if node.flags.is_valid()
            {
                // If a valid, free, and properly sized node is found
                if node.flags.is_valid() && node.flags.is_free() && node.size as usize >= size
                {
                    // And that node supports the proper padding
                    let padding_needed = (layout.align() - (node.ptr as usize % layout.align())) % layout.align();
                    if node.size as usize >= size + padding_needed
                    {
                        // Total space required (size and padding)
                        let space = size + padding_needed;

                        // If there is space left over
                        let new_node = if node.size as usize > space
                        {
                            // Create a new node
                            Some(AllocationNode::new(
                                (node.ptr as usize + space) as *mut u8,
                                node.next,
                                (node.size as usize - space) as u64,
                                AllocationFlags::free() | AllocationFlags::valid()
                            ))
                        }
                        else
                        {
                            None
                        };

                        // Update the current node
                        node.flags.set_taken();
                        node.size = space as u64;
                        node.next = if new_node.is_some() {Some(next_free)} else {node.next};

                        let ptr = (node.ptr as usize + padding_needed) as *mut u8;

                        // If a new node needs to be added, add it
                        if let Some(n) = new_node
                        {
                            *unsafe { next_free.dereference().unwrap() } = n;
                        }

                        // Return the properly padded pointer
                        kdebugln!(ByteMemoryAllocation, " -> 0x{:x} - 0x{:x}", ptr as usize, ptr as usize + layout.size());

                        // Sentinel Write
                        if SENTINEL
                        {
                            for i in 0..size / 2
                            {
                                unsafe {ptr.add(i + layout.size()).write((i & 0xFF) as u8)};
                            }   
                        }

                        break ptr;
                    }
                }

                if let Some(next) = node.next
                {
                    node = unsafe { next.dereference().unwrap() }
                }
                else
                {
                    break 0 as *mut u8;
                }
            }
            else
            {
                break 0 as *mut u8;
            }
        }
    }

    /// Combine a specific node and its successor
    fn combine_specific(&mut self, node: NodePtr, successor: NodePtr)
    {
        let node = unsafe { node.dereference().unwrap() };
        assert_eq!(node.next, Some(successor));
        let successor = unsafe { successor.dereference().unwrap() };
        assert!(node.flags.is_free());
        assert!(node.flags.is_valid());


        assert!(successor.flags.is_free());
        assert!(successor.flags.is_valid());

        node.next = successor.next;
        node.size += successor.size;
        successor.flags.set_invalid();
    }

    /// Walk the table and combine any adjacent nodes which are connected
    fn combine(&mut self)
    {
        let mut current = self.get_node_ptr(0);

        let mut prev = None;

        loop
        {
            let node =  unsafe { current.dereference().unwrap() };
            if node.flags.is_valid()
            {
                let next = node.next;

                if node.flags.is_free()
                {
                    if let Some(prev) = prev
                    {
                        self.combine_specific(prev, current);
                    }
                    else
                    {
                        prev = Some(current);
                    }
                }
                else
                {
                    prev = None;
                }

                if let Some(next) = next
                {
                    current = next;
                }
                else
                {
                    break;
                }
            }
            else
            {
                break;
            }
        }
    }

    /// Deallocate some space with the given layout
    pub fn deallocate(&mut self, ptr: *mut u8, layout: core::alloc::Layout)
    {
        kdebugln!(ByteMemoryAllocation, "Dellocating {} bytes with an alignment of {} bytes at 0x{:x}", layout.size(), layout.align(), ptr as usize);

        // Sentinel Read
        if SENTINEL
        {
            for i in 0..layout.size()
            {
                if unsafe {ptr.add(i + layout.size()).read() != (i & 0xFF) as u8}
                {
                    self.display_node_list();
                    kwarn!("Dellocating {} bytes with an alignment of {} bytes at 0x{:x}", layout.size(), layout.align(), ptr as usize);
                    panic!("Sentinel Triggered at 0x{:x}", ptr as usize);
                }
            }   
        }
        
        let mut current = self.get_node_ptr(0);

        loop
        {
            let node =  unsafe { current.dereference().unwrap() };
            if node.flags.is_valid()
            {
                // Check if the pointer sits within the current node
                if node.ptr as usize <= ptr as usize && node.ptr as usize + node.size as usize > ptr as usize
                {
                    // Then free that node
                    node.flags.set_free();

                    // Combine the nodes
                    self.combine();

                    break;
                }

                if let Some(next) = node.next
                {
                    current = next;
                }
                else
                {
                    break;
                }
            }
            else
            {
                break;
            }
        }
    }
}

/// Initialize the global kernel allocator
pub fn init_kernel_global_allocator(page_count: usize)
{
    kdebugln!(ByteMemoryAllocation, "Initialize the Kernel Global Allocator with {} KBs", page_count * super::PAGE_SIZE / 1024);

    // Insert a new allocation header
    KERNEL_HEAP_POINTER.store(AllocationHeader::new(page_count, 16).unwrap(), core::sync::atomic::Ordering::SeqCst);
}

/// Structure to hold the kernel heap allocator
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

        let ptr = ptr.as_mut().unwrap().allocate(layout);

        ptr
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

pub fn debug_print_layout()
{
    let ptr = KERNEL_HEAP_POINTER.load(core::sync::atomic::Ordering::SeqCst);
    unsafe { ptr.as_mut().unwrap().display_node_list() };
}

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
static GA: GlobalAllocator = GlobalAllocator {};