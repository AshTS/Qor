use crate::{
    mem::{KernelPageBox, KernelPageSeq, PageTable},
    trap::TrapFrame,
};

/// Process Execution State
pub struct ExecutionState {
    stack: KernelPageSeq,
    frame: KernelPageBox<TrapFrame>,
    program_counter: usize,
}

impl ExecutionState {
    /// Construct a new ExecutionState from its raw components
    ///
    /// Safety: The program counter must be valid
    pub unsafe fn new(
        stack: KernelPageSeq,
        frame: KernelPageBox<TrapFrame>,
        program_counter: usize,
    ) -> Self {
        Self {
            stack,
            frame,
            program_counter,
        }
    }

    /// Get a reference to the trap frame
    pub fn trap_frame(&self) -> &TrapFrame {
        self.frame.get()
    }

    /// Get a mutable reference to the trap frame
    pub fn mut_trap_frame(&mut self) -> &mut TrapFrame {
        self.frame.get_mut()
    }

    /// Get the data for this process switch
    pub fn switch_to(&self, mem_map: *const PageTable) -> (usize, usize, usize) {
        let trap_frame = self.frame.raw() as usize;
        let satp = (8 << 60) | ((mem_map as usize) >> 12);

        (trap_frame, self.program_counter, satp)
    }
}
