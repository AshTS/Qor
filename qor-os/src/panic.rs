//! Panic Implementation

/// Panic handler for the kernel
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> !
{
    abort();
}

/// Terminate execution by waiting in a loop
#[no_mangle]
extern "C"
fn abort() -> !
{
    loop
    {
        unsafe { riscv::asm::wfi() };
    }
}