#![no_std]
#![no_main]
#![feature(panic_info_message, global_asm, llvm_asm, asm, alloc_prelude, alloc_error_handler)]
#![allow(dead_code)]

mod asm;
mod debug;
mod drivers;
mod klib;
mod mem;
mod mmio;
mod panic;
mod trap;

extern crate alloc;
use alloc::prelude::v1::*;

/// Kernel Entry Point
#[no_mangle]
extern "C"
fn kinit()
{
    // Initialize the UART driver so kprint will work and we can start logging
    drivers::init_uart_driver();

    // Initialize the page heap
    mem::heap::initialize_heap();

    // Initialize the Global Page Table
    mem::mmu::init_global_page_table();

    // Map space for the trap frame
    trap::init_trap_frame();

    // Identity Map the Kernel
    mem::kernel::identity_map_kernel();

    // Initialize the MMU
    mem::kernel::init_mmu();

    // Initialize the kernel byte heap with 64 4KB pages (256 KB)
    mem::alloc::init_kernel_heap(64);

    // After Returning, we will switch into supervisor mode and go to `kmain`
}

/// Kernel Supervisory Entry Point
#[no_mangle]
extern "C"
fn kmain()
{
    kprintln!("Kernel Start");
    kprintln!("Initializing PLIC");

    drivers::PLIC_DRIVER.set_threshold(drivers::plic::PLICPriority::Priority0);
    drivers::PLIC_DRIVER.enable_interrupt(drivers::plic::PLICInterrupt::Interrupt10);
    drivers::PLIC_DRIVER.set_priority(drivers::plic::PLICInterrupt::Interrupt10, drivers::plic::PLICPriority::Priority1);
}