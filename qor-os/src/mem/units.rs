/// Memory Unit Generic Type, stores the size of a memory region as a number of a particular size of units
pub struct MemoryUnit<const Scale: usize>(usize);

/// Memory Unit for Pages
pub type PageCount = MemoryUnit<{super::PAGE_SIZE}>;

/// Memory Unit for KiBytes
pub type KiByteCount = MemoryUnit<1024>;

/// Memory Unit for Bytes
pub type ByteCount = MemoryUnit<1>;

impl<const Scale: usize> MemoryUnit<Scale> {
    /// Construct the type from a particular number of units
    pub const fn new(units: usize) -> Self {
        Self(units)
    }

    /// Get the raw number of units
    pub const fn raw(&self) -> usize {
        self.0
    }

    /// Get the raw number of bytes
    pub const fn raw_bytes(&self) -> usize {
        Scale * self.0
    }

    /// Get a mutable reference to the number of bytes
    pub fn mut_raw(&mut self) -> &mut usize {
        &mut self.0
    }
}

impl<const ScaleSrc: usize> MemoryUnit<ScaleSrc> {
    pub const fn convert<const ScaleDest: usize>(&self) -> MemoryUnit<ScaleDest> {
        MemoryUnit((self.raw_bytes() + ScaleDest - 1) / ScaleDest)
    }
}
