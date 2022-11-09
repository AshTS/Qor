// Required features
#![feature(alloc_error_handler)] // Allow custom allocator
#![feature(const_option)] // Allow constant unwraps
#![feature(custom_test_frameworks)] // Allow cargo test
#![feature(panic_info_message)] // For panic messages
#![feature(ptr_internals)] // For pointer types
#![feature(fn_align)]
// To allow functions to be forced to a 4 byte boundary

// Use the default allocation error handler
#![feature(default_alloc_error_handler)]
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
extern crate alloc;

use libutils::{sync::{InitThreadMarker, NoInterruptMarker}};

// Includes
mod asm;
mod debug;
mod drivers;
mod halt;
#[macro_use]
mod kprint;
mod mem;
mod panic;
mod process;
mod tasks;
mod test;
mod trap;

/// Kernel Initialize Function (Called immediately after boot)
#[no_mangle]
#[repr(align(4))]
pub extern "C" fn kinit() {
    // Safety: we can construct the `InitThreadMarker` since we are the init thread
    let thread_marker = unsafe { InitThreadMarker::new() };

    // Safety: we can construct a 'NoInterruptMaker' since we are in the init thread and interrupts are disabled for early kernel boot
    let interrupt_marker = unsafe { NoInterruptMarker::new() };

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

    // Initialize the global allocator
    mem::GLOBAL_ALLOCATOR.initialize(
        thread_marker,
        interrupt_marker,
        mem::KiByteCount::new(2048).convert(),
    );
    kdebugln!(
        thread_marker,
        Initialization,
        "Global Allocator Initialized"
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
    mem::set_page_table(&page_table);

    page_table.leak();

    // Initialize a trap frame
    kdebugln!(thread_marker, Initialization, "Initializing Trap Frame");
    trap::initialize_trap_frame(interrupt_marker);

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
    kprintln!(unsafe "Hello World!");

    let mut executor = tasks::SimpleExecutor::new();
    executor.spawn(tasks::Task::new(example_task()));
    executor.run();

    drivers::PLIC_DRIVER.enable_with_priority(
        drivers::interrupts::UART_INTERRUPT,
        drivers::InterruptPriority::Priority7,
    );
    drivers::PLIC_DRIVER.set_threshold(drivers::InterruptPriority::Priority1);

    drivers::CLINT_DRIVER.set_remaining(0, 10_000_000);

    let p = process::Process::from_raw(asm::init_proc_location, mem::PageCount::new(1));

    process::add_process(p);
}

async fn example_task() {
    let driver = drivers::virtio_device_collection();

    let mut buf = alloc::boxed::Box::new([0u8; 4096 * 8]);

    let mut d = driver.block_devices[0].spin_lock();

    if let Some(v) = unsafe { d.async_read(buf.as_mut_ptr(), 4096 * 8, 0x200) } {
        v.await;
        kdebugln!(unsafe "{:?}", buf);
    }
}