

pub struct PowerDriver
{
    base: usize
}

impl PowerDriver
{
    pub const unsafe fn new(base: usize) -> Self
    {
        Self
        {
            base
        }
    }

    pub fn shutdown(&self)
    {
        unsafe
        {
            (self.base as *mut u32).write_volatile(0x5555);
        }
    }
}