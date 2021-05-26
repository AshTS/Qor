use core::usize;

use super::TrapFrame;

/// Interrupt enumeration
#[derive(Debug, Clone, Copy)]
pub enum InterruptType
{
    UserSoftwareInterrupt,
    SupervisorSoftwareInterrupt,
    MachineSoftwareInterrupt,
    UserTimeInterrupt,
    SupervisorTimerInterrupt,
    MachineTimerInterrupt,
    UserExternalInterrupt,
    SupervisorExternalInterrupt,
    MachineExternalInterrupt,
    InstructionAddressMisaligned,
    InstructionAccessFault,
    IllegalInstruction,
    Breakpoint,
    LoadAddressMisaligned,
    LoadAccessFault,
    StoreAddressMisaligned,
    StoreAccessFault,
    UserEnvironmentCall,
    SupervisorEnvironmentCall,
    MachineEnvironmentCall,
    InstructionPageFault,
    LoadPageFault,
    StorePageFault,
    UnknownSync(usize),
    UnknownAsync(usize)
}

/// Interrupt Context
pub struct InterruptContext
{
    epc: usize,
    tval: usize,
    cause: InterruptType,
    hart: usize,
    status: usize,
    frame: &'static mut TrapFrame,
    async_trap: bool
}

impl InterruptContext
{
    /// Create a new Interrupt Context
    pub fn new(epc: usize, tval: usize, cause: usize, hart: usize, status: usize, frame: &'static mut TrapFrame) -> Self
    {
        let async_trap = cause >> 63 & 1 == 1 ;

        let interrupt_type = match (async_trap, cause & 0xfff)
        {
            (true, 0) => InterruptType::UserSoftwareInterrupt,
            (true, 1) => InterruptType::SupervisorSoftwareInterrupt,
            (true, 3) => InterruptType::MachineSoftwareInterrupt,
            (true, 4) => InterruptType::UserTimeInterrupt,
            (true, 5) => InterruptType::SupervisorTimerInterrupt,
            (true, 7) => InterruptType::MachineTimerInterrupt,
            (true, 8) => InterruptType::UserExternalInterrupt,
            (true, 9) => InterruptType::SupervisorExternalInterrupt,
            (true, 11) => InterruptType::MachineExternalInterrupt,

            (false, 0) => InterruptType::InstructionAddressMisaligned,
            (false, 1) => InterruptType::InstructionAccessFault,
            (false, 2) => InterruptType::IllegalInstruction,
            (false, 3) => InterruptType::Breakpoint,
            (false, 4) => InterruptType::LoadAddressMisaligned,
            (false, 5) => InterruptType::LoadAccessFault,
            (false, 6) => InterruptType::StoreAddressMisaligned,
            (false, 7) => InterruptType::StoreAccessFault,
            (false, 8) => InterruptType::UserEnvironmentCall,
            (false, 9) => InterruptType::SupervisorEnvironmentCall,
            (false, 11) => InterruptType::MachineEnvironmentCall,
            (false, 12) => InterruptType::InstructionPageFault,
            (false, 13) => InterruptType::LoadPageFault,
            (false, 15) => InterruptType::StorePageFault,

            (false, default) => InterruptType::UnknownSync(default),
            (true, default) => InterruptType::UnknownAsync(default),
        };

        Self
        {
            epc,
            tval,
            cause: interrupt_type,
            hart,
            status,
            frame,
            async_trap
        }
    }

    /// Get the instruction which was being executed when the trap occured
    pub fn instruction_address(&self) -> usize
    {
        self.epc
    }

    /// Get the value associated with the trap
    pub fn get_associated_value(&self) -> usize
    {
        self.tval
    }

    /// Get the cause of the trap
    pub fn get_cause(&self) -> InterruptType
    {
        self.cause
    }

    /// Get the hart of the trap
    pub fn get_hart(&self) -> usize
    {
        self.hart
    }

    /// Get the mstatus value from the trap
    pub fn get_status(&self) -> usize
    {
        self.status
    }

    /// Get a mutable reference to the Trap Frame
    pub fn get_frame(&mut self) -> & mut TrapFrame
    {
        self.frame
    }

    /// Returns true iff the trap is async
    pub fn is_async(&self) -> bool
    {
        self.async_trap
    }
}

impl core::fmt::Display for InterruptContext
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result
    {
        writeln!(f, "Interrupt:")?;
        writeln!(f, "    Cause:       {:?}", self.cause)?;
        writeln!(f, "    Instruction: 0x{:x}", self.epc)?;
        writeln!(f, "    MTVAL:       0x{:x}", self.tval)?;
        writeln!(f, "    HART:        0x{:x}", self.hart)?;
        writeln!(f, "    Status:      0x{:x}", self.status)?;
        writeln!(f, "    Frame Ptr:   0x{:x}", self.frame as *const TrapFrame as usize)?;

        Ok(())
    }
}