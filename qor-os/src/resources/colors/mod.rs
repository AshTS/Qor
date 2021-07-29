/// Pixel Object
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Pixel
{
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8
}

impl Pixel
{
    /// Create a new pixel
    pub const fn new(red: u8, green: u8, blue: u8) -> Self
    {
        Self
        {
            red,
            green,
            blue,
            alpha: 255
        }
    }
}

pub mod ega;