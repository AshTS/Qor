#![feature(global_asm)]

#![no_std]

mod asm;

pub fn exit(code: usize)
{
    unsafe { asm::make_syscall(60, code, 0, 0, 0, 0, 0) };
}