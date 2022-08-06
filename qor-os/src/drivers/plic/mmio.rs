use crate::drivers::InterruptPriority;

use super::InterruptID;

/// MMIO Driver for the PLIC
#[derive(Debug, Clone)]
pub struct MMIOPlatformLevelInterruptController
{
    base_address: usize
}

impl MMIOPlatformLevelInterruptController
{
    /// Statically construct the PLIC Driver from the `base_address`
    /// 
    /// Safety: The base address given must be a base address for an MMIO PLIC driver
    pub const unsafe fn new(base_address: usize) -> Self
    {
        Self
        {
            base_address,
        }
    }

    /// Enable an interrupt
    pub fn enable_interrupt(&self, interrupt: InterruptID)
    {
        // Safety: Per the constructor safety requirements, the base address is valid
        unsafe
        {
            crate::drivers::plic::raw::enable_interrupt_id(self.base_address, interrupt);
        }
    }

    /// Disable an interrupt
    pub fn disable_interrupt(&self, interrupt: InterruptID)
    {
        // Safety: Per the constructor safety requirements, the base address is valid
        unsafe
        {
            crate::drivers::plic::raw::disable_interrupt_id(self.base_address, interrupt);
        }
    }

    /// Set the priority of an interrupt
    pub fn set_priority(&self, interrupt: InterruptID, priority: InterruptPriority)
    {
        // Safety: Per the constructor safety requirements, the base address is valid
        unsafe
        {
            crate::drivers::plic::raw::set_interrupt_priority(self.base_address, interrupt, priority);
        }
    }

    /// Set the threshold for the PLIC
    pub fn set_threshold(&self, priority: InterruptPriority)
    {
        // Safety: Per the constructor safety requirements, the base address is valid
        unsafe
        {
            crate::drivers::plic::raw::set_threshold(self.base_address, priority);
        }
    }

    /// Get the next interrupt from the PLIC
    pub fn next_interrupt(&self) -> Option<InterruptID>
    {
        // Safety: Per the constructor safety requirements, the base address is valid
        unsafe
        {
            crate::drivers::plic::raw::next_interrupt(self.base_address)
        }
    }

    /// Claim an interrupt
    pub fn claim_interrupt(&self, interrupt: InterruptID)
    {
        // Safety: Per the constructor safety requirements, the base address is valid
        unsafe
        {
            crate::drivers::plic::raw::claim_interrupt(self.base_address, interrupt);
        }
    }

    /// Enable an interrupt and set its priority
    pub fn enable_with_priority(&self, interrupt: InterruptID, priority: InterruptPriority)
    {
        self.enable_interrupt(interrupt);
        self.set_priority(interrupt, priority);
    }
}