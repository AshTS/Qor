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

    mem::heap::display_heap_debug_info();

    kprintln!("Allocate 5, 64, 32");
    let m0 = mem::heap::kalloc(5);
    let m1 = mem::heap::kzalloc(64);
    let m2 = mem::heap::kalloc(32);

    kprintln!("m0: 0x{:x}", m0 as usize);
    kprintln!("m1: 0x{:x}", m1 as usize);
    kprintln!("m2: 0x{:x}", m2 as usize);

    mem::heap::display_heap_debug_info();

    kprintln!("Free 5");
    unsafe { mem::heap::kfree(m0, 5) };

    mem::heap::display_heap_debug_info();

    kprintln!("Free 32");
    unsafe { mem::heap::kfree(m2, 32) };

    mem::heap::display_heap_debug_info();

    kprintln!("Free 64");
    unsafe { mem::heap::kfree(m1, 64) };

    mem::heap::display_heap_debug_info();


    kprintln!("Kernel Start!");
}
