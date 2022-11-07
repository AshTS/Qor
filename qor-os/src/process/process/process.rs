use libutils::sync::{Mutex, MutexGuard, NoInterruptMarker};

use crate::{
    mem::{self, KernelPageBox, KernelPageSeq, PageCount, PAGE_SIZE},
    trap::TrapFrame,
};

use super::*;

/// Inner process structure
pub struct Process {
    atomic_data: AtomicProcessData,
    const_data: ConstantProcessData,
    mutable_data: Mutex<MutableProcessData>,
}

// Constructors:

impl Process {
    /// Construct a new `Process` from the constituent components
    pub fn new(
        atomic_data: AtomicProcessData,
        const_data: ConstantProcessData,
        mutable_data: MutableProcessData,
    ) -> Self {
        Self {
            atomic_data,
            const_data,
            mutable_data: Mutex::new(mutable_data),
        }
    }

    /// Construct from raw values
    pub fn from_raw(fn_ptr: unsafe extern "C" fn(), stack_size: crate::mem::PageCount) -> Self {
        kdebugln!(unsafe "Constructing Process for code at {:x} with stack of size {}", fn_ptr as usize, stack_size);

        let mut page_table = KernelPageBox::new(mem::PageTable::new())
            .expect("Unable to allocate page table for process");
        page_table.identity_map(
            (fn_ptr as usize) & (!(mem::PAGE_SIZE - 1)),
            (fn_ptr as usize) & (!(mem::PAGE_SIZE - 1)) + 1,
            mem::RWXFlags::ReadWriteExecute,
            mem::UGFlags::UserGlobal,
        );

        let stack =
            KernelPageSeq::new(stack_size).expect("Unable to allocate page table for process");

        let trap_frame = TrapFrame::new(
            unsafe { NoInterruptMarker::new() },
            PageCount::new(2).convert(),
        );

        let mut trap_frame =
            KernelPageBox::new(trap_frame).expect("Unable to allocate space for trap stack");

        trap_frame.regs[2] = stack.raw() as usize + stack_size.raw() * PAGE_SIZE - 1;

        let execution_state = unsafe { ExecutionState::new(stack, trap_frame, fn_ptr as usize) };

        Self::new(
            AtomicProcessData::new(),
            ConstantProcessData::new(crate::process::next_process_id()),
            MutableProcessData::new(execution_state, page_table),
        )
    }

    /// Switch to this process
    pub unsafe fn switch_to_process(&self) -> ! {
        let data = self.mutable_data.spin_lock().get_proc_switch_data();

        crate::asm::switch_to_user(data.0, data.1, data.2)
    }
}

// Getters and Setters
impl Process {
    // Atomic Data:

    /// Get the current process state
    pub fn state(&self) -> ProcessState {
        self.atomic_data.state()
    }

    /// Set the current process state
    pub fn set_state(&self, state: ProcessState) {
        self.atomic_data.set_state(state)
    }

    // Const Data:

    /// Get the Process Identifier of a process
    pub fn pid(&self) -> ProcessIdentifier {
        self.const_data.pid()
    }

    // Mutable Data:

    /// Get the mutex guard for the mutable data
    pub fn lock_mutable(&self) -> MutexGuard<'_, MutableProcessData> {
        self.mutable_data.spin_lock()
    }
}
