use crate::trap::TrapCause;
use crate::trap::TrapFrame;

#[no_mangle]
extern "C" fn m_trap(epc: usize,
                     tval: usize,
                     cause: TrapCause,
                     hart: usize,
                     status: usize,
                     frame: &mut TrapFrame) -> usize
{
    match cause
    {
        TrapCause::BreakPoint =>
        {
            kerrorln!(unsafe "Breakpoint Triggered At {:#x}", epc);

            epc + 4
        },
        TrapCause::IllegalInstruction =>
        {
            kerrorln!(unsafe "Illegal Instruction {:016x} at {:#x}", tval, epc);
            panic!();
        }
        TrapCause::InstructionAccessFault =>
        {
            kerrorln!(unsafe "Instruction Access Fault at {:#x}", epc);
            panic!();
        }
        TrapCause::InstructionAddressMisaligned =>
        {
            kerrorln!(unsafe "Misaligned Instruction {:016x} at {:#x}", tval, epc);
            panic!();
        }
        TrapCause::InstructionPageFault =>
        {
            kerrorln!(unsafe "Instruction Page Fault at {:#x}", epc);
            panic!();
        }
        TrapCause::LoadAccessFault =>
        {
            kerrorln!(unsafe "Load Access Fault at address {:#x} in instruction at {:#x}", tval, epc);
            panic!();
        }
        TrapCause::LoadAddressMisaligned =>
        {
            kerrorln!(unsafe "Load Address Misaligned at address {:#x} in instruction at {:#x}", tval, epc);
            panic!();
        }
        TrapCause::LoadPageFault =>
        {
            kerrorln!(unsafe "Load Page Fault at address {:#x} in instruction at {:#x}", tval, epc);
            panic!();
        }
        TrapCause::StoreAccessFault =>
        {
            kerrorln!(unsafe "Store Access Fault at address {:#x} in instruction at {:#x}", tval, epc);
            panic!();
        }
        TrapCause::StoreAddressMisaligned =>
        {
            kerrorln!(unsafe "Store Address Misaligned  at address {:#x} in instruction at {:#x}", tval, epc);
            panic!();
        }
        TrapCause::StorePageFault =>
        {
            kerrorln!(unsafe "Store Page Fault at address {:#x} in instruction at {:#x}", tval, epc);
            panic!();
        }
        _ =>
        {
            kerrorln!(unsafe "Unhandled Trap {:?}:", cause);
            kerrorln!(unsafe "    PC:     {:#x}\n    HART:   {:x}\n    Status: {:#016x}\n    TVal:   {:#x}", epc, hart, status, tval);

            panic!("Unhandled Trap {:?}", cause);
        }
    }
}