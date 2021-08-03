use crate::*;
use super::descriptor::*;

use crate::fs::structures::FilesystemIndex;

/// Write side of a pipe
pub struct WritePipeDescriptor
{
    buffer: alloc::sync::Arc<core::cell::RefCell<utils::ByteRingBuffer>>
}

impl FileDescriptor for WritePipeDescriptor
{
    fn close(&mut self, _fs: &mut fs::vfs::FilesystemInterface)
    {
        // Nothing needs to be done but drop this side, which will occur elsewhere
    }

    fn write(&mut self, _fs: &mut fs::vfs::FilesystemInterface, buffer: *mut u8, count: usize) -> usize
    {
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
}

/// Read side of a pipe
pub struct ReadPipeDescriptor
{
    buffer: alloc::sync::Arc<core::cell::RefCell<utils::ByteRingBuffer>>
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
}

/// Create a new pipe pair
pub fn new_pipe() -> (ReadPipeDescriptor, WritePipeDescriptor)
{
    let buffer = utils::ByteRingBuffer::new();
    let wrapped_buffer = 
        alloc::sync::Arc::new(
            core::cell::RefCell::new(
                buffer));

    (
        ReadPipeDescriptor { buffer: wrapped_buffer.clone() },
        WritePipeDescriptor { buffer: wrapped_buffer.clone() }
    )
}