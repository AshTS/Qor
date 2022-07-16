use core::arch::asm;

#[derive(Clone, Copy)]
pub struct NoInterruptMarker
{
    _empty: ()
}

impl NoInterruptMarker
{
    /// Initialize the `NoInterruptMarker`, this is marked as unsafe as it should only be initialized when in a context where no interrupts can occur
    /// 
    /// # Safety
    /// 
    /// This should only ever be constructed in the `no_interrupt` context
    pub unsafe fn new() -> Self
    {
        Self
        {
            _empty: ()
        }
    }
}

/// Execute a function in an interrupt free context
pub fn no_interrupts<F, R>(func: F) -> R where F: FnOnce(NoInterruptMarker) -> R
{
    // Save the current machine interrupt enable flags
    let mut flags: usize = 0;
    unsafe
    {
        asm!("csrr {flags}, mie", flags = inout(reg) flags);
    }

    let result: R;
    // Safety: Because we execute the code to prevent interrupts, we can construct the `NoInterruptMarker`
    unsafe
    {
        result = func(NoInterruptMarker::new());
    }

    // Reenable the machine interrupt enable flags
    unsafe
    {
        asm!("csrw mie, {flags}", flags = in(reg) flags);
    }

    result
}