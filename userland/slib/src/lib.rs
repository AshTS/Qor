#![feature(global_asm)]

#![no_std]

mod asm;

/// Panic handler for the kernel
#[panic_handler]
pub fn panic_handler(_info: &core::panic::PanicInfo) -> !
{
    loop {}
}

pub fn write(text: &str)
{
    unsafe { asm::make_syscall(10, text.as_ptr() as usize, 0, 0, 0, 0, 0) };
}

pub fn exit(code: usize) -> !
{
    unsafe { asm::make_syscall(60, code, 0, 0, 0, 0, 0) };
    loop {}
}