// Required features
#![feature(alloc_error_handler)] // Allow custom allocator
#![feature(const_option)] // Allow constant unwraps
#![feature(custom_test_frameworks)] // Allow cargo test
#![feature(panic_info_message)] // For panic messages
#![feature(ptr_internals)] // For pointer types
#![feature(fn_align)]
// To allow functions to be forced to a 4 byte boundary

// Allow dead code for partial implementations
#![allow(dead_code)]
// Use the _start symbol instead of main
#![no_main]
// Do not link against a standard library (the core and alloc crates will handle
// anything we would do with the standard library)
#![no_std]
// Set the test runner for the custom test framework
#![test_runner(crate::test::test_runner)]
// Get the test main so it can be run after initialization
#![reexport_test_harness_main = "test_main"]

// Alloc Prelude

use drivers::virtio::driver;
use libutils::sync::no_interrupts;
extern crate alloc;

// Includes
mod asm;
mod debug;
mod drivers;
mod errno;
mod fs;
mod halt;
#[macro_use]
mod kprint;
mod mem;
mod panic;
mod process;
mod tasks;
mod test;
mod trap;
mod types;

/// Kernel Initialize Function (Called immediately after boot)
#[no_mangle]
#[repr(align(4))]
pub extern "C" fn kinit() {
    // Safety: we can construct the `InitThreadMarker` since we are the init thread
    let thread_marker = unsafe { libutils::sync::InitThreadMarker::new() };

    // Safety: we can construct a 'NoInterruptMaker' since we are in the init thread and interrupts are disabled for early kernel boot
    let interrupt_marker = unsafe { libutils::sync::NoInterruptMarker::new() };

    // Initialize the UART driver
    drivers::UART_DRIVER.init(thread_marker);
    kdebugln!(thread_marker, Initialization, "UART Driver Initialized");

    // Initialize the page allocator
    mem::PAGE_ALLOCATOR
        .initialize(thread_marker, interrupt_marker)
        .expect("Unable to initialize GPA");
    kdebugln!(
        thread_marker,
        Initialization,
        "Global Page Allocator Initialized"
    );

    // Initialize the global page table
    kdebugln!(
        thread_marker,
        Initialization,
        "Initializing Global Page Table"
    );
    let mut page_table = mem::PAGE_ALLOCATOR
        .allocate(interrupt_marker, mem::PageTable::new())
        .expect("Unable to allocate Global Page Table");
    mem::identity_map_kernel(&mut page_table);
    mem::set_page_table(page_table);

    // Initialize a trap frame
    kdebugln!(thread_marker, Initialization, "Initializing Trap Frame");
    trap::initialize_trap_frame(interrupt_marker, 0);

    // Initialize the global allocator
    mem::GLOBAL_ALLOCATOR.initialize(
        thread_marker,
        interrupt_marker,
        mem::KiByteCount::new(8192).convert(),
        0
    );
    kdebugln!(
        thread_marker,
        Initialization,
        "Global Allocator Initialized"
    );

    // Initialize the process map
    kdebugln!(thread_marker, Initialization, "Initializing Process Map");
    process::init_process_map(thread_marker);

    // Discover virtio devices
    kdebugln!(thread_marker, Initialization, "VirtIO Device Discovery");
    drivers::virtio_device_discovery(thread_marker).unwrap();
}

/// Kernel Main Function (Called in supervisor mode)
#[no_mangle]
#[repr(align(4))]
pub extern "C" fn kmain() {
    // Safety: we can construct the `InitThreadMarker` since we are the init thread
    let thread_marker = unsafe { libutils::sync::InitThreadMarker::new() };

    kdebugln!(thread_marker, Initialization, "Switch to Supervisor Mode");

    // Initialize the global filesystem
    kdebugln!(
        thread_marker,
        Initialization,
        "Initializing Virtual Filesystem"
    );
    fs::init_global_filesystem(thread_marker);

    // Mount the boot filesystem
    tasks::execute_task(mount_filesystem());

    // Initialize the global executor
    kdebugln!(
        thread_marker,
        Initialization,
        "Initializing Global Executor"
    );
    tasks::init_global_executor(thread_marker);

    // Setup the PLIC timer
    kdebugln!(thread_marker, Initialization, "Initialize PLIC Timer");
    drivers::PLIC_DRIVER.enable_with_priority(
        drivers::interrupts::UART_INTERRUPT,
        drivers::InterruptPriority::Priority7,
    );
    drivers::PLIC_DRIVER.set_threshold(drivers::InterruptPriority::Priority1);

    // Spawn cleanup process
    kdebugln!(thread_marker, Initialization, "Spawning Cleanup Process");
    let p = process::Process::from_raw(asm::init_proc_location, mem::PageCount::new(1));
    process::add_process(p);

    // Start the context switch timer
    kdebugln!(
        thread_marker,
        Initialization,
        "Starting Process Switch Timer"
    );
    // drivers::CLINT_DRIVER.set_remaining(0, 10);

    // We can now let the other threads know they can start safely
    WAITING_FLAG.store(true, atomic::Ordering::Relaxed);
}

/// Mount the boot filesystem
async fn mount_filesystem() {
    use fs::FileSystem;

    // Get the driver from the virtio collection
    let driver = drivers::virtio_device_collection().block_devices[0].clone();

    // Construct a buffered block device wrapper
    let buffer = crate::drivers::BlockDeviceBuffer::<1024, _, _, _>::new(driver);

    // Access the virtual file system and acquire the lock on it
    let vfs_arc = fs::global_filesystem_arc();
    let mut vfs = vfs_arc.async_lock().await;

    // Construct and initialize the Minix3 filesystem
    kdebugln!(unsafe "Initializing Minix3 Filesystem");
    let mut minix3 = fs::minix3::Minix3Filesystem::new(buffer);
    minix3.init().await.unwrap();

    // Mount and index the filesystem
    kdebugln!(unsafe "Mounting Minix3 Filesystem");
    vfs.mount_fs("/".into(), alloc::boxed::Box::new(minix3))
        .await
        .unwrap();
    vfs.index().await.unwrap();
}

#[no_mangle]
#[used]
pub static WAITING_FLAG: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

/// Kernel Initialize Function (Called immediately after boot)
#[no_mangle]
#[repr(align(4))]
pub extern "C" fn kinit2(hart: usize) {
    kprintln!(unsafe "Hello From Thread ID{}", hart);
    
    // Initialize a trap frame
    kdebugln!(unsafe Initialization, "Initializing Trap Frame for TID{}", hart);
    no_interrupts(|interrupt_marker| {
        trap::initialize_trap_frame(interrupt_marker, hart);
    });

    // Initialize the global allocator
    mem::GLOBAL_ALLOCATOR.initialize(
        unsafe { libutils::sync::InitThreadMarker::new() },
        unsafe { libutils::sync::NoInterruptMarker::new() },
        mem::KiByteCount::new(8192).convert(),
        hart
    );
}

/// Kernel Main Function
#[no_mangle]
#[repr(align(4))]
pub extern "C" fn kmain2(hart: usize) {
    kprintln!(unsafe "Hello Again From Thread ID{}", hart);
    drivers::CLINT_DRIVER.set_remaining(hart, 10);
    drivers::CLINT_DRIVER.set_remaining(0, 10);
}