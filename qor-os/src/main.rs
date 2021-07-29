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
use alloc::*;
use alloc::prelude::v1::*;

// Includes
mod asm;
mod debug;
mod drivers;
mod fs;
mod mem;
mod kprint;
mod panic;
mod process;
mod resources;
mod syscalls;
mod test;
mod trap;
mod utils;

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
    mem::alloc::init_kernel_global_allocator(256);
    kdebugln!(Initialization, "Global Kernel Byte Allocator Initialized");

    // Identity map the kernel
    mem::identity_map_kernel();
    kdebugln!(Initialization, "Identity Mapped Kernel");

    // Set up the trap frame
    trap::init_trap_frame();
    kdebugln!(Initialization, "Trap Frame Initialized");
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
    
    // Initialize the Process Manager
    process::scheduler::init_process_manager();
    kdebugln!(Initialization, "Process Manager Initialized");

    // Enumerate the virtio drivers
    drivers::virtio::probe_virtio_address_space();
    kdebugln!(Initialization, "VirtIO Devices Enumerated");

    // Enumerate the virtio drivers
    kdebugln!(Initialization, "Initialize VirtIO Devices");
    drivers::virtio::initialize_virtio_devices();
    kdebugln!(Initialization, "VirtIO Devices Initialized");

    // Initialize the virtio interrtupts
    drivers::virtio::init_virtio_interrupts();
    kdebugln!(Initialization, "VirtIO Interrupts Initialized");

    // Test the GPU
    let driver = drivers::virtio::get_gpu_driver(0).unwrap();

    driver.init();

    for i in 0..16
    {
        driver.write_glpyh(&resources::fonts::vga::GLYPHS['#' as u8 as usize], i, 0, resources::colors::ega::EGA_COLORS[i], resources::colors::ega::EGA_COLORS[0])
    }
    
    driver.invalidate(0, 0, 640, 480);

    let mut vfs = fs::vfs::FilesystemInterface::new();
    let mut disk0 = fs::minix3::Minix3Filesystem::new(0);

    use fs::fstrait::Filesystem;
    use libutils::paths::OwnedPath;

    vfs.init().unwrap();
    disk0.init().unwrap();

    vfs.mount_fs(&OwnedPath::new("/"), Box::new(disk0)).unwrap();

    vfs.index().unwrap();

    let mut elf_proc = process::elf::load_elf(&mut vfs, &OwnedPath::new("/bin/shell")).unwrap();
    process::scheduler::get_init_process_mut().unwrap().register_child(elf_proc.pid);
    elf_proc.connect_to_term();
    process::scheduler::add_process(elf_proc);

    // Start the timer
    drivers::init_timer_driver(1000);
    kdebugln!(Initialization, "Timer Started");
}