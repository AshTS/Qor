// Disable the standard library
#![no_std]
#![no_main]
#![feature(panic_info_message, asm, global_asm)]

mod asm;
mod mmio;
mod uart;

// Macros
#[macro_export]
macro_rules! print
{
    ($($args:tt)+) => ({});
}

#[macro_export]
macro_rules! println
{
    () => ({print!("\r\n")});

    ($fmt:expr) => ({
        print!(concat!($fmt, "\r\n"))
    });

    ($fmt:expr, $($args:tt)+) => ({
        print!(concat!($fmt, "\r\n"), $($args)+)
    });
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> !
{
    print!("Aborting: ");

    if let Some(p) = info.location()
    {
        println!("line {}, file {}: {}", p.line(), p.file(), p.message().unwrap());
    }
    else
    {
        println!("no info available");
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
    
}

