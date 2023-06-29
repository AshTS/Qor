//! Test Running Framework
#[cfg(test)]
use crate::*;
use crate::harts::machine_mode_sync;

/// Trait for all tests
#[cfg(test)]
pub trait TestFunction {
    fn run(&self);
    fn sync_run(&self);
}

#[cfg(test)]
pub static SYNC_TEST_FLAG: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(true);

// Implement testable
#[cfg(test)]
impl<T: Fn()> TestFunction for T {
    fn run(&self) {
        crate::kprint!(unsafe "Running Test {}......\t", core::any::type_name::<T>());
        self();
        crate::kprintln!(unsafe "\x1b[32m[OK]\x1b[m");
    }
    
    fn sync_run(&self) {
        let is_primary_hart = riscv::register::mhartid::read() == 0;

        machine_mode_sync();

        if is_primary_hart {
            crate::kprint!(unsafe "Running Sync Test {}......\t", core::any::type_name::<T>());
        }

        self();
        machine_mode_sync();
         
        if is_primary_hart {
            crate::kprintln!(unsafe "\x1b[32m[DONE]\x1b[m");
        }
    }
}

/// Test Runner
#[cfg(test)]
pub fn test_runner(tests: &[&dyn TestFunction]) {
    kprintln!(unsafe "Running {} Tests", tests.len());

    for test in tests {
        test.run();
    }

    kprintln!(unsafe "Testing Complete");
}

/// Test Runner
#[cfg(test)]
pub fn sync_test_runner(tests: &[&dyn TestFunction]) {
    let is_primary_hart = riscv::register::mhartid::read() == 0;

    if is_primary_hart {
        kprintln!(unsafe "Running {} Sync Tests", tests.len());
    }

    for test in tests {
        test.sync_run();
    }

    if is_primary_hart {
        SYNC_TEST_FLAG.store(false, core::sync::atomic::Ordering::Release);
        SYNC_TEST_FLAG.store(true, core::sync::atomic::Ordering::Release);
        kprintln!(unsafe "Sync Testing Complete");
    }
}

/// Finish Testing
#[cfg(test)]
pub fn finish_testing() {
    kprintln!(unsafe "All Testing Complete");
    halt::kernel_halt();
}