// Required features
#![feature(alloc_error_handler)]        // Allow custom allocator
#![feature(const_option)]               // Allow constant unwraps
#![feature(custom_test_frameworks)]     // Allow cargo test
#![feature(panic_info_message)]         // For panic messages
#![feature(ptr_internals)]              // For pointer types

// Use the default allocation error handler
// #![feature(default_alloc_error_handler)]

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
// use alloc::*;

use libutils::sync::{InitThreadMarker, NoInterruptMarker};

// Includes
mod asm;
mod debug;
mod drivers;
mod halt;
#[macro_use]
mod kprint;
mod mem;
mod panic;
mod test;

/// Kernel Initialize Function (Called immediately after boot)
#[no_mangle]
pub extern "C"
fn kinit()
{
    // Safety: we can construct the `InitThreadMarker` since we are the init thread
    let thread_marker = unsafe { InitThreadMarker::new() };

    // Safety: we can construct a 'NoInterruptMaker' since we are in the init thread and interrupts are disabled for early kernel boot
    let interrupt_marker = unsafe { NoInterruptMarker::new() };

    // Initialize the UART driver
    drivers::UART_DRIVER.init(thread_marker);
    kdebugln!(thread_marker, Initialization, "UART Driver Initialized");

    // Initialize the page allocator
    mem::PAGE_ALLOCATOR.initialize(thread_marker, interrupt_marker).expect("Unable to initialize GPA");
    kdebugln!(thread_marker, Initialization, "Global Page Allocator Initialized");
}

/// Kernel Main Function (Called in supervisor mode)
#[no_mangle]
pub extern "C"
fn kmain()
{

}