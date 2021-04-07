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

    kprintln!("Kernel Start!");
}
