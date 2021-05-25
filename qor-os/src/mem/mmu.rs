//! Memory Management Unit Interactions

use crate::*;

/// Virtual Address Translation Error
#[derive(Debug, Clone, Copy)]
pub enum TranslationError
{
    InvalidPage(usize),
    NoLeaf
}

/// Sv39 Virtual Address
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtualAddress(u64);

impl VirtualAddress
{
    /// Get the page offset
    pub fn page_offset(&self) -> usize
    {
        // The offset is stored in the lower 12 bits
        (self.0 & ((1 << 12) - 1)) as usize
    }

    /// Get the given virtual page number
    pub fn virtual_page_number(&self, index: usize) -> usize
    {
        // Ensure the index is in [0, 3]
        assert!(index < 4);

        ((self.0 >> (12 + 9 * index)) & ((1 << 9) - 1)) as usize
    }

    /// Set the page offset
    pub fn set_page_offset(&mut self, page_offset: usize)
    {
        self.0 &= !((1 << 12) - 1);
        self.0 |= page_offset as u64 & ((1 << 12) - 1);
    }

    /// Set the virtual page number
    pub fn set_virtual_page_number(&mut self, index: usize, number: usize)
    {
        // Ensure the index is in [0, 3]
        assert!(index < 4);

        self.0 &= !(((1 << 9) - 1) << (12 + 9 * index));
        self.0 |= (number as u64 & ((1 << 9) - 1))  << (12 + 9 * index);
    }
}

/// Sv39 Page Table Entry Flag
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PageTableEntryFlags(u8);

impl PageTableEntryFlags
{
    /// Valid Flag
    pub fn valid() -> Self
    {
        Self(1)
    }

    /// Readable Flag
    pub fn readable() -> Self
    {
        Self(2)
    }

    /// Writable Flag
    pub fn writable() -> Self
    {
        Self(4)
    }

    /// Executable Flag
    pub fn executable() -> Self
    {
        Self(8)
    }

    /// User Flag
    pub fn user() -> Self
    {
        Self(16)
    }

    /// Global Flag
    pub fn global() -> Self
    {
        Self(32)
    }

    /// Accessed Flag
    pub fn accessed() -> Self
    {
        Self(64)
    }

    /// Dirty Flag
    pub fn dirty() -> Self
    {
        Self(128)
    }
}

impl core::ops::BitOr<PageTableEntryFlags> for PageTableEntryFlags
{
    type Output = PageTableEntryFlags;

    fn bitor(self, rhs: PageTableEntryFlags) -> Self::Output
    {
        PageTableEntryFlags(self.0 | rhs.0)
    }
}

impl core::ops::BitAnd<PageTableEntryFlags> for PageTableEntryFlags
{
    type Output = bool;

    fn bitand(self, rhs: PageTableEntryFlags) -> Self::Output
    {
        self.0 & rhs.0 > 0
    }
}

/// Sv39 Page Table Entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PageTableEntry(u64);

impl PageTableEntry
{
    /// Create a new Page Table Entry
    pub fn new(physical_address: usize, flags: PageTableEntryFlags) -> Self
    {
        PageTableEntry(((physical_address as u64) << 10) | flags.0 as u64)
    } 

    /// Get the flag portion of the entry
    pub fn flag(&self) -> PageTableEntryFlags
    {
        PageTableEntryFlags(self.0 as u8 & 0xFF)
    }

    /// Get the given Physical Page Number of the entry
    pub fn ppn(&self, index: usize) -> usize
    {
        (self.0 as usize >> (10 + 9 * index)) & ((1 << 9) - 1)
    }
}

/// Sv39 Page Table
#[derive(Debug)]
pub struct PageTable
{
    entries: [PageTableEntry; 512]
}

// Ensure the page table is one page in size
static_assertions::const_assert!(core::mem::size_of::<PageTable>() == 4096);

impl PageTable
{
    /// Allocate a new page table
    pub fn allocate() -> &'static mut Self
    {
        // Allocate a new page on the kernel heap
        let page_ptr = super::kpzalloc(4096 / super::PAGE_SIZE).unwrap() as *mut Self;

        // Safety: The kernel will only give valid, free memory, and the memory has been zeroed
        unsafe { page_ptr.as_mut().unwrap() }
    }

    /// Inner mapping implementation
    fn inner_map(&mut self, vaddr: usize, paddr: usize, flags: PageTableEntryFlags, level: usize)
    {
        assert!(level < 3);

        // Ensure a leaf is being mapped
        assert!(flags.0 & 0xe != 0);

        // Separate out the vpn
        let vpn = [
				(vaddr >> 12) & 0x1ff,
				(vaddr >> 21) & 0x1ff,
				(vaddr >> 30) & 0x1ff,
	        ];

        // Reference to the current entry
        let mut v = &mut self.entries[vpn[2]];

        // Loop over all of the levels needed
        for i in (level..2).rev()
        {
            // If there is no entry placed here
            if !(v.flag() & PageTableEntryFlags::valid())
            {
                // Create a new table to link to
                let sub_table = PageTable::allocate();
                
                // Create the link
                *v = PageTableEntry::new((sub_table as *mut PageTable as usize) >> 12, PageTableEntryFlags::valid());
            }
            
            // Update the walking pointer
            let entry = ((v.0 & !0x3ff) << 2) as *mut PageTableEntry;
	        v = unsafe { entry.add(vpn[i]).as_mut().unwrap() };
        }

        // Insert the leaf entry
        *v = PageTableEntry::new(paddr >> 12, flags | PageTableEntryFlags::valid());
    }

    /// Add a new mapping to the page table
    pub fn map(&mut self, vaddr: usize, paddr: usize, flags: PageTableEntryFlags, level: usize)
    {
        kdebugln!(MemoryMapping, "Mapping Virt 0x{:x} to Phys 0x{:x} ({})", vaddr, paddr,
            match level
            {
                0 => "4KiB",
                1 => "2MiB",
                2 => "1GiB",
                _ => unreachable!()
            });

        self.inner_map(vaddr, paddr, flags, level);
    }

    /// Drop the given page table assuming it is at the given level
    fn drop_level(&mut self, level: usize)
    {
        // If not at the lowest level, drop all the lower ones
        if level > 0
        {
            // Loop ever every entry
            for entry in &self.entries
            {
                // If the entry is valid
                if entry.flag() & PageTableEntryFlags::valid()
                {
                    // Then drop the page it links to
                    let page = (entry.0 & !0x3ff) << 2;

                    // Convert the address to a page table
                    let table = unsafe { (page as *mut PageTable).as_mut().unwrap() };
                    
                    // Drop the found table
                    table.drop_level(level - 1);
                }
            }
        }

        // Drop the current table
        super::kpfree(self as *mut PageTable as usize, 1).unwrap();
    }

    /// Drop a top level table
    pub fn drop(&mut self)
    {
        kdebugln!(MemoryMapping, "Dropping the page table at 0x{:x}", self as *mut PageTable as usize);

        self.drop_level(2);
    }

    /// Convert a virtual address to a physical address
    pub fn virt_to_phys(&self, vaddr: usize) -> Result<usize, TranslationError>
    {
        // Separate out the virtual page numbers
        let vpn = [
				(vaddr >> 12) & 0x1ff,
				(vaddr >> 21) & 0x1ff,
				(vaddr >> 30) & 0x1ff,
	        ];

        // Get the offset from the vaddr
        let offset = vaddr & 0xfff;

        // Reference to the current entry
        let mut v = &self.entries[vpn[2]];

        // Loop over all of the levels
        for i in (0..=2).rev()
        {
            let phys_addr = ((v.0 & !0x3ff) << 2) as usize;

            // If this entry is invalid, return an error
            if !(v.flag() & PageTableEntryFlags::valid())
            {
                return Err(TranslationError::InvalidPage(i));
            }
            // Check if the given level is a leaf
            else if v.flag().0 & 0xE != 0
            {
                let mut result = offset;

                for j in 0..=2
                {
                    // This level comes from the virtual address
                    if j < i
                    {
                        result += vpn[j] << (12 + 9 * i);
                    }
                    // This level comes from the physical address
                    else
                    {
                        result += phys_addr;
                        break;
                    }
                }
                
                return Ok(result);
            }
            // Otherwise, this is not a leaf
            else
            {
                // Update the walking pointer
                let entry = ((v.0 & !0x3ff) << 2) as *mut PageTableEntry;
                v = unsafe { entry.add(vpn[i - 1]).as_ref().unwrap() };
            }
        }

        Err(TranslationError::NoLeaf)
    }

    /// Internal identity map helper
    fn identity_map_helper(&mut self, start_addr: usize, end_addr: usize, flags: PageTableEntryFlags)
    {
        // Convert the addresses to pages to map
        let start_page = start_addr & !(4096 - 1);
        let end_page = end_addr & !(4096 - 1);

        // Get the length of the region in bytes
        let mut length = 4096 + end_page - start_page;

        // Running page poitner
        let mut current_page = start_page;

        while length > 0
        {
            // If the length is greater than one GiB and is aligned to a 1 GiB boundary
            if length >= 0x4000_0000 && current_page & (0x4000_0000 - 1) == 0
            {
                // Map a one GiB page
                self.inner_map(current_page, current_page, flags, 2);

                current_page += 0x4000_0000;
                length -= 0x4000_0000;
            }

            // If the length is greater than 2 MiB and is aligned to a 2 MiB boundary
            if length >= 0x20_0000 && current_page & (0x20_0000 - 1) == 0
            {
                // Map a 2 MiB page
                self.inner_map(current_page, current_page, flags, 1);

                current_page += 0x20_0000;
                length -= 0x20_0000;
            }

            // Otherwise, map the individual 4 KiB pages
            self.inner_map(current_page, current_page, flags, 0);

            current_page += 0x1000;
            length -= 0x1000;
        }
    }

    /// Identity map a region of memory
    pub fn identity_map(&mut self, start_addr: usize, end_addr: usize, flags: PageTableEntryFlags)
    {
        kdebugln!(MemoryMapping, "Identity Mapping 0x{:x} - 0x{:x}", start_addr, end_addr);

        self.identity_map_helper(start_addr, end_addr, flags);
    }
}

/*
    ========================== Tests for MMU Helpers ==========================
*/

/// Test MMU Helpers - Page Offset Extraction
#[test_case]
pub fn test_page_offset_extraction()
{
    assert_eq!(VirtualAddress(0x46F45E).page_offset(), 0x45E);
    assert_eq!(VirtualAddress(0xE).page_offset(), 0xE);
    assert_eq!(VirtualAddress(0xFFFFFFFFFF).page_offset(), 0xFFF);
    assert_eq!(VirtualAddress(0x0).page_offset(), 0);

    let mut value = VirtualAddress(0xFFFFFFFFFFFFFFFF);
    value.set_page_offset(0x9A8);
    assert_eq!(value.page_offset(), 0x9A8);
}

/// Test MMU Helpers - Virtual Page Number
#[test_case]
pub fn test_virtual_page_number_extraction()
{
    assert_eq!(VirtualAddress(0xA6B46CF45E).virtual_page_number(0), 0xCF);
    assert_eq!(VirtualAddress(0xA6B46CF45E).virtual_page_number(1), 0x1A3);
    assert_eq!(VirtualAddress(0xA6B46CF45E).virtual_page_number(2), 0x9A);

    let mut value = VirtualAddress(0xFFFFFFFFFFFFFFFF);
    value.set_virtual_page_number(0, 0x1A8);
    assert_eq!(value.virtual_page_number(0), 0x1A8);

    value.set_virtual_page_number(1, 0x8D);
    assert_eq!(value.virtual_page_number(1), 0x8D);

    value.set_virtual_page_number(2, 0x1A8);
    assert_eq!(value.virtual_page_number(2), 0x1A8);

    assert_eq!(value.virtual_page_number(1), 0x8D);
    assert_eq!(value.virtual_page_number(0), 0x1A8);
}

/*
    ============================= Tests for Mapper =============================
*/

/// Test MMU - Mapping and Virtual Address Translation
#[test_case]
pub fn test_mapping_virtual_address_translation()
{
    let table = mem::mmu::PageTable::allocate();

    table.map(0x3F_0000_0000, 0x1_0000_0000, PageTableEntryFlags::readable() | PageTableEntryFlags::writable() | PageTableEntryFlags::executable(), 0);

    
    let root_ppn = table as *mut mem::mmu::PageTable as usize >> 12;
    let satp_val = 8 << 60 | root_ppn;
    riscv::register::satp::write(satp_val);

    let ptr = 0x3F_0000_0123;
    let next_ptr = table.virt_to_phys(ptr).unwrap();

    assert_eq!(next_ptr, 0x1_0000_0123);

    table.drop();
}