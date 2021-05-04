#![no_std]
#![no_main]

/// Panic handler for the kernel
#[panic_handler]
pub fn panic_handler(info: &core::panic::PanicInfo) -> !
{
    loop {}
}