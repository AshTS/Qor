/// PLIC Interrupt Identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PLICInterrupt(pub u32);

/// PLIC Interrupt Priority
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PLICPriority
{
    Disable = 0,
    Priority1 = 1,
    Priority2 = 2,
    Priority3 = 3,
    Priority4 = 4,
    Priority5 = 5,
    Priority6 = 6,
    Priority7 = 7,
}

/// PLIC Driver
pub struct PLICDriver
{
    base: usize
}

impl PLICDriver
{
    /// Create a new PLIC Driver at the given base address
    /// Safety: The given base address must be a valid PLIC Driver base address
    pub const unsafe fn new(base: usize) -> Self
    {
        Self
        {
            base
        }
    }

    /// Enable an interrupt with the given ID
    pub fn enable(&self, id: PLICInterrupt)
    {
        assert!(id.0 != 0);

        if id.0 >= 32
        {
            unimplemented!("Interrupts with id 32 or greater are not yet implemented");
        }

        let id_raw = (1 << (id.0 % 32)) as u32;

        // Safety: See the safety requirement for this driver's initialization
        unsafe 
        {
            super::mmio::write_offset(self.base, 0x2000, id_raw)
        }
    }

    /// Set the priority for an interrupt
    pub fn set_priority(&self, id: PLICInterrupt, priority: PLICPriority)
    {
        assert!(id.0 != 0);

        if id.0 >= 32
        {
            unimplemented!("Interrupts with id 32 or greater are not yet implemented");
        }

        let priority_value = priority as u8 as u32 & 7;
        
        // Safety: See the safety requirement for this driver's initialization
        unsafe 
        {
            super::mmio::write_offset(self.base, 4 * id.0 as usize, priority_value)
        }
    }

    /// Set the threshold for the PLIC as a whole
    pub fn set_threshold(&self, priority: PLICPriority)
    {
        let threshold_value = priority as u8 as u32 & 7;

        // Safety: See the safety requirement for this driver's initialization
        unsafe 
        {
            super::mmio::write_offset(self.base, 0x20_0000, threshold_value)
        }
    }

    /// Complete an interrupt
    pub fn complete(&self, id: PLICInterrupt)
    {
        assert!(id.0 != 0);

        if id.0 >= 32
        {
            unimplemented!("Interrupts with id 32 or greater are not yet implemented");
        }

        // Safety: See the safety requirement for this driver's initialization
        unsafe 
        {
            super::mmio::write_offset(self.base, 0x20_0004, id.0)
        }
    }

    /// Get the next available interrupt
    pub fn next_interrupt(&self) -> Option<PLICInterrupt>
    {
        // Get the next claimed interrupt
        let next_claimed: PLICInterrupt = unsafe { super::mmio::read_offset(self.base, 0x20_0004) };

        if next_claimed.0 > 0
        {
            Some(next_claimed)
        }
        else
        {
            None
        }
    }
}