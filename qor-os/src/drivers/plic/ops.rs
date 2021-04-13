use crate::*;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum PLICInterrupt
{
    Interrupt0 = 1 << 0,
    Interrupt1 = 1 << 1,
    Interrupt2 = 1 << 2,
    Interrupt3 = 1 << 3,
    Interrupt4 = 1 << 4,
    Interrupt5 = 1 << 5,
    Interrupt6 = 1 << 6,
    Interrupt7 = 1 << 7,
    Interrupt8 = 1 << 8,
    Interrupt9 = 1 << 9,
    Interrupt10 = 1 << 10,
    Interrupt11 = 1 << 11,
    Interrupt12 = 1 << 12,
    Interrupt13 = 1 << 13,
    Interrupt14 = 1 << 14,
    Interrupt15 = 1 << 15,
    Interrupt16 = 1 << 16,
    Interrupt17 = 1 << 17,
    Interrupt18 = 1 << 18,
    Interrupt19 = 1 << 19,
    Interrupt20 = 1 << 20,
    Interrupt21 = 1 << 21,
    Interrupt22 = 1 << 22,
    Interrupt23 = 1 << 23,
    Interrupt24 = 1 << 24,
    Interrupt25 = 1 << 25,
    Interrupt26 = 1 << 26,
    Interrupt27 = 1 << 27,
    Interrupt28 = 1 << 28,
    Interrupt29 = 1 << 29,
    Interrupt30 = 1 << 30,
    Interrupt31 = 1 << 31,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum PLICPriority
{
    Priority0 = 0,
    Priority1 = 1,
    Priority2 = 2,
    Priority3 = 3,
    Priority4 = 4,
    Priority5 = 5,
    Priority6 = 6,
    Priority7 = 7
}

/// Convert the bit mapped interrupt code to an interrupt
pub fn bitmapped_to_index(code: u32) -> u32
{
    let mut i = 0;

    while (code >> i) & 1 == 0
    {
        i += 1;

        if i == 32
        {
            panic!("Interrupt had no bits set");
        }
    }

    i
}

/// Enable the interrupt with the given ID
/// Safety: The base address must be a valid pointer to a PLIC MMIO device
pub unsafe fn enable_interrupt(base: usize, id: PLICInterrupt)
{
    let previous = mmio::mmio_read_int(base, 0x2000);

    mmio::mmio_write_int(base, 0x2000, previous | id as u32);
}

/// Set the given interrupt's priority to the given priority
/// Safety: The base address must be a valid pointer to a PLIC MMIO device
pub unsafe fn set_priority(base: usize, id: PLICInterrupt, priority: PLICPriority)
{
    let i = bitmapped_to_index(id as u32);

    mmio::mmio_write_int(base, 4 * i as usize, priority as u32);
}

/// Set the global threshold to the given threshold
/// Safety: The base address must be a valid pointer to a PLIC MMIO device
pub unsafe fn set_threshold(base: usize, threshold: PLICPriority)
{
    mmio::mmio_write_int(base, 0x20_0000, threshold as u32);
}

/// Get the next interrupt
/// Safety: The base address must be a valid pointer to a PLIC MMIO device
pub unsafe fn next(base: usize) -> Option<PLICInterrupt>
{
    let value = mmio::mmio_read_int(base, 0x20_0004);

    if value == 0
    {
        None
    }
    else
    {
        match value
        {
            0 => Some(PLICInterrupt::Interrupt0),
            1 => Some(PLICInterrupt::Interrupt1),
            2 => Some(PLICInterrupt::Interrupt2),
            3 => Some(PLICInterrupt::Interrupt3),
            4 => Some(PLICInterrupt::Interrupt4),
            5 => Some(PLICInterrupt::Interrupt5),
            6 => Some(PLICInterrupt::Interrupt6),
            7 => Some(PLICInterrupt::Interrupt7),
            8 => Some(PLICInterrupt::Interrupt8),
            9 => Some(PLICInterrupt::Interrupt9),
            10 => Some(PLICInterrupt::Interrupt10),
            11 => Some(PLICInterrupt::Interrupt11),
            12 => Some(PLICInterrupt::Interrupt12),
            13 => Some(PLICInterrupt::Interrupt13),
            14 => Some(PLICInterrupt::Interrupt14),
            15 => Some(PLICInterrupt::Interrupt15),
            16 => Some(PLICInterrupt::Interrupt16),
            17 => Some(PLICInterrupt::Interrupt17),
            18 => Some(PLICInterrupt::Interrupt18),
            19 => Some(PLICInterrupt::Interrupt19),
            20 => Some(PLICInterrupt::Interrupt20),
            21 => Some(PLICInterrupt::Interrupt21),
            22 => Some(PLICInterrupt::Interrupt22),
            23 => Some(PLICInterrupt::Interrupt23),
            24 => Some(PLICInterrupt::Interrupt24),
            25 => Some(PLICInterrupt::Interrupt25),
            26 => Some(PLICInterrupt::Interrupt26),
            27 => Some(PLICInterrupt::Interrupt27),
            28 => Some(PLICInterrupt::Interrupt28),
            29 => Some(PLICInterrupt::Interrupt29),
            30 => Some(PLICInterrupt::Interrupt30),
            31 => Some(PLICInterrupt::Interrupt31),
            _ => None
        }
    }
}

/// Complete a pending interrupt
/// Safety: The base address must be a valid pointer to a PLIC MMIO device
pub unsafe fn complete(base: usize, id: PLICInterrupt)
{
    let i = bitmapped_to_index(id as u32);
    mmio::mmio_write_int(base, 0x20_0004, i);
}