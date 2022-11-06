/// Table Entry for the Sv39 MMU
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct TableEntry(u64);

/// Virtual Address
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtualAddress(u64);

/// Memory Page Sizes / Level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryPageLevel {
    Level0,
    Level1,
    Level2,
}

/// Read Write Execute Flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RWXFlags {
    ReadOnly,
    ReadWrite,
    Execute,
    ReadExecute,
    ReadWriteExecute,
}

/// User and Global Flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UGFlags {
    User,
    Global,
    UserGlobal,
    None,
}

/// Page Table
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct PageTable([TableEntry; 512]);

impl TableEntry {
    /// Construct a new table entry, it would be an empty (unmapped) entry
    pub fn new() -> Self {
        Self(0)
    }

    /// Private function to get a particular bit
    fn bit(&self, bit: usize) -> bool {
        (self.0 >> bit) & 1 > 0
    }

    /// Private function to set a particular bit
    fn set_bit(&mut self, bit: usize, value: bool) {
        if value {
            self.0 |= 1 << bit;
        } else {
            self.0 &= !(1 << bit);
        }
    }

    /// Get the valid bit
    pub fn valid(&self) -> bool {
        self.bit(0)
    }

    /// Set the valid bit
    pub fn set_valid(&mut self, value: bool) {
        self.set_bit(0, value)
    }

    /// Get the read bit
    pub fn read(&self) -> bool {
        self.bit(1)
    }

    /// Set the read bit
    pub fn set_read(&mut self, value: bool) {
        self.set_bit(1, value)
    }

    /// Get the write bit
    pub fn write(&self) -> bool {
        self.bit(2)
    }

    /// Set the write bit
    pub fn set_write(&mut self, value: bool) {
        self.set_bit(2, value)
    }

    /// Get execute bit
    pub fn execute(&self) -> bool {
        self.bit(3)
    }

    /// Set the execute bit
    pub fn set_execute(&mut self, value: bool) {
        self.set_bit(3, value)
    }

    /// Get user bit
    pub fn user(&self) -> bool {
        self.bit(4)
    }

    /// Set the user bit
    pub fn set_user(&mut self, value: bool) {
        self.set_bit(4, value)
    }

    /// Get global bit
    pub fn global(&self) -> bool {
        self.bit(5)
    }

    /// Set the global bit
    pub fn set_global(&mut self, value: bool) {
        self.set_bit(5, value)
    }

    /// Get accessed bit
    pub fn accessed(&self) -> bool {
        self.bit(6)
    }

    /// Set the accessed bit
    pub fn set_accessed(&mut self, value: bool) {
        self.set_bit(6, value)
    }

    /// Get dirty bit
    pub fn dirty(&self) -> bool {
        self.bit(7)
    }

    /// Set the dirty bit
    pub fn set_dirty(&mut self, value: bool) {
        self.set_bit(7, value)
    }

    /// Get the RSW value
    pub fn rsw(&self) -> u64 {
        (self.0 >> 8) & 0b11
    }

    /// Set the RSW value
    pub fn set_rsw(&mut self, value: u64) {
        self.0 &= !(0b11 << 8);
        self.0 |= (value & 0b11) << 8;
    }

    /// Get the physical page number at the given level
    pub fn ppn(&self, level: MemoryPageLevel) -> u64 {
        if level.index() < 2 {
            (self.0 >> (10 + 9 * level.index())) & 0x1ff
        } else {
            (self.0 >> (10 + 9 * level.index())) & 0x3ffffff
        }
    }

    /// Set the physical page number at the given level
    pub fn set_ppn(&mut self, level: MemoryPageLevel, value: u64) {
        if level.index() < 2 {
            self.0 &= !(0x1ff << (10 + 9 * level.index()));
            self.0 |= (value & 0x1ff) << (10 + 9 * level.index());
        } else {
            self.0 &= !(0x3ffffff << (10 + 9 * level.index()));
            self.0 |= (value & 0x3ffffff) << (10 + 9 * level.index());
        }
    }

    /// Get the full physical page number
    pub fn full_ppn(&self) -> u64 {
        (self.0 >> 10) & 0xfffffffffff
    }

    /// Set the full physical page number
    pub fn set_full_ppn(&mut self, value: u64) {
        self.0 &= !(0xfffffffffff << 10);
        self.0 |= (value & 0xfffffffffff) << 10;
    }

    /// Check if the entry is a leaf
    pub fn is_leaf(&self) -> bool {
        self.read() || self.write() || self.execute()
    }

    /// Get a mutable reference to the next level of the page table
    pub unsafe fn next_level_mut(&self) -> &mut PageTable {
        (((self.full_ppn() << 12) as usize) as *mut PageTable)
            .as_mut()
            .unwrap()
    }
}

impl core::fmt::Display for TableEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.valid() {
            if self.is_leaf() {
                write!(f, "Leaf  ")?;
            } else {
                write!(f, "Nested")?;
            }

            write!(f, " {:#x}", self.full_ppn() << 12)?;

            if self.is_leaf() {
                // daguxrwv
                write!(f, " ")?;
                write!(f, "{}", if self.dirty() { "d" } else { "-" })?;
                write!(f, "{}", if self.accessed() { "a" } else { "-" })?;
                write!(f, "{}", if self.global() { "g" } else { "-" })?;
                write!(f, "{}", if self.user() { "u" } else { "-" })?;
                write!(f, "{}", if self.execute() { "x" } else { "-" })?;
                write!(f, "{}", if self.read() { "r" } else { "-" })?;
                write!(f, "{}", if self.write() { "w" } else { "-" })?;
                write!(f, "v")?;
            }
        } else {
            write!(f, "Invalid")?;
        }

        Ok(())
    }
}

impl VirtualAddress {
    /// Wrap an address
    pub fn new(addr: u64) -> Self {
        Self(addr)
    }

    /// Get the wrapped value
    pub fn get(&self) -> u64 {
        self.0
    }

    /// Get the virtual page number for the given level [0, 2]
    pub fn vpn(&self, level: MemoryPageLevel) -> u64 {
        (self.0 >> (12 + level.index() * 9)) & 0x1ff
    }

    /// Get the page offset in the virtual address
    pub fn page_offset(&self) -> u64 {
        self.0 & 0xfff
    }

    /// Get the page offset in the virtual address for the given level
    pub fn page_offset_level(&self, level: MemoryPageLevel) -> u64 {
        match level {
            MemoryPageLevel::Level0 => self.0 & 0xfff,
            MemoryPageLevel::Level1 => self.0 & 0x1fffff,
            MemoryPageLevel::Level2 => self.0 & 0x3fffffff,
        }
    }
}

impl MemoryPageLevel {
    /// Convert a memory page level to an index
    pub fn index(&self) -> usize {
        match self {
            MemoryPageLevel::Level0 => 0,
            MemoryPageLevel::Level1 => 1,
            MemoryPageLevel::Level2 => 2,
        }
    }
}

impl core::fmt::Display for MemoryPageLevel {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match &self {
            MemoryPageLevel::Level0 => write!(f, "4 KiB"),
            MemoryPageLevel::Level1 => write!(f, "2 MiB"),
            MemoryPageLevel::Level2 => write!(f, "1 GiB"),
        }
    }
}

impl RWXFlags {
    /// Determine if the read flag is present
    pub fn read(&self) -> bool {
        matches!(
            self,
            RWXFlags::ReadExecute
                | RWXFlags::ReadOnly
                | RWXFlags::ReadWrite
                | RWXFlags::ReadWriteExecute
        )
    }

    /// Determine if the write flag is present
    pub fn write(&self) -> bool {
        matches!(self, RWXFlags::ReadWrite | RWXFlags::ReadWriteExecute)
    }

    /// Determine if the execute flag is present
    pub fn execute(&self) -> bool {
        matches!(
            self,
            RWXFlags::Execute | RWXFlags::ReadExecute | RWXFlags::ReadWriteExecute
        )
    }
}

impl core::fmt::Display for RWXFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            RWXFlags::ReadOnly => write!(f, "r--"),
            RWXFlags::ReadWrite => write!(f, "rw-"),
            RWXFlags::Execute => write!(f, "--x"),
            RWXFlags::ReadExecute => write!(f, "r-x"),
            RWXFlags::ReadWriteExecute => write!(f, "rwx"),
        }
    }
}

impl UGFlags {
    /// Determine if the user flag is present
    pub fn user(&self) -> bool {
        matches!(self, UGFlags::User | UGFlags::UserGlobal)
    }

    /// Determine if the global flag is present
    pub fn global(&self) -> bool {
        matches!(self, UGFlags::Global | UGFlags::UserGlobal)
    }
}

impl core::fmt::Display for UGFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            UGFlags::User => write!(f, "u-"),
            UGFlags::Global => write!(f, "-g"),
            UGFlags::UserGlobal => write!(f, "ug"),
            UGFlags::None => write!(f, "--"),
        }
    }
}

impl PageTable {
    /// Construct a new, empty page table
    pub fn new() -> Self {
        PageTable([TableEntry::new(); 512])
    }

    /// Translate a virtual address to an entry and an address
    pub fn translate_to_entry(&self, virt: VirtualAddress) -> Option<(TableEntry, usize)> {
        // Get the components of the virtual address
        let vpn2 = virt.vpn(MemoryPageLevel::Level2);
        let vpn1 = virt.vpn(MemoryPageLevel::Level1);
        let vpn0 = virt.vpn(MemoryPageLevel::Level0);

        // Next, start going down the table of entries at each level
        let entry_level2 = self.0[(vpn2 % 512) as usize];
        // If the entry is not valid, we will return a page fault
        if entry_level2.valid() == false {
            return None;
        }
        // If the entry is a leaf, we will break out here, returning the physical address
        else if entry_level2.is_leaf() {
            let result = (entry_level2.ppn(MemoryPageLevel::Level2) << 30)
                | virt.page_offset_level(MemoryPageLevel::Level2);
            return Some((entry_level2, result as usize));
        }

        // Otherwise, we need to get the next page table from the entry
        // Safety: Since this entry is valid and not a leaf, we know it must point to a valid page table
        let level2_table = unsafe { entry_level2.next_level_mut() };

        // Next, start going down the table of entries at each level
        let entry_level1 = level2_table.0[(vpn1 % 512) as usize];
        // If the entry is not valid, we will return a page fault
        if entry_level1.valid() == false {
            return None;
        }
        // If the entry is a leaf, we will break out here, returning the physical address
        else if entry_level1.is_leaf() {
            let result = (entry_level1.ppn(MemoryPageLevel::Level2) << 30)
                | (entry_level1.ppn(MemoryPageLevel::Level1) << 21)
                | virt.page_offset_level(MemoryPageLevel::Level1);
            return Some((entry_level1, result as usize));
        }

        // Otherwise, we need to get the next page table from the entry
        // Safety: Since this entry is valid and not a leaf, we know it must point to a valid page table
        let level1_table = unsafe { entry_level1.next_level_mut() };

        // Next, start going down the table of entries at each level
        let entry_level0 = level1_table.0[(vpn0 % 512) as usize];
        // If the entry is not valid, we will return a page fault
        if entry_level0.valid() == false {
            return None;
        }
        // If the entry is a leaf, we will break out here, returning the physical address
        else if entry_level0.is_leaf() {
            let result = (entry_level0.ppn(MemoryPageLevel::Level2) << 30)
                | (entry_level0.ppn(MemoryPageLevel::Level1) << 21)
                | (entry_level0.ppn(MemoryPageLevel::Level0) << 12)
                | virt.page_offset_level(MemoryPageLevel::Level0);
            return Some((entry_level0, result as usize));
        }

        // If all else fails, and we are missing a leaf node, return a page fault
        None
    }

    /// Translate a virtual address to a physical address
    pub fn translate(&self, virt: VirtualAddress) -> Option<usize> {
        self.translate_to_entry(virt).map(|v| v.1)
    }

    /// Map a virtual address to a physical address
    pub fn map(
        &mut self,
        virt: VirtualAddress,
        phys: usize,
        access_flags: RWXFlags,
        flags: UGFlags,
        level: MemoryPageLevel,
    ) {
        kdebugln!(unsafe MemoryMapping, "Map {:x} -> {:x} {}{} at {}", virt.0, phys, access_flags, flags, level);

        // Get the current reference to an entry
        let mut entry = &mut self.0[virt.vpn(MemoryPageLevel::Level2) as usize];

        // Go backwards through the levels to the appropriate end point
        let levels = [MemoryPageLevel::Level1, MemoryPageLevel::Level0];
        for level in &levels[..2 - level.index()] {
            // If the current entry is invalid, we need to allocate a new page table for the next level
            if entry.valid() == false {
                // Allocate the next level
                let next_level = libutils::sync::no_interrupts(|no_interrupts| {
                    crate::mem::PAGE_ALLOCATOR.allocate_static(no_interrupts, PageTable::new())
                })
                .expect("Unable to allocate page table");

                // Convert the next page table into an address
                let next_level_address = next_level as *mut PageTable as usize;

                // Clear the entry
                *entry = TableEntry::new();

                // Write the address of the next page table into the current entry
                entry.set_full_ppn(next_level_address as u64 >> 12);

                // Validate the current entry, and make sure the flags are clear
                entry.set_valid(true);
            }

            // Step to the next entry
            // Safety: We just ensured the entry is either valid or we allocated a new one
            entry = &mut unsafe { entry.next_level_mut() }.0[virt.vpn(*level) as usize];
        }

        // Fill in the entry
        *entry = TableEntry::new();
        entry.set_valid(true);
        entry.set_full_ppn(phys as u64 >> 12);

        entry.set_read(access_flags.read());
        entry.set_write(access_flags.write());
        entry.set_execute(access_flags.execute());

        entry.set_global(flags.global());
        entry.set_user(flags.user());

        entry.set_dirty(access_flags.write());
        entry.set_accessed(access_flags.read());
    }

    /// Identity map a range of addresses
    pub fn identity_map(
        &mut self,
        start: usize,
        end: usize,
        access_flags: RWXFlags,
        flags: UGFlags,
    ) {
        // Loop over the range in page sized chunks
        let mut walking_addr = start & !(4096 - 1);
        while walking_addr <= end {
            // Map each page sized chunk
            self.map(
                VirtualAddress(walking_addr as u64),
                walking_addr,
                access_flags,
                flags,
                MemoryPageLevel::Level0,
            );

            walking_addr += crate::mem::PAGE_SIZE;
        }
    }

    /// Recursively remove all of the page tables and free the pages allocated
    pub fn unmap_all(&mut self, no_interrupts: libutils::sync::NoInterruptMarker) {
        // We iterate over all of the entries in the table
        for entry in &mut self.0 {
            // If the entry is currently valid, we much recursively delete any non-leaf entries
            if entry.valid() {
                // If it is not a leaf, it points to another page, a page which we need to delete.
                if !entry.is_leaf() {
                    // Safety: We know this entry is both valid and not a leaf
                    let next_level = unsafe { entry.next_level_mut() };

                    // Unmap the next level
                    next_level.unmap_all(no_interrupts);

                    // Free the level
                    // Safety: We know this page is one that has been allocated as it was from the page table
                    unsafe {
                        crate::mem::PAGE_ALLOCATOR
                            .free_pages_unchecked(
                                no_interrupts,
                                next_level as *mut PageTable as *mut crate::mem::Page,
                                1,
                            )
                            .expect("Unable to free page table entry");
                    }
                }

                // Invalidate the entry
                entry.set_valid(false);
            }
        }
    }
}

impl core::fmt::Display for PageTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for (i, entry) in self.0.iter().enumerate() {
            writeln!(f, "{:#05x}  {}", i, entry)?;
        }

        Ok(())
    }
}
