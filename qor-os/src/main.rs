// Required features
#![feature(alloc_error_handler)]        // Allow custom allocator
#![feature(alloc_prelude)]              // Allocation prelude
#![feature(asm)]                        // For testing processes (ecall)
#![feature(const_option)]               // Allow constant unwraps
#![feature(custom_test_frameworks)]     // Allow cargo test
#![feature(global_asm)]                 // For assembly file compilation
#![feature(panic_info_message)]         // For panic messages

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
use alloc::prelude::v1::*;

// Includes
mod asm;
mod debug;
mod drivers;
mod mem;
mod kprint;
mod panic;
mod process;
mod syscalls;
mod test;
mod trap;

/// Kernel Initialize Function (Called immediately after boot)
#[no_mangle]
pub extern "C"
fn kinit()
{
    // Initialize the UART driver
    drivers::init_uart_driver();
    kdebugln!(Initialization, "UART Driver Initialized");

    // Initialize the global kernel page allocator
    mem::init_kernel_page_allocator();
    kdebugln!(Initialization, "Global Kernel Page Allocator Initialized");
    
    // Run any tests if testing is requested
    #[cfg(test)]
    test_main();

    // Initialize the kernel heap
    mem::alloc::init_kernel_global_allocator(64);
    kdebugln!(Initialization, "Global Kernel Byte Allocator Initialized");

    // Identity map the kernel
    mem::identity_map_kernel();
    kdebugln!(Initialization, "Identity Mapped Kernel");

    // Set up the trap frame
    trap::init_trap_frame();
    kdebugln!(Initialization, "Trap Frame Initialized");
}

pub fn init_proc()
{
    loop {}
}

pub fn test_proc()
{
    let mut i = 0;
    let mut j = 0;

    while j < 10
    {
        while i < 50_000_000
        {
            i += 1;
        }

        i = 0;
        j += 1;

        unsafe { asm!("ecall") };
    }

    unsafe { asm!("li a0, 60") };
    unsafe { asm!("li a1, 42") };
    unsafe { asm!("ecall") };
    loop {};
}

pub fn test_proc2()
{
    let mut i = 0;

    loop
    {
        while i < 10_000_000
        {
            i += 1;
        }

        i = 0;

        unsafe { asm!("ecall") };
    }
}


/// Kernel Main Function (Called in supervisor mode)
#[no_mangle]
pub extern "C"
fn kmain()
{
    kdebugln!(Initialization, "Started Supervisor Mode");

    // Initialize the PLIC
    drivers::init_plic_driver();
    kdebugln!(Initialization, "PLIC Driver Initialized");
    
    process::scheduler::init_process_manager();

    let process = process::process::Process::from_fn_ptr(init_proc);
    process::scheduler::add_process(process);

    let process = process::process::Process::from_fn_ptr(test_proc);
    process::scheduler::add_process(process);

    let process = process::process::Process::from_fn_ptr(test_proc2);
    process::scheduler::add_process(process);

    // Start the timer
    drivers::init_timer_driver(1000);
    kdebugln!(Initialization, "Timer Started");
}