/// Structure for memory dumps
#[derive(Debug, Clone, Copy)]
pub struct MemoryDump {
    start_address: usize,
    display_address: usize,
    length: usize,
    show_chars: bool,
}

impl MemoryDump {
    /// Construct a new memory dump object
    ///
    /// # Safety
    /// The address range must point to valid memory
    pub unsafe fn new(base_address: usize, length: usize) -> Self {
        Self {
            start_address: base_address,
            display_address: base_address & !15,
            length,
            show_chars: true,
        }
    }

    /// Construct a new memory dump object with a virtual address displayed
    ///
    /// # Safety
    /// The address range must point to valid memory
    pub unsafe fn new_virtual(base_address: usize, virtual_address: usize, length: usize) -> Self {
        Self {
            start_address: base_address,
            display_address: virtual_address & !15,
            length,
            show_chars: true,
        }
    }

    /// Read a byte at the given address, or return None if is it not in the memory bank
    /// 
    /// # Safety
    /// The address range must point to valid memory
    unsafe fn read_byte(&self, addr: usize) -> Option<u8> {
        if addr >= self.start_address && addr < self.start_address + self.length {
            Some((addr as *const u8).read())
        } else {
            None
        }
    }
}

impl core::fmt::Display for MemoryDump {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let start_line = (self.start_address & !15) >> 4;
        let end_line = ((self.start_address + self.length - 1) & !15) >> 4;

        let display_off = self.display_address & !15;

        for line in start_line..=end_line {
            let line = line * 16;
            let display_line = line - start_line * 16 + display_off;
            write!(f, "  {display_line:x} ")?;

            for offset in 0..16 {
                if let Some(v) = unsafe { self.read_byte(line + offset) } {
                    write!(f, "{v:02x} ")?;
                } else {
                    write!(f, "   ")?;
                }

                if offset == 7 {
                    write!(f, " ")?;
                }
            }

            if self.show_chars {
                write!(f, "    |")?;
                for offset in 0..16 {
                    if let Some(v) = unsafe { self.read_byte(line + offset) } {
                        if matches!(v, 32..=127) {
                            write!(f, "{}", v as char)?;
                        } else {
                            write!(f, ".")?;
                        }
                    } else {
                        write!(f, " ")?;
                    }
                }
                write!(f, "|")?;
            }

            if line != end_line * 16 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}
