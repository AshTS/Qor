use crate::*;

use fs::fstrait::Filesystem;

use fs::structures::FilesystemIndex;
use alloc::collections::BTreeMap;

/// Descriptor table type
pub type DescriptorTable = alloc::sync::Arc<core::cell::RefCell<BTreeMap<usize, Box<dyn super::descriptor::FileDescriptor>>>>;

/// Seek Modes
#[derive(Debug, Clone, Copy)]
pub enum SeekMode
{
    SeekSet,
    SeekCurrent,
    SeekEnd
}

/// Stdin buffer
pub static mut STDIN_BUFFER: utils::ByteRingBuffer = utils::ByteRingBuffer::new();

/// File Descriptor Trait
pub trait FileDescriptor
{
    /// Close the file descriptor
    fn close(&mut self, fs: &mut fs::vfs::FilesystemInterface);

    /// Write to the descriptor
    fn write(&mut self, fs: &mut fs::vfs::FilesystemInterface, buffer: *mut u8, count: usize) -> usize;

    /// Read from the descriptor
    fn read(&mut self, fs: &mut fs::vfs::FilesystemInterface, buffer: *mut u8, count: usize) -> usize;

    /// Get the inode of the entry
    fn get_inode(&mut self) -> Option<FilesystemIndex>
    {
        None
    }

    /// Seek to the given location in the descriptor
    fn seek(&mut self, offset: usize, _mode: SeekMode) -> usize
    {
        offset
    }
}

// ========== Utility File Descriptors ==========

/// Null File Descriptor
#[derive(Debug, Clone)]
pub struct NullDescriptor;

impl FileDescriptor for NullDescriptor
{
    fn close(&mut self, _: &mut fs::vfs::FilesystemInterface) {}

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
    fn close(&mut self, _: &mut fs::vfs::FilesystemInterface) {}

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
    fn close(&mut self, _: &mut fs::vfs::FilesystemInterface) {}
    
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
    fn close(&mut self, _: &mut fs::vfs::FilesystemInterface) {}
    
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
    data: Vec<u8>,
    is_write: bool,
    is_read: bool
}

// Must be kept in sync with syscalls.h
const O_RDONLY: usize = 1;
const O_WRONLY: usize = 2;
const O_APPEND: usize = 4;
const O_TRUNC: usize =  8;
const O_CREAT: usize =  16;
const O_EXCL: usize =   32;

impl InodeFileDescriptor
{
    pub fn new(fs: &mut fs::vfs::FilesystemInterface, inode: FilesystemIndex, mode: usize) -> Result<Self, ()>
    {
        let mut temp = Self
        {
            inode,
            index: 0,
            data: Vec::new(),
            is_write: mode & O_WRONLY > 0,
            is_read: mode & O_RDONLY > 0
        };

        if (temp.is_read || (mode & O_APPEND) > 0) && (mode & O_TRUNC) == 0
        {
            if let Ok(data) = fs.read_inode(temp.inode)
            {
                temp.data = data;
            }
            else
            {
                return Err(());
            }
        }

        Ok(temp)
    }
}

impl FileDescriptor for InodeFileDescriptor
{
    fn close(&mut self, fs: &mut fs::vfs::FilesystemInterface)
    {
        if self.is_write
        {
            fs.write_inode(self.inode, &self.data).unwrap();
        }
    }

    fn write(&mut self, _fs: &mut fs::vfs::FilesystemInterface, buffer: *mut u8, count: usize) -> usize
    {
        if !self.is_write
        {
            return usize::MAX;
        }


        for i in 0..count
        {
            let value = unsafe { buffer.add(i).read() };

            if self.index < self.data.len()
            {
                self.data[self.index] = value;
            }
            else
            {
                self.data.push(value);
            }

            self.index += 1;
        }

        count
    }

    // TODO: This read implementation is beyond inefficent
    fn read(&mut self, _fs: &mut fs::vfs::FilesystemInterface, buffer: *mut u8, count: usize) -> usize
    {
        if !self.is_read
        {
            return usize::MAX;
        }

        let mut written = 0;

        while self.index < self.data.len()
        {
            let data = self.data[self.index];
            unsafe { buffer.add(written).write(data) };

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

    /// Seek to the given location in the descriptor
    fn seek(&mut self, offset: usize, mode: SeekMode) -> usize
    {
        match mode
        {
            SeekMode::SeekSet => 
            {
                self.index = offset;
                self.index
            },
            SeekMode::SeekCurrent => 
            {
                self.index += offset;
                self.index
            },
            SeekMode::SeekEnd => 
            {
                self.index = self.data.len() - 1 + offset;
                self.data.extend_from_slice(&vec![0; offset]);

                self.index
            },
        }
    }
}

/// Byte interface wrapper
pub struct ByteInterfaceDescriptor
{
    interface: &'static mut dyn crate::drivers::generic::ByteInterface
}

impl ByteInterfaceDescriptor
{
    /// Create a new ByteInterfaceDescriptor
    pub fn new(interface: &'static mut dyn crate::drivers::generic::ByteInterface) -> Self
    {
        Self
        {
            interface
        }
    }
}

impl FileDescriptor for ByteInterfaceDescriptor
{
    fn close(&mut self, _: &mut fs::vfs::FilesystemInterface)
    {
        self.interface.flush()
    }
    
    fn write(&mut self, _: &mut fs::vfs::FilesystemInterface, buffer: *mut u8, count: usize) -> usize
    {
        for i in 0..count
        {
            self.interface.write_byte(unsafe { buffer.add(i).read() });
        }

        count
    }

    fn read(&mut self, _: &mut fs::vfs::FilesystemInterface, buffer: *mut u8, count: usize) -> usize
    {
        let mut i = 0;

        while i < count
        {
            if let Some(byte) = self.interface.read_byte()
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

/// Buffer descriptor
pub struct BufferDescriptor
{
    buffer: &'static mut dyn crate::drivers::generic::BufferInterface,
    index: usize
}

impl BufferDescriptor
{
    /// Create a new buffer descriptor
    pub fn new(buffer: &'static mut dyn crate::drivers::generic::BufferInterface) -> Self
    {
        Self
        {
            buffer,
            index: 0,
        }
    }
}

impl FileDescriptor for BufferDescriptor
{
    fn get_inode(&mut self) -> Option<FilesystemIndex>
    {
        None
    }

    fn seek(&mut self, offset: usize, mode: SeekMode) -> usize
    {
        match mode
        {
            SeekMode::SeekSet => 
            {
                if offset >= self.buffer.get_size()
                {
                    return offset - 1;
                }

                self.index = offset;
                self.index
            },
            SeekMode::SeekCurrent => 
            {
                if self.index + offset >= self.buffer.get_size()
                {
                    return offset - 1;
                }

                self.index += offset;
                self.index
            },
            SeekMode::SeekEnd => 
            {
                if offset != 0
                {
                    return offset - 1;
                }

                self.index = self.buffer.get_size() - 1;
                self.index
            },
        }
    }

    fn close(&mut self, _fs: &mut fs::vfs::FilesystemInterface)
    {
        self.buffer.flush();
    }

    fn write(&mut self, _fs: &mut fs::vfs::FilesystemInterface, buffer: *mut u8, count: usize) -> usize
    {
        for i in 0..count
        {
            let value = unsafe { buffer.add(i).read() };

            self.buffer.write_byte(self.index, value);

            if self.index >= self.buffer.get_size()
            {
                return i + 1;
            }

            self.index += 1;
        }

        count
    }

    fn read(&mut self, _fs: &mut fs::vfs::FilesystemInterface, buffer: *mut u8, count: usize) -> usize
    {
        for i in 0..count
        {
            if let Some(value) = self.buffer.read_byte(self.index)
            {
                unsafe { buffer.add(i).write(value) };
            }

            if self.index >= self.buffer.get_size()
            {
                return i + 1;
            }

            self.index += 1;
        }

        count
    }
}