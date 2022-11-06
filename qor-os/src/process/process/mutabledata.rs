use crate::{
    mem::{Page, PageTable},
    trap::TrapFrame,
};

/// Mutable Process Data
pub struct MutableProcessData {
    program_counter: usize,
    frame: TrapFrame,
    stack: &'static mut [Page],
    root: &'static PageTable,
}
