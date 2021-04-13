#![no_std]
#![no_main]
#![feature(panic_info_message, global_asm, llvm_asm, asm)]
#![allow(dead_code)]

mod asm;
mod debug;
mod drivers;
mod klib;
mod mem;
mod mmio;
mod panic;

/// Kernel Entry Point
#[no_mangle]
extern "C"
fn kinit()
{
    // Initialize the UART driver so kprint will work and we can start logging
    drivers::init_uart_driver();

    // Initialize the heap
    mem::heap::initialize_heap();

    // Initialize the Global Page Table
    mem::mmu::init_global_page_table();

    // Identity Map the Kernel
    mem::kernel::identity_map_kernel();

    // Initialize the MMU
    mem::kernel::init_mmu();

    // After Returning, we will switch into supervisor mode and go to `kmain`
}

/// Kernel Supervisory Entry Point
#[no_mangle]
extern "C"
fn kmain()
{
    kprintln!("Kernel Start");
}