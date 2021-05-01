// Page size
pub const PAGE_SIZE: usize = 4096;

#[repr(C)]
/// Structure to store page data
pub struct PageData
{
    this_ptr: *mut PageData,
    next_page: *mut PageData,
    page_count: usize,
    is_head: bool
}

impl PageData
{
    /// Initialize a new page data structure at the given memory location with
    /// the given number of pages, will panic if the page count is zero
    /// Safety: the given address must be 4096 byte aligned and must have the
    /// proper number of free bytes following it
    pub fn init(address: usize, page_count: usize)
    {
        let addr = address as *mut PageData;

        // Size check, panic if allocating no space
        if page_count == 0
        {
            panic!("Cannot initialize a space of zero pages");
        }

        // Head node
        let mut data = PageData
        {
            this_ptr: addr,
            next_page: core::ptr::null_mut(),
            page_count: 1,
            is_head: true
        };

        // Initialize the second page (where the data is stored)
        if page_count > 1
        {
            let second_page = (address + PAGE_SIZE) as *mut PageData;

            let second_data = PageData
            {
                this_ptr: second_page,
                next_page: core::ptr::null_mut(),
                page_count: page_count - 1,
                is_head: false
            };

            data.next_page = second_page;

            // Store the second node
            unsafe 
            {
                *second_page = second_data;
            }
        }

        // Store the head
        unsafe 
        {
            *addr = data;
        }
    }

    /// Initialize a new page data structure at the given memory location with
    /// the given number of pages, will panic if the page count is zero without
    /// setting the is_head flag and allowing the next pointer to be set
    /// Safety: the given address must be 4096 byte aligned and must have the
    /// proper number of free bytes following it
    pub fn init_not_head(address: usize, page_count: usize, next: *mut PageData)
    {
        let addr = address as *mut PageData;

        // Size check, panic if allocating no space
        if page_count == 0
        {
            panic!("Cannot initialize a space of zero pages");
        }

        let data = PageData
        {
            this_ptr: addr,
            next_page: next,
            page_count: page_count, 
            is_head: false
        };

        // Store the data
        unsafe 
        {
            *addr = data;
        }
    }

    /// Get the total number of pages referenced by this chain
    /// Safety: Assuming the initialization was to a valid address (see the
    /// safety for the `PageData::init` function) this function will be safe
    pub fn get_number_pages(&self) -> usize
    {
        if let Some(next) = unsafe{self.next_page.as_ref()}
        {
            self.page_count + next.get_number_pages() - if self.is_head {1} else {0}
        }
        else
        {
            self.page_count - if self.is_head {1} else {0}
        } 
    }

    /// Get reference to the next page
    /// Safety: Assuming the initialization was to a valid address (see the
    /// safety for the `PageData::init` function) this function will be safe
    pub fn next(&self) -> Option<&mut PageData>
    {
        unsafe { self.next_page.as_mut() }
    }

    /// Get the amount of free space, for all non-head nodes this will return
    /// the number of pages, for a head node it will return zero
    pub fn get_space(&self) -> usize 
    {
        if self.is_head
        {
            0
        }
        else
        {
            self.page_count
        }
    }

    /// Get the current pointer
    pub fn get_current_pointer(&self) -> *mut PageData
    {
        self.this_ptr
    }

    /// Get the next pointer
    pub fn get_next_pointer(&self) -> *mut PageData
    {
        self.next_page
    }

    /// Set the next pointer
    pub fn set_next(&mut self, ptr: *mut PageData)
    {
        self.next_page = ptr;
    }

    /// Defragment the kernel heap
    pub fn defrag(&mut self)
    {
        if self.is_head
        {
            if let Some(next) = self.next()
            {
                next.defrag()
            }
        }
        else
        {
            let mut current = unsafe { self.get_current_pointer().as_mut() }.unwrap();

            loop
            {
                if let Some(next) = current.next()
                {
                    if next.get_current_pointer() as usize == current.get_current_pointer() as usize + PAGE_SIZE * current.get_space()
                    {
                        self.next_page = next.get_next_pointer();
                        self.page_count += next.get_space();
                        current = next;
                    }
                    else
                    {
                        break;
                    }
                }
                else
                {
                    break;
                }
            }
        }
    }
}

impl core::fmt::Display for PageData
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        write!(f, "PageData(free: {}, this: 0x{:x}, next: {:x}){}",
            self.page_count,
            self.this_ptr as usize,
            self.next_page as usize, 
            if self.is_head {" (Head)"} else {""})
    }
}