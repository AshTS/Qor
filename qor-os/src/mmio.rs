//! MMIO Helper functions (all of which are unsafe)

use core::u8;

pub unsafe fn mmio_write_byte(address: usize, offset: usize, value: u8)
{
    (address as *mut u8).add(offset).write_volatile(value);
}

pub unsafe fn mmio_read_byte(address: usize, offset: usize) -> u8
{
    (address as *mut u8).add(offset).read_volatile()
}