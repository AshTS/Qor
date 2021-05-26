/// Trap handler (only called from the trap handler in assembly)
#[no_mangle]
extern "C" fn m_trap(epc: usize,
                     tval: usize,
                     cause: usize,
                     hart: usize,
                     status: usize,
                     frame: &'static mut super::TrapFrame)
                     -> usize
{
    super::handler::interrupt_handler(
        super::InterruptContext::new(epc, tval, cause, hart, status, frame))
}