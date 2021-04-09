#![no_std]
#![no_main]
#![feature(panic_info_message, asm, global_asm)]
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
fn kmain()
{
    // Initialize the UART driver so kprint will work and we can start logging
    drivers::init_uart_driver();

    // Initialize the heap
    mem::heap::initialize_heap();

    // Initialize the Global Page Table
    mem::mmu::init_global_page_table();

    mem::kvalloc(0x42000, 16, 
        mem::EntryBits::Read as usize |
                mem::EntryBits::Write as usize);

    mem::heap::display_heap_debug_info();

    kprintln!("Kernel Start");
}
