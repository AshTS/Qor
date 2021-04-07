use super::super::mmio;

/// Safety: if the base address is a vaild base address for a UART driver,
/// this will perform as expected.
pub unsafe fn uart_init(base: usize)
{
    // Set word length 0b11 will set an 8 bit word length
    let lcr = 0b0000011;
    mmio::mmio_write_byte(base, 3, lcr);

    // Enable the recieve buffer interrupts
    mmio::mmio_write_byte(base, 1, 0b0000001);

    // Divisor calculation
    let divisor = 592u16;
    let divisor_low = divisor & 0xFF;
    let divisor_high = (divisor & 0xFF00) >> 8;

    // Open the divisor latch
    mmio::mmio_write_byte(base, 3, lcr | 1 << 7);

    mmio::mmio_write_byte(base, 0, divisor_low as u8);
    mmio::mmio_write_byte(base, 1, divisor_high as u8);

    // Close the divisor latch
    mmio::mmio_write_byte(base, 3, lcr);
}

/// Safety: if the base address is a vaild base address for a UART driver,
/// this will perform as expected.
pub unsafe fn uart_read(base: usize) -> Option<u8>
{
    // Check if there is pending data
    if mmio::mmio_read_byte(base, 5) & 1 == 0
    {
        None
    }
    else
    {
        Some(mmio::mmio_read_byte(base, 0))
    }
}

/// Safety: if the base address is a vaild base address for a UART driver,
/// this will perform as expected.
pub unsafe fn uart_write(base: usize, data: u8)
{
    mmio::mmio_write_byte(base, 0, data);
}