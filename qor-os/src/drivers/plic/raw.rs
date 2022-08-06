use crate::drivers::{plic::InterruptID, InterruptPriority};

const PRIORITY_BANK: usize = 0x0000;
const ENABLE_BANK: usize = 0x2000;
const THRESHOLD: usize = 0x20_0000;
const CLAIM: usize = 0x20_0004;

/// Enable the interrupt with the given id
/// 
/// Safety: The base address must be a valid base address for a PLIC driver, in addition, accesses from multiple threads can overlap safely as the changes are atomic
pub unsafe fn enable_interrupt_id(base_address: usize, id: InterruptID)
{
    let address = base_address + ENABLE_BANK + 4 * (id as usize / 32);
    let bit_mask = 1 << (id % 32);

    let ptr = address as *mut u32;
    ptr.write_volatile(ptr.read_volatile() | bit_mask);
}

/// Disable the interrupt with the given id
/// 
/// Safety: The base address must be a valid base address for a PLIC driver, in addition, accesses from multiple threads can overlap safely as the changes are atomic
pub unsafe fn disable_interrupt_id(base_address: usize, id: InterruptID)
{
    let address = base_address + ENABLE_BANK + 4 * (id as usize / 32);
    let bit_mask = 1 << (id % 32);

    let ptr = address as *mut u32;
    ptr.write_volatile(ptr.read_volatile() & !bit_mask);
}

/// Set the priority for the interrupt with the given id
/// 
/// Safety: The base address must be a valid base address for a PLIC driver, in addition, accesses from multiple threads can overlap safely as the changes are atomic
pub unsafe fn set_interrupt_priority(base_address: usize, id: InterruptID, priority: InterruptPriority)
{
    let address = base_address + PRIORITY_BANK + id as usize * 4;
    
    let ptr = address as *mut u32;
    ptr.write_volatile(priority as u32);
}

/// Set the threshold of the PLIC
/// 
/// Safety: The base address must be a valid base address for a PLIC driver, in addition, accesses from multiple threads can overlap safely as the changes are atomic
pub unsafe fn set_threshold(base_address: usize, threshold: InterruptPriority)
{
    let address = base_address + THRESHOLD;

    let ptr = address as *mut u32;
    ptr.write_volatile(threshold as u32);
}

/// Get the next interrupt from the PLIC in order of priority
/// 
/// Safety: The base address must be a valid base address for a PLIC driver, in addition, accesses from multiple threads can overlap safely as the changes are atomic
pub unsafe fn next_interrupt(base_address: usize) -> Option<InterruptID>
{
    let address = base_address + CLAIM;

    let ptr = address as *mut u32;
    
    match ptr.read_volatile()
    {
        0 => None,
        value => Some(value)
    }
}

/// Claim an interrupt
/// 
/// Safety: The base address must be a valid base address for a PLIC driver, in addition, accesses from multiple threads can overlap safely as the changes are atomic
pub unsafe fn claim_interrupt(base_address: usize, id: InterruptID)
{
    let address = base_address + CLAIM;

    let ptr = address as *mut u32;
    ptr.write_volatile(id);
}