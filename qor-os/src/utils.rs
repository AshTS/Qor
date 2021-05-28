use crate::*;

/// Dump a region of memory
pub fn mem_dump(address: *mut u8, size: usize)
{
    let start_row = address as usize & !15;
    let num_rows = (size + (start_row & 15)) / 16;

    for row in 0..num_rows
    {
        kprint!(" {:x}\t", start_row / 16 + row);

        let row: &[u8; 16] = unsafe{ (address as *mut [u8; 16]).add(row).as_ref().unwrap() };

        for i in 0..16
        {
            kprint!("{:02X} ", row[i]);

            if i == 7
            {
                kprint!(" ");
            }
        }

        kprint!("     ");

        for i in 0..16
        {
            kprint!("{}", 
                match row[i]
                {
                    32..=127 => row[i] as char,
                    _ => '.'
                }
            );
        }

        kprintln!();
    }
}