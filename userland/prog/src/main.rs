#![no_std]
#![no_main]

/// Panic handler for the kernel
#[panic_handler]
pub fn panic_handler(_info: &core::panic::PanicInfo) -> !
{
    loop {}
}


#[no_mangle]
extern "C"
fn _start()
{
    slib::exit(5);
    loop {}
}