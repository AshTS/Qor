//! Test Running Framework
#[cfg(test)]
use crate::*;

/// Trait for all tests
#[cfg(test)]
pub trait TestFunction {
    fn run(&self);
}

// Implement testable
#[cfg(test)]
impl<T: Fn()> TestFunction for T {
    fn run(&self) {
        crate::kprint!(unsafe "Running Test {}......\t", core::any::type_name::<T>());
        self();
        crate::kprintln!(unsafe "\x1b[32m[OK]\x1b[m");
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

    halt::kernel_halt();
}
