//! Raw functions for memory mapped UART
use crate::drivers::mmio;

const UART_IIR: usize = 1;
const UART_FCR: usize = 2;
const UART_LCR: usize = 3;
const UART_LSR: usize = 5;

/// Initialize the UART driver at the given base address
///
/// Safety: The base address must be a proper UART driver base address, and we must have a gaurentee that no other thread could be using this UART port
pub unsafe fn init(base_address: usize) {
    // Set the word length to 8 bits via the LCR register
    let lcr = 0b11;
    mmio::write_offset::<u8>(base_address, UART_LCR, lcr);

    // Enable the FIFO for UART IO
    mmio::write_offset::<u8>(base_address, UART_FCR, 0b1);

    // Enable the recieve buffer interrupt
    mmio::write_offset::<u8>(base_address, UART_IIR, 0b1);

    // Open the divisor latch
    mmio::write_offset::<u8>(base_address, UART_LCR, lcr | (1 << 7));

    // Write the clock divisor
    let divisor = 592u16;
    let divisor_low = divisor & 0xff;
    let divisor_high = (divisor & 0xff00) >> 8;

    // Write the low byte
    mmio::write_offset::<u8>(base_address, 0, divisor_low as u8);

    // Write the high byte
    mmio::write_offset::<u8>(base_address, 1, divisor_high as u8);

    // Close the divisor latch
    mmio::write_offset::<u8>(base_address, UART_LCR, lcr);
}

/// Read a byte from the UART port
///
/// Safety: The base address must be a proper UART driver base address, where the port has been initialized, and we must have a gaurentee that no other thread could be using this UART port
pub unsafe fn read_byte(base_address: usize) -> Option<u8> {
    // Check the input pending bit
    if mmio::read_offset::<u8>(base_address, UART_LSR) & 1 == 0 {
        None
    } else {
        Some(mmio::read_offset::<u8>(base_address, 0))
    }
}

/// Write a byte to the UART port
///
/// Safety: The base address must be a proper UART driver base address, where the port has been initialized, and we must have a gaurentee that no other thread could be using this UART port
pub unsafe fn write_byte(base_address: usize, byte: u8) {
    mmio::write_offset::<u8>(base_address, 0, byte)
}
