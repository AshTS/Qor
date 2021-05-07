#![no_std]
#![no_main]
#![feature(panic_info_message, global_asm, llvm_asm, asm, alloc_prelude, alloc_error_handler, map_first_last, option_insert, array_methods)]
#![allow(dead_code)]

mod asm;
mod debug;
mod drivers;
mod elf;
mod fs;
mod klib;
mod mem;
mod mmio;
mod panic;
mod process;
mod syscall;
mod trap;

extern crate alloc;
use alloc::prelude::v1::*;
use alloc::vec;

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

    // Initialize the platform level interrupt controller
    drivers::init_plic_driver();

    // Initialize the virtio drivers (including the block device driver)
    drivers::virtio::probe_virt_io();

    // Initialize the virtio interrtupts
    drivers::init_virtio_interrupts();

    // Initialzize the process manager
    process::init_process_manager();
    
    // Create the file system interface
    let mut interface = fs::FileSystemInterface::new(0);

    if let Err(e) = interface.initialize()
    {
        panic!("Unable to initialize file system: `{}`", e.msg);
    }

    let data = interface.get_inode(10);

    let mut buffer = Box::new(vec![0u8; data.size as usize]);

    interface.read_inode(data, &mut *buffer.as_mut_slice(), data.size as usize);
    
    let data = match elf::load_elf(&buffer)
    {
        Err(e) => { panic!("Unable to load Elf: `{}`", e.msg); },
        Ok(data) => { data}
    };

    kprintln!("Adding Process With PID: {}", process::get_process_manager().unwrap().add_process(data));

    drivers::TIMER_DRIVER.set_remaining_time(1);
}