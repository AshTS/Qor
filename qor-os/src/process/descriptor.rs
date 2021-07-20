use crate::*;

use fs::fstrait::Filesystem;

use fs::structures::FilesystemIndex;

/// Stdin buffer
pub static mut STDIN_BUFFER: utils::ByteRingBuffer = utils::ByteRingBuffer::new();

/// File Descriptor Trait
pub trait FileDescriptor
{
    /// Close the file descriptor
    fn close(&mut self);

    /// Write to the descriptor
    fn write(&mut self, fs: &mut fs::vfs::FilesystemInterface, buffer: *mut u8, count: usize) -> usize;

    /// Read from the descriptor
    fn read(&mut self, fs: &mut fs::vfs::FilesystemInterface, buffer: *mut u8, count: usize) -> usize;

    /// Get the inode of the entry
    fn get_inode(&mut self) -> Option<FilesystemIndex>
    {
        None
    }
}

// ========== Utility File Descriptors ==========

/// Null File Descriptor
#[derive(Debug, Clone)]
pub struct NullDescriptor;

impl FileDescriptor for NullDescriptor
{
    fn close(&mut self) {}

    fn write(&mut self, _: &mut fs::vfs::FilesystemInterface, _: *mut u8, count: usize) -> usize
    {
        count
    }

    fn read(&mut self, _: &mut fs::vfs::FilesystemInterface, buffer: *mut u8, count: usize) -> usize
    {
        for _ in 0..count
        {
            unsafe { buffer.add(count).write_volatile(0) }; 
        }

        count
    }
}

/// UART Out File Descriptor
#[derive(Debug, Clone)]
pub struct UARTOut;

impl FileDescriptor for UARTOut
{
    fn close(&mut self) {}

    fn write(&mut self, _: &mut fs::vfs::FilesystemInterface, buffer: *mut u8, count: usize) -> usize
    {
        for i in 0..count
        {
            kprint!("{}", unsafe { buffer.add(i).read() } as char)
        }

        count
    }

    fn read(&mut self, _: &mut fs::vfs::FilesystemInterface, _buffer: *mut u8, _count: usize) -> usize
    {
        0
    }
}

/// UART Error File Descriptor
#[derive(Debug, Clone)]
pub struct UARTError;

impl FileDescriptor for UARTError
{
    fn close(&mut self) {}
    
    fn write(&mut self, _: &mut fs::vfs::FilesystemInterface, buffer: *mut u8, count: usize) -> usize
    {
        for i in 0..count
        {
            kerror!("{}", unsafe { buffer.add(i).read() } as char)
        }

        count
    }

    fn read(&mut self, _: &mut fs::vfs::FilesystemInterface, _buffer: *mut u8, _count: usize) -> usize
    {
        0
    }
}

/// UART In File Descriptor
#[derive(Debug, Clone)]
pub struct UARTIn;

impl FileDescriptor for UARTIn
{
    fn close(&mut self) {}
    
    fn write(&mut self, _: &mut fs::vfs::FilesystemInterface, _buffer: *mut u8, _count: usize) -> usize
    {
        0
    }

    fn read(&mut self, _: &mut fs::vfs::FilesystemInterface, buffer: *mut u8, count: usize) -> usize
    {
        let mut i = 0;

        while i < count
        {
            if let Some(byte) = unsafe { STDIN_BUFFER.dequeue_byte() }
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
}

/// Filesystem Inode File Descriptor
#[derive(Debug, Clone)]
pub struct InodeFileDescriptor
{
    pub inode: FilesystemIndex,
    index: usize,
    data: Vec<u8>
}

impl InodeFileDescriptor
{
    pub fn new(inode: FilesystemIndex) -> Self
    {
        Self
        {
            inode,
            index: 0,
            data: Vec::new()
        }
    }
}

impl FileDescriptor for InodeFileDescriptor
{
    fn close(&mut self)
    {
        
    }

    fn write(&mut self, _fs: &mut fs::vfs::FilesystemInterface, _buffer: *mut u8, _count: usize) -> usize
    {
        unimplemented!()
    }

    // TODO: This read implementation is beyond inefficent
    fn read(&mut self, fs: &mut fs::vfs::FilesystemInterface, buffer: *mut u8, count: usize) -> usize
    {
        if self.data.len() == 0
        {
            if let Ok(data) = fs.read_inode(self.inode)
            {
                self.data = data;
            }
            else
            {
                return usize::MAX;
            }
        }

        let mut written = 0;

        while self.index < self.data.len()
        {
            unsafe { buffer.add(self.index).write(self.data[self.index]) };

            written += 1;
            self.index += 1;

            if written == count
            {
                break;
            }
        }

        written
    }

    /// Get the inode of the entry
    fn get_inode(&mut self) -> Option<FilesystemIndex>
    {
        Some(self.inode)
    }
}