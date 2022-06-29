#[derive(Clone, Copy)]
pub struct InitThreadMarker
{
    _empty: ()
}

impl InitThreadMarker
{
    /// Initialize the `InitThreadMarker`, this is marked as unsafe as it should only be initialized when the kernel is first booting.
    /// 
    /// # Safety
    /// 
    /// This should only ever be constructed during kernel boot
    pub unsafe fn new() -> Self
    {
        Self
        {
            _empty: ()
        }
    }
}