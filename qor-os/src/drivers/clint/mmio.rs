#[derive(Debug, Clone, Copy)]
pub struct MMIOCoreLevelInterruptor {
    base_address: usize,
}

impl MMIOCoreLevelInterruptor {
    /// Statically construct the CLINT Driver from the `base_address`
    ///
    /// Safety: The base address given must be a base address for an MMIO CLINT driver
    pub const unsafe fn new(base_address: usize) -> Self {
        Self { base_address }
    }

    pub fn get_time(&self) -> u64 {
        // Safety: Per the constructor safety requirements, the base address is valid
        unsafe { crate::drivers::clint::raw::timer_get_time(self.base_address) }
    }

    pub fn set_remaining(&self, hart: usize, remaining: u64) {
        // Safety: Per the constructor safety requirements, the base address is valid
        unsafe {
            crate::drivers::clint::raw::timer_set_remaining(self.base_address, hart, remaining)
        }
    }

    pub fn set_compare(&self, hart: usize, compare: u64) {
        // Safety: Per the constructor safety requirements, the base address is valid
        unsafe { crate::drivers::clint::raw::timer_set_cmp(self.base_address, hart, compare) }
    }
}
