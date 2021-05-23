//! Page grain memory allocation

use core::usize;

use crate::*;

// Size of the system pages
pub const PAGE_SIZE: usize = 4096;

// Ensure the size of the page is a multiple of 16 (required for the faster clearing for kzalloc)
static_assertions::const_assert_eq!(PAGE_SIZE % 16, 0);

pub const PAGE_MAP_EXCESS: usize = PAGE_SIZE - core::mem::size_of::<core::ptr::NonNull<u8>>() - core::mem::size_of::<*mut PageMap>() - core::mem::size_of::<usize>();
pub const PAGE_MAP_ENTRIES: usize = PAGE_MAP_EXCESS * 8;

/// Kernel Page Allocation Errors
#[derive(Debug)]
pub enum KernelPageAllocationError
{
    NotInTable(usize),
    NotAllocated(usize),
    NotAligned(usize),
    NotEnoughPages(usize)
}

/// Page Map Structure
pub struct PageMap
{
    ptr: core::ptr::NonNull<u8>,
    next_page: *mut PageMap,
    number: usize,
    data: [u8; PAGE_MAP_EXCESS]
}

// Ensure the PageMap structure is the right size
static_assertions::const_assert_eq!(core::mem::size_of::<PageMap>(), PAGE_SIZE);

impl PageMap
{
    /// Initialize the kernel page map
    ///
    /// Safety: The pointer and number of pages must be both valid and free
    pub unsafe fn initialize(ptr: usize, num_pages: usize) -> &'static mut Self
    {
        // Assert the pointer is properly aligned
        if ptr & (PAGE_SIZE - 1) != 0
        {
            panic!("The pointer given for the kernel heap is not {} aligned: 0x{:x}", PAGE_SIZE, ptr);
        }

        // Assert there are pages available in the kernel heap
        if num_pages == 0
        {
            panic!("No pages allocated to the kernel heap at 0x{:?}", ptr);
        }

        // Initialize the map
        let mut remaining = num_pages;
        let mut current_ptr = ptr;
        while remaining > 0
        {
            // Panic Safety: The safety assumption for this function asserts
            // that the pointer is valid, and the object will be initialized here
            let mut map = (current_ptr as *mut PageMap).as_mut().unwrap();

            // Step past the map
            current_ptr += PAGE_SIZE;
            remaining -= 1;

            // Get the number of pages to give to this page map
            let this_number = remaining.min(PAGE_MAP_ENTRIES);

            map.ptr = core::ptr::NonNull::new(current_ptr as *mut u8).unwrap();
            map.number = this_number;

            // Set the map to have all pages free
            map.data = [0; PAGE_MAP_EXCESS];

            // Step past the data
            current_ptr += this_number;
            remaining -= this_number;

            // Set the next pointer to null if no pages are remaining
            if remaining == 0
            {
                map.next_page = 0 as *mut PageMap;
            }
            // Otherwise set the next pointer to the next Page Map
            else
            {
                map.next_page = current_ptr as *mut PageMap;
            }
        }

        // Panic Safety: The safety assumption for this function asserts that
        // the pointer is valid, and it is initialized as a PageMap within this
        // function
        (ptr as *mut PageMap).as_mut().unwrap()
    }

    /// Check if the given page is allocated, returns None if the page doesn't
    /// exist
    pub fn check_allocation(&self, page_number: usize) -> Option<bool>
    {
        if page_number < self.number
        {
            Some(
                (self.data[page_number / 8] << (page_number % 8)) & 128 != 0
            )
        }
        else
        {
            None
        }
    }

    /// Get the address for a given page
    pub fn page_to_address(&self, page_number: usize) -> Option<usize>
    {
        if page_number < self.number
        {
            Some(self.ptr.as_ptr() as usize + page_number * PAGE_SIZE)
        }
        else
        {
            None
        }
    }

    /// Get the page number for a given address
    pub fn address_to_page(&self, address: usize) -> Option<usize>
    {
        if address & (PAGE_SIZE - 1) != 0
        {
            return None;
        }

        if address < self.ptr.as_ptr() as usize
        {
            return None;
        }

        let addr = address - self.ptr.as_ptr() as usize;
        let num = addr / PAGE_SIZE;

        if num < self.number
        {
            Some(num)
        }
        else
        {
            None
        }
    }

    /// Check if an address is in this map
    pub fn address_in_map(&self, address: usize) -> bool
    {
        if address < self.ptr.as_ptr() as usize
        {
            false
        } 
        else if (address - self.ptr.as_ptr() as usize) / PAGE_SIZE >= self.number
        {
            false
        }
        else
        {
            true
        }
    }

    /// Display the Page Map Allocations in Debug Mode
    pub fn debug_display(&self)
    {
        // Display the first page address
        kdebugln!(KernelPageTable, "First Address: 0x{:x}", self.ptr.as_ptr() as usize);

        let mut count = 0usize;
        let mut is_allocated = self.check_allocation(0).unwrap_or(false);

        for i in 0..self.number
        {
            if self.check_allocation(i) != Some(is_allocated) || i == self.number - 1
            {
                if count > 0
                {
                    kdebugln!(KernelPageTable, "{}: 0x{:x} - 0x{:x}    {} page{} {} bytes",
                              if is_allocated {"[ALLOC]"} else {"[FREE ]"},
                              self.page_to_address(i - count).unwrap_or(0),
                              self.page_to_address(i - 1).unwrap_or(0),
                              count,
                              if count > 1 {"s"} else {""},
                              count * PAGE_SIZE);
                }

                count = 1;
                is_allocated = !is_allocated;
            }
            else
            {
                count += 1;
            }
        }

        // Possibly recurse to the next page map
        if !self.next_page.is_null()
        {
            unsafe{ self.next_page.as_ref().unwrap().debug_display() };
        }
    }

    /// Free a set of pages
    pub fn free_page(&mut self, addr: usize) -> Result<(), KernelPageAllocationError>
    {
        if !self.address_in_map(addr)
        {
            // If the address is not in this map, skip to the next page
            if let Some(next) = unsafe { self.next_page.as_mut() }
            {
                next.free_page(addr)
            }
            else
            {
                #[cfg(not(test))]
                kerrorln!("Unable to free 0x{:x}, because it is not in the table", addr);
                Err(KernelPageAllocationError::NotInTable(addr))
            }
        }
        else
        {
            // Get the page number
            if let Some(page_number) = self.address_to_page(addr)
            {
                // Ensure the page is allocated
                if self.check_allocation(page_number) == Some(true)
                {
                    // Set the proper bit to 0
                    self.data[page_number / 8] &= !(128 >> (page_number % 8));
                    Ok(())
                }
                else
                {
                    #[cfg(not(test))]
                    kerrorln!("Unable to free 0x{:x}, because it is not allocated", addr);
                    Err(KernelPageAllocationError::NotAllocated(addr))
                }
            }
            else
            {
                #[cfg(not(test))]
                kerrorln!("Unable to free 0x{:x}, because it is not aligned", addr);
                Err(KernelPageAllocationError::NotAligned(addr))
            }
        }
    }

    /// Free consecutive pages
    pub fn free_pages(&mut self, addr: usize, count: usize) -> Result<(), KernelPageAllocationError>
    {
        // Loop over all invovled pages
        for i in 0..count
        {
            // Attempt to free the page
            self.free_page(addr + i * PAGE_SIZE)?;
        }

        Ok(())
    }

    /// Allocate consecutive pages
    pub fn alloc_pages(&mut self, count: usize) -> Result<usize, KernelPageAllocationError>
    {
        let mut found = 0;
        let mut start_page = 0;
        for i in 0..self.number / 8
        {
            // Skip if the given block of pages is fully allocated
            if self.data[i] == 0xFF { found = 0; continue; }

            // Loop over each page and check if it was allocated
            for j in 0..8
            {
                if self.check_allocation(i * 8 + j) != Some(false)
                {
                    found = 0;
                }
                else if found == 0
                {
                    start_page = i * 8 + j;
                    found += 1;
                }
                else
                {
                    found += 1;
                }

                if found == count
                {
                    for i in start_page..(start_page + count)
                    {
                        self.data[i / 8] |= 128 >> (i % 8);
                    }

                    return Ok(self.ptr.as_ptr() as usize + start_page * PAGE_SIZE);
                }
            }
        }

        // If the address is not in this map, skip to the next page
        if let Some(next) = unsafe { self.next_page.as_mut() }
        {
            next.alloc_pages(count)
        }
        else
        {
            #[cfg(not(test))]
            kerrorln!("Unable to allocate {} page{}, no space remaining", count, if count > 1 { "s" } else { "" });
            Err(KernelPageAllocationError::NotEnoughPages(count))
        }
    }

    /// Total pages linked
    pub fn total_pages(&self) -> usize
    {
        // If the address is not in this map, skip to the next page
        if let Some(next) = unsafe { self.next_page.as_ref() }
        {
            next.total_pages() + self.number
        }
        else
        {
            self.number
        }
    }

    /// Total allocated pages
    pub fn total_alloc_pages(&self) -> usize
    {
        let mut total = 0;
        for i in 0..self.number
        {
            if self.check_allocation(i) == Some(true)
            {
                total += 1;
            }
        }

        // If the address is not in this map, skip to the next page
        if let Some(next) = unsafe { self.next_page.as_ref() }
        {
            next.total_alloc_pages() + total
        }
        else
        {
            total
        }
    }
}