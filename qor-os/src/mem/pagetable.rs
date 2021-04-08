use core::u64;

use super::pages::PAGE_SIZE;

#[repr(u64)]
/// bit masks for the various bits within the entry structure
pub enum EntryBits
{
    Valid = 0b1 << 0,
    Read = 0b1 << 1,
    Write = 0b1 << 2,
    Execute = 0b1 << 3,
    Global = 0b1 << 5,
    Accessed = 0b1 << 6,
    Dirty = 0b1 << 7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Entry structure for the Sv39 Memory Management System
pub struct Entry
{
    data: u64
}

impl Entry
{
    /// Get the value for the given bit position
    pub fn get_bit(&self, bit: EntryBits) -> bool
    {
        self.data & (bit as u64) > 0
    }

    /// Set the value for the given bit position
    pub fn set_bit(&mut self, bit: EntryBits, val: bool)
    {
        let bit_u64 = bit as u64;
        self.data = (self.data & (!bit_u64)) | if val {bit_u64} else {0};
    }

    /// Get the PPN value
    pub fn get_ppn(&self) -> usize
    {
        ((self.data >> 10) & ((1 << 44) - 1)) as usize
    }

    /// Set the PPN value
    pub fn set_ppn(&mut self, ppn: usize)
    {
        let mask = ((1 << 44) - 1) << 12;
        self.data &= mask;
        self.data |= ((ppn & ((1 << 44) - 1)) << 12) as u64;
    }

    /// Return true iff the entry has its valid bit set
    pub fn is_valid(&self) -> bool
    {
        self.get_bit(EntryBits::Valid)
    }

    /// Returns true iff the given entry is a leaf node (i.e) it points to a
    /// page where memory will be stored, this is signified by none of the read,
    /// write or execute bits being set
    pub fn is_leaf(&self) -> bool
    {
        self.get_bit(EntryBits::Execute) |
        self.get_bit(EntryBits::Read) |
        self.get_bit(EntryBits::Write)
    }

    /// Get the wrapped data
    pub fn get_data(&self) -> u64
    {
        self.data
    }

    /// Set the wrapped data
    pub fn set_data(&mut self, data: u64)
    {
        self.data = data
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Table structure for the Sv39 Memory Management System, made up of 512
/// Entries
pub struct Table
{
    entries : [Entry; 512]
}

impl Table
{
    /// Create a new Table from the page number where the table is to be
    /// allocated
    /// Safety: The given page number must be valid
    pub unsafe fn new(page_number: usize) -> &'static mut Self
    {
        let address = (page_number * PAGE_SIZE) as *mut Table;

        let mut table = *address;

        // Invalidate all of the entries
        for i in table.entries.as_mut()
        {
            i.set_bit(EntryBits::Valid, false);
        }

        address.as_mut().unwrap()
    }
}

impl core::ops::Index<usize> for Table
{
    type Output = Entry;

    fn index(&self, index: usize) -> &Self::Output
    {
        &self.entries[index]
    }
}

impl core::ops::IndexMut<usize> for Table
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output
    {
        &mut self.entries[index]
    }
}