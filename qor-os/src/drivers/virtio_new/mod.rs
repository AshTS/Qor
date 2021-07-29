mod consts;
pub use consts::*;

mod discovery;
pub use discovery::*;

mod driver;
pub use driver::*;

pub mod drivers;

mod structs;
pub use structs::*;

// Discovered and initialized VirtIO devices
pub static mut VIRTIO_DEVICE_COLLECTION: Option<DeviceCollection> = None;

/// Get the block driver with the given index
pub fn get_block_driver(index: usize) -> Option<&'static mut drivers::block::BlockDriver>
{
    if let Some(collection) = unsafe { &mut VIRTIO_DEVICE_COLLECTION }
    {
        if let Some(driver) = collection.block_devices.get_mut(index)
        {
            Some(driver)
        }
        else
        {
            None
        }
    }
    else
    {
        None
    }
}