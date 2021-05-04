use crate::*;

/// Dump a page of memory
pub fn dump_page(ptr: *mut u8, size: usize)
{
    kprint!("\x1b[33m");

    let ptr = (ptr as usize & !(4095)) as *mut u8;

    for row in 0..(size + 15) / 16
    {
        kprint!("  ");

        for col in 0..16
        {
            kprint!("{:02x} ", unsafe { ptr.add(row * 16 + col).read() });

            if col % 8 == 7
            {
                kprint!("  ");
            }
        }

        for col in 0..16
        {
            let v = unsafe { ptr.add(row * 16 + col).read() };

            kprint!("{}",
                match v
                {
                    32..=127 => v as char,
                    _ => '.'
                });
        }

        kprintln!();
    }

    kprint!("\x1b[0m");
}