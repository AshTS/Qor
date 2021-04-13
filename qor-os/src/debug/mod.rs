//! Flags for debug displays

use core::sync::atomic::AtomicBool;

/// Reasons for debug mode operations
#[derive(Debug, Clone, Copy)]
pub enum DebugMode
{
    Allocation,
    MemoryMapping,
    PageMapping,
    KernelVirtMapping,
    Other
}

static ALL: AtomicBool = AtomicBool::new(true);
static ALLOCATION: AtomicBool = AtomicBool::new(false);
static MEMORY_MAPPING: AtomicBool = AtomicBool::new(false);
static PAGE_MAPPING: AtomicBool = AtomicBool::new(false);
static KERNEL_MAPPING: AtomicBool = AtomicBool::new(false);

/// Check if a debug mode is enabled
pub fn check_debug(mode: DebugMode) -> bool
{
    if !ALL.load(core::sync::atomic::Ordering::Relaxed)
    {
        return false;
    }

    match mode
    {
        DebugMode::Allocation => ALLOCATION.load(core::sync::atomic::Ordering::Relaxed),
        DebugMode::MemoryMapping =>  MEMORY_MAPPING.load(core::sync::atomic::Ordering::Relaxed),
        DebugMode::PageMapping =>  PAGE_MAPPING.load(core::sync::atomic::Ordering::Relaxed),
        DebugMode::KernelVirtMapping =>  KERNEL_MAPPING.load(core::sync::atomic::Ordering::Relaxed),
        DebugMode::Other =>  true,
    }
}