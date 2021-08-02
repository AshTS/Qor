use crate::*;

pub mod driver;
pub use driver::*;

pub mod structs;

// Global Graphics Driver
static mut GLOBAL_GRAPHICS_DRIVER: Option<GenericGraphics> = None;

/// Initialize the graphics driver
pub fn init_graphics_driver() -> bool
{
    // Test the GPU
    if let Some(raw_driver) = super::virtio::get_gpu_driver(0)
    {
        let mut driver = GenericGraphics::new(raw_driver);

        driver.init();

        driver.force_update();

        *unsafe { &mut GLOBAL_GRAPHICS_DRIVER } = Some(driver);

        true
    }
    else
    {
        kerrorln!("Unable to find GPU driver, /dev/fb0 and /dev/disp will not be available");
        false
    }
}

/// Get a reference to the global graphics driver
pub fn get_global_graphics_driver() -> &'static mut GenericGraphics
{
    if let Some(reference) = unsafe { &mut GLOBAL_GRAPHICS_DRIVER }
    {
        reference
    }
    else
    {
        panic!("Cannot access uninitialized graphics driver");
    }
}

/// Check if the graphics driver is loaded
pub fn is_graphics_driver_loaded() -> bool
{
    unsafe { &GLOBAL_GRAPHICS_DRIVER }.is_some()
}