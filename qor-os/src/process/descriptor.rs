use crate::*;

/// File Descriptor Trait
pub trait FileDescriptor
{
    /// Close the file descriptor
    fn close(&mut self);

    /// Write to the descriptor
    fn write(&mut self, fs: &mut fs::interface::FilesystemInterface, buffer: *mut u8, count: usize) -> usize;

    /// Read from the descriptor
    fn read(&mut self, fs: &mut fs::interface::FilesystemInterface, buffer: *mut u8, count: usize) -> usize;
}

// ========== Utility File Descriptors ==========

/// Null File Descriptor
pub struct NullDescriptor;

impl FileDescriptor for NullDescriptor
{
    fn close(&mut self) {}

    fn write(&mut self, _: &mut fs::interface::FilesystemInterface, _: *mut u8, count: usize) -> usize
    {
        count
    }

    fn read(&mut self, _: &mut fs::interface::FilesystemInterface, buffer: *mut u8, count: usize) -> usize
    {
        for _ in 0..count
        {
            unsafe { buffer.add(count).write_volatile(0) }; 
        }

        count
    }
}

/// UART Out File Descriptor
pub struct UARTOut;

impl FileDescriptor for UARTOut
{
    fn close(&mut self) {}

    fn write(&mut self, _: &mut fs::interface::FilesystemInterface, buffer: *mut u8, count: usize) -> usize
    {
        for i in 0..count
        {
            kprint!("{}", unsafe { buffer.add(i).read() } as char)
        }

        count
    }

    fn read(&mut self, _: &mut fs::interface::FilesystemInterface, _buffer: *mut u8, _count: usize) -> usize
    {
        0
    }
}

/// UART Error File Descriptor
pub struct UARTError;

impl FileDescriptor for UARTError
{
    fn close(&mut self) {}
    
    fn write(&mut self, _: &mut fs::interface::FilesystemInterface, buffer: *mut u8, count: usize) -> usize
    {
        for i in 0..count
        {
            kerror!("{}", unsafe { buffer.add(i).read() } as char)
        }

        count
    }

    fn read(&mut self, _: &mut fs::interface::FilesystemInterface, _buffer: *mut u8, _count: usize) -> usize
    {
        0
    }
}

/// Filesystem Inode File Descriptor
pub struct InodeFileDescriptor(pub usize);

impl FileDescriptor for InodeFileDescriptor
{
    fn close(&mut self)
    {
        
    }

    fn write(&mut self, _fs: &mut fs::interface::FilesystemInterface, _buffer: *mut u8, _count: usize) -> usize
    {
        unimplemented!()
    }

    fn read(&mut self, fs: &mut fs::interface::FilesystemInterface, buffer: *mut u8, count: usize) -> usize
    {
        fs.read_file(self.0, buffer, count)
    }
}