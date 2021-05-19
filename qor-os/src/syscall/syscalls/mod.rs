pub mod close;
pub use close::*;

pub mod exit;
pub use exit::*;

pub mod open;
pub use open::*;

pub mod write;
pub use write::*;

pub unsafe fn to_str(ptr: usize) -> &'static str
{
    let mut l = 0;
    let ptr = ptr as *mut u8;

    while *ptr.add(l) != 0
    {
        l += 1;
    }

    let slice = core::slice::from_raw_parts(ptr as *mut u8, l);

    core::str::from_utf8_unchecked(slice)
}