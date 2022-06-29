/// Read data from an offset into an mmio interface
pub unsafe fn read_offset<T>(base: usize, offset: usize) -> T
{
    ((base + offset) as *mut T).read_volatile()
}

/// Write data into an offset into an mmio interface
pub unsafe fn write_offset<T>(base: usize, offset: usize, data: T)
{
    ((base + offset) as *mut T).write_volatile(data);
}