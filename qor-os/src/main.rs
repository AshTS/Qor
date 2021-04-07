// Disable the standard library
#![no_std]
#![no_main]
#![feature(panic_info_message, asm, global_asm)]

use core::fmt::Write;

use uart::UartDriver;

mod asm;
mod mmio;
mod uart;

lazy_static::lazy_static!
{
    // Safety: The QEMU emulator has a UART mmio interface at 0x1000_0000
    static ref UART_DRIVER: spin::Mutex<UartDriver> = spin::Mutex::new(unsafe{UartDriver::new(0x1000_0000)});
}

// Macros
#[macro_export]
macro_rules! kprint
{
    ($($args:tt)+) => ({
        use core::fmt::Write;
		let _ = write!(UART_DRIVER.lock(), $($args)+);    
    });
}

#[macro_export]
macro_rules! kprintln
{
    () => ({kprint!("\r\n")});

    ($fmt:expr) => ({
        kprint!(concat!($fmt, "\r\n"))
    });

    ($fmt:expr, $($args:tt)+) => ({
        kprint!(concat!($fmt, "\r\n"), $($args)+)
    });
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> !
{
    kprint!("Aborting: ");

    if let Some(p) = info.location()
    {
        kprintln!("line {}, file {}: {}", p.line(), p.file(), info.message().unwrap());
    }
    else
    {
        kprintln!("no info available");
    }

    abort();
}

#[no_mangle]
extern "C"
fn abort() -> !
{
    loop
    {
        unsafe{asm!("wfi")}
    }
}

#[no_mangle]
extern "C"
fn kmain()
{
    kprintln!("Kernel Start!");
}

