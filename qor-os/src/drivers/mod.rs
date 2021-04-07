mod uart;

lazy_static::lazy_static!
{
    // Safety: The QEMU emulator has a UART mmio interface at 0x1000_0000
    pub static ref UART_DRIVER: spin::Mutex<uart::UartDriver> = 
        spin::Mutex::new(unsafe{uart::UartDriver::new(0x1000_0000)});
}