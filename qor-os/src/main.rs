// Required features
// #![feature(alloc_error_handler)] // Allow custom allocator
#![feature(const_option)] // Allow constant unwraps
#![feature(custom_test_frameworks)] // Allow cargo test
#![feature(panic_info_message)] // For panic messages
#![feature(ptr_internals)] // For pointer types
#![feature(fn_align)]
// To allow functions to be forced to a 4 byte boundary
#![feature(slice_split_at_unchecked)] // More efficient splitting
#![feature(strict_provenance)] // Allow more explicit pointer manipulations
#![feature(strict_provenance_atomic_ptr)]

#![feature(ptr_sub_ptr)] // For pointer manipulations to create more ergonomic static construction of allocators
#![feature(const_ptr_sub_ptr)]
#![feature(slice_from_ptr_range)]

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

// extern crate alloc;

// Includes
mod asm;
mod debug;
mod drivers;
mod errno;
mod halt;
mod harts;
#[macro_use]
mod kprint;
mod mem;
mod panic;
#[cfg(test)]
mod test;

/// Kernel Initialize Function (Called immediately after boot)
#[no_mangle]
#[repr(align(4))]
pub extern "C" fn kinit() {
    // We get the init thread marker until we allow the other harts to start at the end of this function because we are the only thread that will be running
    let init_thread_marker = unsafe { libutils::sync::InitThreadMarker::new() };

    // We get the no interrupts marker because within the init function, we have interrupts disabled
    let _no_interrupt_marker = unsafe { libutils::sync::NoInterruptMarker::new() };

    // Initialize the UART Driver
    drivers::UART_DRIVER.init(&init_thread_marker);
    kdebugln!(&init_thread_marker, "Initialized UART Driver");

    // Run any tests if testing is requested
    #[cfg(test)]
    test_main();

    // Initialize the Kernel Static Page Bump Allocator
    kdebugln!(&init_thread_marker, Initialization, "Initialize Page Bump Allocator");
    mem::initialize_kernel_bump_allocator();

    // At the end of the kinit function, we can allow the other harts to begin running, here we destroy the `init_thread_marker`
    kdebugln!(&init_thread_marker, Initialization, "Enabling Secondary Harts");
    harts::enable_other_harts(init_thread_marker);

    // Synchronize the other harts
    harts::machine_mode_sync();

    #[cfg(test)]
    test::sync_test_runner();

    #[cfg(test)]
    crate::drivers::POWER_DRIVER.shutdown_success();

    loop { core::hint::spin_loop() }
}

/// Kernel Main Function (Called in supervisor mode)
#[no_mangle]
#[repr(align(4))]
pub extern "C" fn kmain() {
}

/// Kernel Initialize Function (Called immediately after boot)
#[no_mangle]
#[repr(align(4))]
pub extern "C" fn kinit2() {
    harts::machine_mode_sync();
    
    #[cfg(test)]
    test::sync_test_runner();

    loop { core::hint::spin_loop() }
}

/// Kernel Main Function (Called in supervisor mode)
#[no_mangle]
#[repr(align(4))]
pub extern "C" fn kmain2() {
}

/// Trap Handler
#[no_mangle]
#[repr(align(4))]
pub extern "C" fn m_trap() {
}

#[cfg(test)]
pub fn sync_test_a() {
    assert_eq!(2, 2);
}