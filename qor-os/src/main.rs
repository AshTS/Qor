#![no_std]
#![no_main]
#![feature(panic_info_message, asm, global_asm, llvm_asm)]
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
fn kinit() -> usize
{
    // Initialize the UART driver so kprint will work and we can start logging
    drivers::init_uart_driver();

    // Initialize the heap
    mem::heap::initialize_heap();

    // Initialize the Global Page Table
    mem::mmu::init_global_page_table();

    // Identity Map the Kernel
    mem::kernel::identity_map_kernel();

    kprintln!("0x{:x} -> 0x{:x}", 0x800035e6 as usize, mem::mmu::virt_to_phys(0x800035e6).unwrap());

    mem::kernel::init_mmu()
}

/// Kernel Supervisory Entry Point
#[no_mangle]
extern "C"
fn kmain()
{
    kprintln!("Kernel Start");
}