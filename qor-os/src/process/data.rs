/// Process Data
pub struct ProcessData
{
    pub stack_size: usize, // Stack size in pages
    pub mem_ptr: *mut u8,
    pub mem_size: usize, // Size of the memory allocated in pages
}