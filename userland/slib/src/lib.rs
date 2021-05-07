#![feature(global_asm)]

#![no_std]

mod asm;

/// Panic handler for the kernel
#[panic_handler]
pub fn panic_handler(_info: &core::panic::PanicInfo) -> !
{
    loop {}
}

pub fn exit(code: usize) -> !
{
    unsafe { asm::make_syscall(60, code, 0, 0, 0, 0, 0) };
    loop {}
}