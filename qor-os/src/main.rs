// Required features
// #![feature(alloc_error_handler)] // Allow custom allocator
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

// extern crate alloc;

// Includes
mod asm;
mod debug;
mod drivers;
mod errno;
mod halt;
#[macro_use]
mod kprint;
mod panic;
mod test;

/// Kernel Initialize Function (Called immediately after boot)
#[no_mangle]
#[repr(align(4))]
pub extern "C" fn kinit() {
    kprintln!(unsafe "Hello World!");
    loop {}
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
    kprintln!(unsafe "Hello World!");
    loop {}
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