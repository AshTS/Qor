/// Get the ellapsed time on the main timer
///
/// Safety: The base address must be a valid base address of the CLINT timer
pub unsafe fn timer_get_time(base_address: usize) -> u64 {
    let address = base_address + 0xbff8;
    let ptr = address as *const u64;

    ptr.read_volatile()
}

/// Set the compare value for the given hardware thread
///
/// Safety: The base address must be a valid base address of the CLINT timer
pub unsafe fn timer_set_cmp(base_address: usize, hart: usize, cmp: u64) {
    let address = base_address + 0x4000;
    let ptr = address as *mut u64;

    ptr.add(hart).write_volatile(cmp);
}

/// Set the duration remaining on the timer for a given hardware thread
///
/// Safety: The base address must be a valid base address of the CLINT timer
pub unsafe fn timer_set_remaining(base_address: usize, hart: usize, duration: u64) {
    timer_set_cmp(base_address, hart, timer_get_time(base_address) + duration)
}
