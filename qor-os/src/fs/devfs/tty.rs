use crate::*;

use super::super::structures::*;

use crate::process::descriptor::*;

/// Get the file descriptor for the pseudo terminal secondary with the given
/// index
pub fn get_pseudo_terminal_secondary_file_descriptor(_index: usize, _inode: FilesystemIndex) -> FilesystemResult<Box<dyn crate::process::descriptor::FileDescriptor>>
{
    todo!()
}

/// Get the open pseudo terminal indexes
pub fn get_open_pseudo_terminal_indexes() -> Vec<usize>
{
    vec![]
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TeletypeSettings
{
    input_flags: u32,
    output_flags: u32,
    control_flags: u32,
    local_flags: u32,

    line_discipline: u8,
    control_characters: [u8; 32],
    
    input_speed: u32,
    output_speed: u32
}

impl TeletypeSettings
{
    pub const fn new() -> Self
    {
        Self {
            input_flags: 0,
            output_flags: 0,
            control_flags: 0,
            local_flags: 0,
            line_discipline: 0,
            control_characters: [0; 32],
            input_speed: 0,
            output_speed: 0
        }
    }
}

pub trait TeletypeDevice
{
    fn tty_read_byte(&mut self) -> Option<u8>;
    fn tty_write_byte(&mut self, byte: u8);
    fn tty_push_byte(&mut self, byte: u8);
    fn tty_pop_byte(&mut self) -> Option<u8>;
    fn tty_close(&mut self);

    fn get_tty_settings(&self) -> TeletypeSettings;
    fn set_tty_settings(&mut self, settings: TeletypeSettings);

    fn bytes_available(&self) -> bool;

    fn backspace(&mut self) -> bool;

    fn handle_input(&mut self, byte: u8)
    {
        let _settings = self.get_tty_settings();

        if byte >= 0x20 && byte < 0x7F
        {
            self.tty_write_byte(byte)
        }
        else if byte == 0x7F
        {
            if self.backspace()
            {
                self.tty_write_byte(0x08);
                self.tty_write_byte(0x20);
                self.tty_write_byte(0x08);
            }
        }
        else if byte == 0xD
        {
            self.tty_write_byte(0xA);
            self.tty_write_byte(0xD);
        }
    }
}

struct UARTTeletype
{
    
}

impl UARTTeletype
{
    pub const fn new() -> Self
    {
        Self {}
    }
}

/// Byte interface wrapper
pub struct TeletypeSecondaryDescriptor
{
    teletype: &'static mut dyn TeletypeDevice,
    inode: FilesystemIndex
}

impl TeletypeSecondaryDescriptor
{
    /// Create a new ByteInterfaceDescriptor
    pub fn new(teletype: &'static mut dyn TeletypeDevice, inode: FilesystemIndex) -> Self
    {
        Self
        {
            teletype,
            inode
        }
    }
}

impl FileDescriptor for TeletypeSecondaryDescriptor
{
    fn close(&mut self, _: &mut fs::vfs::FilesystemInterface)
    {
        self.teletype.tty_close();
    }
    
    fn write(&mut self, _: &mut fs::vfs::FilesystemInterface, buffer: *mut u8, count: usize) -> usize
    {
        for i in 0..count
        {
            self.teletype.tty_write_byte(unsafe { buffer.add(i).read() });
        }

        count
    }

    fn read(&mut self, _: &mut fs::vfs::FilesystemInterface, buffer: *mut u8, count: usize) -> usize
    {
        let mut i = 0;

        while i < count
        {
            if let Some(byte) = self.teletype.tty_read_byte()
            {
                unsafe { buffer.add(i).write(byte) };
                i += 1;
            }
            else
            {
                break;
            }
        }

        i
    }

    fn get_inode(&mut self) -> Option<FilesystemIndex>
    {
        Some(self.inode)
    }

    fn check_available(&self) -> bool
    {
        self.teletype.bytes_available()
    }
}

impl core::ops::Drop for TeletypeSecondaryDescriptor
{
    fn drop(&mut self)
    {
        unsafe { Box::from_raw(self.teletype as *mut dyn TeletypeDevice); }
    }
}