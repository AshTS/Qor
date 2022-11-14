use core::future::Future;

/// Generic Block Device Driver Trait
pub trait BlockDevice<
    const BLOCK_SIZE: usize,
    ReadResult: Future<Output = ()>,
    WriteResult: Future<Output = ()>,
>
{
    /// Asynchronously read from the block device at the given offset
    unsafe fn async_read(&mut self, buffer: usize, size: u32, offset: u64) -> Option<ReadResult>;

    /// Asynchronously write to the block device at the given offset
    unsafe fn async_write(&mut self, buffer: usize, size: u32, offset: u64) -> Option<WriteResult>;
}
