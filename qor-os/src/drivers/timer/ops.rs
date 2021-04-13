use crate::mmio;

/// Set the time to the next timer interrupt
/// Safety: This performs a raw memory read, so the base address must be a valid timer device
pub unsafe fn set_timer(base: usize, time: usize)
{
    let next_time = get_time(base) + (time as usize);
    let next_time_int = (next_time * 10) as u64;

    mmio::mmio_write_long(base, 0x4000, next_time_int);
}

/// Get the ellapsed time since system start
/// Safety: This performs a raw memory read, so the base address must be a valid timer device
pub unsafe fn get_time(base: usize) -> usize
{
    mmio::mmio_read_long(base, 0xbff8) as usize / 10
}