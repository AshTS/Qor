use lazy_static::lazy_static;

use crate::*;

use super::pages::*;

use core::sync::atomic::AtomicBool;

// Static flag storing a boolean which is true iff the heap has been initialized
// Safety: This must only be set to true during heap initialization
lazy_static!
{
    static ref HEAP_INITIALIZED: AtomicBool = AtomicBool::new(false);
}

/// Get a static reference to the head of the heap (will panic if the heap has
/// not been initialized)
fn get_heap_head() -> &'static mut PageData
{
    if !HEAP_INITIALIZED.load(core::sync::atomic::Ordering::SeqCst)
    {
        panic!("Heap is not initialized, cannot get reference to heap head");
    }

    // Safety: Because of the above check, this will be safe as long as 
    // HEAP_INITIALIZED is treated properly
    unsafe
    {
        (get_heap_start() as *mut PageData).as_mut().unwrap()
    }
}

/// Display information about the heap via kprint
pub fn display_heap_debug_info()
{
    let start = get_heap_start();
    let size = get_heap_size();

    let page_count = size / PAGE_SIZE;

    kprintln!("=============================================");
    kprintln!("Heap Information: ");

    kprintln!("PAGE SIZE:    {} bytes", PAGE_SIZE);
    kprintln!("HEAP START:   0x{:016x}", start);
    kprintln!("HEAP SIZE:    0x{:016x}", size);
    kprintln!("PAGE COUNT:   {} pages", page_count);
    kprintln!("=============================================");
    let free_pages = get_heap_head().get_number_pages();
    kprintln!(" ({}/{}) pages ({}%) Free", free_pages, page_count, free_pages * 100 / page_count);

    let mut heap_ref = get_heap_head();

    loop
    {
        kprintln!("  {}", heap_ref);

        if let Some(next) = heap_ref.next()
        {
            heap_ref = next;
        }
        else
        {
            break;
        }
    }

    kprintln!("=============================================");
}

/// Initialize the heap
pub fn initialize_heap()
{
    let start = get_heap_start();
    
    // Ensure the heap is aligned
    if start % 4096 != 0
    {
        panic!("Heap Start Address `0x{:016x}` not aligned to a page boundary ({} bytes)", start, PAGE_SIZE);
    }

    let page_count = get_heap_size() / PAGE_SIZE;
    PageData::init(start, page_count);

    kprintln!("Heap Initialized");

    HEAP_INITIALIZED.store(true, core::sync::atomic::Ordering::SeqCst);
}

/// Allocate a number of pages on the kernel heap
pub fn kalloc(count: usize) -> *mut u8
{
    let mut prev = get_heap_head();

    if count > prev.get_number_pages()
    {
        panic!("Out of heap space, cannot allocate {} pages", count);
    }

    // The above check allows this to be safe
    let mut ptr = unsafe { prev.get_next_pointer().as_mut() }.unwrap();

    while ptr.get_number_pages() < count
    {
        if let Some(next) = unsafe { ptr.get_next_pointer().as_mut() }
        {
            core::mem::swap(&mut prev, &mut ptr);
            ptr = next;
        }
        else
        {
            panic!("Out of heap space, cannot allocate {} pages", count);
        }
    }

    prev.set_next((ptr.get_current_pointer() as usize + PAGE_SIZE * count) as *mut PageData);

    if ptr.get_number_pages() - count > 0
    {
        PageData::init_not_head(ptr.get_current_pointer() as usize + PAGE_SIZE * count,
                      ptr.get_number_pages() - count,
                           ptr.get_next_pointer());
    }

    ptr.get_current_pointer() as *mut u8
}

/// Free a number of pages back to the kernel heap
/// Safety: The pointer must be to the allocated space, and the count must be
/// accurate
pub unsafe fn kfree(ptr: *mut u8, count: usize)
{
    let mut walking = get_heap_head();

    while !((walking.get_current_pointer() as usize) < (ptr as usize) && (ptr as usize)  < walking.get_next_pointer() as usize)
    {
        if let Some(next) = walking.get_next_pointer().as_mut()
        {
            walking = next;
        }
        else
        {
            break;
        }
    }

    PageData::init_not_head(ptr as usize,
                      count,
                           walking.get_next_pointer());

    walking.set_next(ptr as *mut PageData);

    get_heap_head().defrag();
}

/// Allocate a number of pages on the kernel heap with zeros
pub fn kzalloc(count: usize) -> *mut u8
{
    let ptr = kalloc(count) as *mut u64;

    for offset in 0..(count * PAGE_SIZE / 8)
    {
        // Safety: if the allocator is working properly, this should be safe
        unsafe { ptr.add(offset).write(0) }
    }

    ptr as *mut u8
}