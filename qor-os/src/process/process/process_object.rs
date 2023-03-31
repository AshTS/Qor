use libutils::sync::{semaphore::SignalSemaphoreSender, Mutex, MutexGuard, NoInterruptMarker};

use crate::{
    mem::{self, KernelPageBox, KernelPageSeq, PageCount, PAGE_SIZE},
    trap::TrapFrame,
};

use super::*;

/// Inner process structure
pub struct Process {
    state: Mutex<ProcessState>,
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
            state: Mutex::new(ProcessState::Pending),
            atomic_data,
            const_data,
            mutable_data: Mutex::new(mutable_data),
        }
    }

    /// Construct from raw values
    pub fn from_raw(fn_ptr: unsafe extern "C" fn(), stack_size: crate::mem::PageCount) -> Self {
        let pid = crate::process::next_process_id();
        kdebugln!(unsafe "Constructing Process for code at {:x} with stack of size {}, giving it PID {}", fn_ptr as usize, stack_size, pid);

        let mut page_table = KernelPageBox::new(mem::PageTable::new())
            .expect("Unable to allocate page table for process");
        page_table.identity_map(
            (fn_ptr as usize) & (!(mem::PAGE_SIZE - 1)),
            (fn_ptr as usize) & ((!(mem::PAGE_SIZE - 1)) + 1),
            mem::RWXFlags::ReadWriteExecute,
            mem::UGFlags::UserGlobal,
        );

        let stack =
            KernelPageSeq::new(stack_size).expect("Unable to allocate page table for process");

        page_table.identity_map(
            stack.raw() as usize,
            stack.raw() as usize + 1,
            mem::RWXFlags::ReadWriteExecute,
            mem::UGFlags::UserGlobal,
        );

        let mut trap_frame = TrapFrame::new(
            unsafe { NoInterruptMarker::new() },
            PageCount::new(2).convert(),
        );

        trap_frame.pid = pid;

        let mut trap_frame =
            KernelPageBox::new(trap_frame).expect("Unable to allocate space for trap stack");

        trap_frame.regs[2] = stack.raw() as usize + stack_size.raw() * PAGE_SIZE - 1;

        let execution_state = unsafe { ExecutionState::new(stack, trap_frame, fn_ptr as usize) };

        Self::new(
            AtomicProcessData::new(),
            ConstantProcessData::new(pid),
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
    // State:

    /// Get the current process state mutex
    pub fn state_mutex(&self) -> &Mutex<ProcessState> {
        &self.state
    }

    /// Get the state of the process asynchronously
    pub async fn async_state(&self) -> MutexGuard<ProcessState> {
        self.state.async_lock().await
    }

    /// Synchronously set the state of the process
    pub fn set_state(&self, state: ProcessState) {
        *self.state.spin_lock() = state;
    }

    // Atomic Data:

    /// Check the child pending semaphore
    pub fn check_child_semaphore(&self) -> bool {
        self.atomic_data.check_child_semaphore()
    }

    /// Check the optional waiting semaphore
    pub fn check_wait_semaphore(&self) -> Option<bool> {
        self.atomic_data.check_wait_semaphore()
    }

    /// Get a new sender for the wait semaphore
    pub fn new_wait_semaphore(&self) -> SignalSemaphoreSender {
        self.atomic_data.new_wait_semaphore()
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

impl core::ops::Drop for Process {
    fn drop(&mut self) {
        kdebugln!(unsafe "Dropping PID {}", self.pid());
    }
}
