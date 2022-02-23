use crate::*;
use super::descriptor::*;

use crate::fs::structures::FilesystemIndex;

/// Write side of a pipe
pub struct WritePipeDescriptor
{
    buffer: alloc::sync::Arc<core::cell::RefCell<utils::ByteRingBuffer>>,
    read_end: Option<alloc::sync::Weak<core::cell::RefCell<Box<dyn FileDescriptor>>>>
}

impl WritePipeDescriptor
{
    fn is_end_closed(&self) -> bool
    {
        if let Some(end) = &self.read_end
        {
            end.upgrade().is_none()
        }
        else
        {
            false
        }
    }
}

impl FileDescriptor for WritePipeDescriptor
{
    fn close(&mut self, _fs: &mut fs::vfs::FilesystemInterface)
    {
        // Nothing needs to be done but drop this side, which will occur elsewhere
    }

    fn write(&mut self, _fs: &mut fs::vfs::FilesystemInterface, buffer: *mut u8, count: usize) -> usize
    {
        if self.is_end_closed()
        {
            return errno::EPIPE;
        }

        for i in 0..count
        {
            self.buffer.borrow_mut().enqueue_byte(unsafe { buffer.add(i).read() });
        }

        count
    }

    fn read(&mut self, _fs: &mut fs::vfs::FilesystemInterface, _buffer: *mut u8, _count: usize) -> usize
    {
        // Cannot read from the write end
        0
    }

    fn get_inode(&mut self) -> Option<FilesystemIndex>
    {
        None
    }

    fn set_end(&mut self, end: &alloc::sync::Arc<core::cell::RefCell<Box<dyn FileDescriptor>>>)
    {
        self.read_end = Some(alloc::sync::Arc::<core::cell::RefCell::<Box<dyn FileDescriptor>>>::downgrade(end));
    }
}

/// Read side of a pipe
pub struct ReadPipeDescriptor
{
    buffer: alloc::sync::Arc<core::cell::RefCell<utils::ByteRingBuffer>>,
    write_end: Option<alloc::sync::Weak<core::cell::RefCell<Box<dyn FileDescriptor>>>>
}

impl ReadPipeDescriptor
{
    fn is_end_closed(&self) -> bool
    {
        if let Some(end) = &self.write_end
        {
            end.upgrade().is_none()
        }
        else
        {
            false
        }
    }
}

impl FileDescriptor for ReadPipeDescriptor
{
    fn close(&mut self, _fs: &mut fs::vfs::FilesystemInterface)
    {
        // Nothing needs to be done but drop this side, which will occur elsewhere
    }

    fn write(&mut self, _fs: &mut fs::vfs::FilesystemInterface, _buffer: *mut u8, _count: usize) -> usize
    {
        // Cannot write to the read end
        0
    }

    fn read(&mut self, _fs: &mut fs::vfs::FilesystemInterface, buffer: *mut u8, count: usize) -> usize
    {
        for i in 0..count
        {
            if let Some(data) = self.buffer.borrow_mut().dequeue_byte()
            {
                unsafe { buffer.add(i).write(data) }
            }
            else
            {
                return i;
            }
        }

        count
    }

    fn get_inode(&mut self) -> Option<FilesystemIndex>
    {
        None
    }

    fn check_available(&self) -> bool
    {
        !self.buffer.borrow_mut().is_empty() || self.is_end_closed()
    }

    fn set_end(&mut self, end: &alloc::sync::Arc<core::cell::RefCell<Box<dyn FileDescriptor>>>)
    {
        self.write_end = Some(alloc::sync::Arc::<core::cell::RefCell::<Box<dyn FileDescriptor>>>::downgrade(end));
    }
}

/// Create a new pipe pair
pub fn new_pipe() -> (alloc::sync::Arc<core::cell::RefCell<Box<dyn FileDescriptor>>>, alloc::sync::Arc<core::cell::RefCell<Box<dyn FileDescriptor>>>)
{
    let buffer = utils::ByteRingBuffer::new();
    let wrapped_buffer = 
        alloc::sync::Arc::new(
            core::cell::RefCell::new(
                buffer));
    
    let read = alloc::sync::Arc::new(core::cell::RefCell::new(Box::new(ReadPipeDescriptor { buffer: wrapped_buffer.clone(), write_end: None }) as Box<dyn FileDescriptor>));
    let write = alloc::sync::Arc::new(core::cell::RefCell::new(Box::new(WritePipeDescriptor { buffer: wrapped_buffer.clone(), read_end: None }) as Box<dyn FileDescriptor>));

    read.borrow_mut().set_end(&write);
    write.borrow_mut().set_end(&read);

    (read, write)
}