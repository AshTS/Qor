/// Memory statistics for a process
#[derive(Debug, Clone, Copy)]
pub struct MemoryStats
{
    pub resident: usize,
    pub shared: usize,
    pub text: usize,
    pub data: usize
}

impl MemoryStats
{
    /// Create a new MemoryStats object
    pub fn new(resident: usize, shared: usize, text: usize, data: usize) -> Self
    {
        Self
        {
            resident,
            shared,
            text,
            data
        }
    }
}

impl core::fmt::Display for MemoryStats
{
    /// Render the memory statistics as will appear in /proc/[pid]/statm
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result
    {
        write!(f, "{} {} {} {} {} {} {}", 
            self.resident + self.text + self.data,
            self.resident,
            self.shared,
            self.text,
            0,
            self.data,
            0)
    }
    
}