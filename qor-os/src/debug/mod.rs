//! Flags for debug displays

use core::sync::atomic::AtomicBool;

/// Reasons for debug mode operations
#[derive(Debug, Clone, Copy)]
pub enum DebugMode
{
    Allocation,
    MemoryAllocation,
    MemoryMapping,
    PageMapping,
    KernelVirtMapping,
    Interrupts,
    VirtIO,
    BlockDevice,
    ElfParsing,
    Syscall,
    Other
}

static ALL: AtomicBool = AtomicBool::new(true);
static ALLOCATION: AtomicBool = AtomicBool::new(false);
static MEMORY_MAPPING: AtomicBool = AtomicBool::new(false);
static PAGE_MAPPING: AtomicBool = AtomicBool::new(false);
static KERNEL_MAPPING: AtomicBool = AtomicBool::new(false);
static INTERRUPTS: AtomicBool = AtomicBool::new(false);
static MEMORY_ALLOCATION: AtomicBool = AtomicBool::new(false);
static VIRTIO: AtomicBool = AtomicBool::new(false);
static BLOCK_DEVICE: AtomicBool = AtomicBool::new(false);
static ELF_PARSING: AtomicBool = AtomicBool::new(false);
static SYS_CALLS: AtomicBool = AtomicBool::new(true);

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
        DebugMode::Interrupts =>  INTERRUPTS.load(core::sync::atomic::Ordering::Relaxed),
        DebugMode::MemoryAllocation =>  MEMORY_ALLOCATION.load(core::sync::atomic::Ordering::Relaxed),
        DebugMode::VirtIO =>  VIRTIO.load(core::sync::atomic::Ordering::Relaxed),
        DebugMode::BlockDevice =>  BLOCK_DEVICE.load(core::sync::atomic::Ordering::Relaxed),
        DebugMode::ElfParsing =>  ELF_PARSING.load(core::sync::atomic::Ordering::Relaxed),
        DebugMode::Syscall =>  SYS_CALLS.load(core::sync::atomic::Ordering::Relaxed),
        DebugMode::Other =>  true,
    }
}