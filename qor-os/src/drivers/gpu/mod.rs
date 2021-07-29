pub mod driver;
pub use driver::*;

// Global Graphics Driver
static mut GLOBAL_GRAPHICS_DRIVER: Option<GenericGraphics> = None;

/// Initialize the graphics driver
pub fn init_graphics_driver()
{

    // Test the GPU
    let raw_driver = super::virtio::get_gpu_driver(0).unwrap();
    let mut driver = GenericGraphics::new(raw_driver);

    driver.init();

    driver.force_update();

    *unsafe { &mut GLOBAL_GRAPHICS_DRIVER } = Some(driver);
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