#![no_std]
#![no_main]
#![feature(panic_info_message, asm, global_asm)]
#![allow(dead_code)]

mod asm;
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

    let ptr = mem::heap::kalloc(1);

    let root = mem::mmu::alloc_table();

    mem::mmu::kvalloc(root, 0x1000_0000, 16, mem::pagetable::EntryBits::Read as usize | mem::pagetable::EntryBits::Write as usize);

    kprintln!("Kernel Start!");

    mem::heap::display_heap_debug_info();

    mem::mmu::kvfree(root, 0x1000_0000, 8);
}
