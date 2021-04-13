use crate::mmio;

/// Enable the interrupt with the given ID
/// Safety: The base address must be a valid pointer to a PLIC MMIO device
pub unsafe fn enable_interrupt(base: usize, id: u8)
{
    assert!(id < 32);

    let previous = mmio::mmio_read_int(base, 0x2000);

    mmio::mmio_write_int(base, 0x2000, previous | (1 << (id as u32)));
}

/// Set the given interrupt's priority to the given priority
/// Safety: The base address must be a valid pointer to a PLIC MMIO device
pub unsafe fn set_priority(base: usize, id: u8, priority: u8)
{
    assert!(priority < 8);

    mmio::mmio_write_int(base, 4 * id as usize, priority as u32);
}

/// Set the global threshold to the given threshold
/// Safety: The base address must be a valid pointer to a PLIC MMIO device
pub unsafe fn set_threshold(base: usize, threshold: u8)
{
    assert!(threshold < 8);

    mmio::mmio_write_int(base, 0x20_0000, threshold as u32);
}

/// Get the next interrupt
/// Safety: The base address must be a valid pointer to a PLIC MMIO device
pub unsafe fn next(base: usize) -> Option<u32>
{
    let value = mmio::mmio_read_int(base, 0x20_0004);

    if value == 0
    {
        None
    }
    else
    {
        Some(value)
    }
}

/// Complete a pending interrupt
/// Safety: The base address must be a valid pointer to a PLIC MMIO device
pub unsafe fn complete(base: usize, id: u32)
{
    mmio::mmio_write_int(base, 0x20_0004, id);
}