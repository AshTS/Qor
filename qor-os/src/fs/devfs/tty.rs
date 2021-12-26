use crate::*;
use crate::process::signals::*;
use crate::fs::ioctl::IOControlCommand;

use super::super::structures::*;

use crate::process::{descriptor::*, PID};

use super::tty_consts::*;

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
    pub input_flags: u32,
    pub output_flags: u32,
    pub control_flags: u32,
    pub local_flags: u32,

    pub line_discipline: u8,
    pub control_characters: [u8; 32],
    
    pub input_speed: u32,
    pub output_speed: u32
}

impl TeletypeSettings
{
    pub const fn new() -> Self
    {
        Self {
            input_flags: IXON | ICRNL,
            output_flags: OPOST,
            control_flags: 0,
            local_flags: ECHO | ICANON | ISIG | IEXTEN,
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

    fn handle_input(&mut self, byte: u8) -> bool
    {
        let settings = self.get_tty_settings();

        if settings.local_flags & IEXTEN > 0 && self.get_preserve_next_state() && (settings.input_flags & IXON == 0 || !self.get_paused_state())
        {
            self.set_preserve_next_state(false);
            return false;
        }

        if settings.input_flags & IXON > 0
        {
            if byte == 17
            {
                self.set_paused_state(false);
                return true;
            }
        }

        if self.get_paused_state() && settings.input_flags & IXON > 0 { return true; }

        if settings.input_flags & IXON > 0
        {
            if byte == 19
            {
                self.set_paused_state(true);
                return true;
            }
        }

        if settings.local_flags & IEXTEN > 0
        {
            if byte == 22
            {
                self.set_preserve_next_state(true);
                return true;
            }
        }

        if settings.local_flags & ISIG > 0
        {
            if byte == 3
            {
                if crate::process::scheduler::get_process_manager().as_mut().unwrap().send_signal_group(
                    self.get_foreground_process_group(),
                    0,
                    POSIXSignal::new(0, 0, SignalType::SIGINT)).is_err()
                {
                    kwarnln!("TTY Couldn't send SIGINT to PGID {}", self.get_foreground_process_group());
                }
                return true;
            }
            else if byte == 26
            {
                if crate::process::scheduler::get_process_manager().as_mut().unwrap().send_signal_group(
                    self.get_foreground_process_group(),
                    0,
                    POSIXSignal::new(0, 0, SignalType::SIGSTOP)).is_err()
                {
                    kwarnln!("TTY Couldn't send SIGSTOP to PGID {}", self.get_foreground_process_group());
                }
                return true;
            }
        }

        if byte >= 0x20 && byte < 0x7F
        {
            if settings.local_flags & ECHO > 0
            {
                self.tty_write_byte(byte)
            }
        }
        else if byte == 0x7F
        {
            if self.get_tty_settings().local_flags & ICANON > 0
            {
                if self.backspace()
                {
                    if settings.local_flags & ECHO > 0
                    {
                        self.tty_write_byte(0x08);
                        self.tty_write_byte(0x20);
                        self.tty_write_byte(0x08);
                    }
                }
                return true;
            }
        }
        else if byte == 0xD && settings.input_flags & ICRNL > 0
        {
            if settings.local_flags & ECHO > 0
            {
                self.tty_write_byte(0x0A);
            }
        }
        
        false
    }

    fn flush_tty(&mut self);

    fn get_foreground_process_group(&self) -> PID;
    fn set_foreground_process_group(&mut self, pgid: PID);

    fn get_paused_state(&self) -> bool;
    fn set_paused_state(&mut self, state: bool);

    fn get_preserve_next_state(&self) -> bool;
    fn set_preserve_next_state(&mut self, state: bool);

    fn bytes_to_backaspace(&self) -> bool;

    fn exec_ioctl(&mut self, cmd: IOControlCommand) -> usize
    {
        match cmd
        {
            IOControlCommand::TeletypeGetSettings { response } => 
            {
                *response = self.get_tty_settings();
                0
            },
            IOControlCommand::TeletypeSetSettingsNoWait { response } => 
            {
                self.set_tty_settings(*response);
                0
            },
            IOControlCommand::TeletypeSetSettingsDrain { .. } => 
            {
                todo!();
                // self.set_tty_settings(*response);
                // 0
            },
            IOControlCommand::TeletypeSetSettingsFlush { response } => 
            {
                self.flush_tty();

                self.set_tty_settings(*response);
                0
            }
            IOControlCommand::TeletypeGetProcessGroup {response } =>
            {
                *response = self.get_foreground_process_group();
                0
            }
            IOControlCommand::TeletypeSetProcessGroup { response } => 
            {
                self.set_foreground_process_group(*response);
                0
            }
            _ => crate::errno::ENOIOCTLCMD
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