//! Test Running Framework
use crate::*;
use crate::harts::machine_mode_sync;

/// Trait for all tests
pub trait TestFunction {
    fn run(&self);
    fn sync_run(&self);
}

// Implement testable
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
pub fn test_runner(tests: &[&dyn TestFunction]) {
    kprintln!(unsafe "Running {} Tests", tests.len());

    for test in tests {
        test.run();
    }

    kprintln!(unsafe "Testing Complete");
}

/// Test Runner
fn priv_sync_test_runner(tests: &[&dyn TestFunction]) {
    let is_primary_hart = riscv::register::mhartid::read() == 0;

    if is_primary_hart {
        kprintln!(unsafe "Running {} Sync Tests", tests.len());
    }

    for test in tests {
        test.sync_run();
    }

    machine_mode_sync();

    if is_primary_hart {
        kprintln!(unsafe "Sync Testing Complete");
    }
}

pub fn sync_test_runner() {
    priv_sync_test_runner(&[&crate::mem::bump::sync_test::collective_test]);
}

/// Finish Testing
pub fn finish_testing() {
    kprintln!(unsafe "All Testing Complete");
    halt::kernel_halt();
}