use crate::*;

/// Stdin buffer
pub static mut STDIN_BUFFER: utils::ByteRingBuffer = utils::ByteRingBuffer::new();

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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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

/// UART In File Descriptor
#[derive(Debug, Clone)]
pub struct UARTIn;

impl FileDescriptor for UARTIn
{
    fn close(&mut self) {}
    
    fn write(&mut self, _: &mut fs::interface::FilesystemInterface, _buffer: *mut u8, _count: usize) -> usize
    {
        0
    }

    fn read(&mut self, _: &mut fs::interface::FilesystemInterface, buffer: *mut u8, count: usize) -> usize
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