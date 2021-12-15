/// Kernel Utils

// Modules
pub mod memdump;
pub use memdump::*;

pub mod ringbuffer;
pub use ringbuffer::*;

pub mod blocking;
pub use blocking::*;

use crate::*;

/// Seperate a path into a path and the last item (path, name)
pub fn separate_path_last(path: &str) -> (String, String)
{
    if let Some((name, path_items)) = path.split("/").collect::<Vec<_>>().split_last()
    {
        (path_items.join("/") + "/", name.to_string())
    }
    else
    {
        (String::new(), String::new())
    }
}