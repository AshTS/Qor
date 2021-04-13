//! MMIO Helper functions (all of which are unsafe)

/// Write a u8 to the volatile address: address + offset 
/// Safety: This is performing raw memory access
pub unsafe fn mmio_write_byte(address: usize, offset: usize, value: u8)
{
    (address as *mut u8).add(offset).write_volatile(value);
}

/// Read a u8 from the volatile address: address + offset
/// Safety: This is performing raw memory access
pub unsafe fn mmio_read_byte(address: usize, offset: usize) -> u8
{
    (address as *mut u8).add(offset).read_volatile()
}

/// Write a u16 to the volatile address: address + offset 
/// Safety: This is performing raw memory access
pub unsafe fn mmio_write_short(address: usize, offset: usize, value: u16)
{
    (address as *mut u16).add(offset >> 1).write_volatile(value);
}

/// Read a u16 from the volatile address: address + offset
/// Safety: This is performing raw memory access
pub unsafe fn mmio_read_short(address: usize, offset: usize) -> u16
{
    (address as *mut u16).add(offset >> 1).read_volatile()
}

/// Write a u32 to the volatile address: address + offset 
/// Safety: This is performing raw memory access
pub unsafe fn mmio_write_int(address: usize, offset: usize, value: u32)
{
    (address as *mut u32).add(offset >> 2).write_volatile(value);
}

/// Read a u32 from the volatile address: address + offset
/// Safety: This is performing raw memory access
pub unsafe fn mmio_read_int(address: usize, offset: usize) -> u32
{
    (address as *mut u32).add(offset >> 2).read_volatile()
}

/// Write a u64 to the volatile address: address + offset 
/// Safety: This is performing raw memory access
pub unsafe fn mmio_write_long(address: usize, offset: usize, value: u64)
{
    (address as *mut u64).add(offset >> 3).write_volatile(value);
}

/// Read a u64 from the volatile address: address + offset
/// Safety: This is performing raw memory access
pub unsafe fn mmio_read_long(address: usize, offset: usize) -> u64
{
    (address as *mut u64).add(offset >> 3).read_volatile()
}