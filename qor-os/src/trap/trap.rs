#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapCause {
    InstructionAddressMisaligned = 0x0000000000000000,
    InstructionAccessFault = 0x0000000000000001,
    IllegalInstruction = 0x0000000000000002,
    BreakPoint = 0x0000000000000003,
    LoadAddressMisaligned = 0x0000000000000004,
    LoadAccessFault = 0x0000000000000005,
    StoreAddressMisaligned = 0x0000000000000006,
    StoreAccessFault = 0x0000000000000007,
    UserEnvironmentCall = 0x0000000000000008,
    SupervisorEnvironmentCall = 0x0000000000000009,
    MachineEnvironmentCall = 0x000000000000000b,
    InstructionPageFault = 0x000000000000000c,
    LoadPageFault = 0x000000000000000d,
    StorePageFault = 0x000000000000000f,

    SupervisorInterrupt = 0x8000000000000001,
    MachineInterrupt = 0x8000000000000003,
    SupervisorTimer = 0x8000000000000005,
    MachineTimer = 0x8000000000000007,
    SupervisorExternal = 0x8000000000000009,
    MachineExternal = 0x800000000000000b,
}
