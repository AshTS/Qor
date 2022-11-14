use super::*;
use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::sync::Arc;
use core::future::Future;
use core::marker::PhantomData;
use libutils::sync::Mutex;

pub struct BlockDeviceBuffer<
    const BLOCK_SIZE: usize,
    R: Future<Output = ()> + Send,
    W: Future<Output = ()> + Send,
    BlkDev: BlockDevice<BLOCK_SIZE, R, W>,
> {
    cache: BTreeMap<usize, Box<[u8; BLOCK_SIZE]>>,
    dirty: BTreeMap<usize, ()>,
    blk_dev: Arc<Mutex<BlkDev>>,
    _0: PhantomData<R>,
    _1: PhantomData<W>,
}

impl<
        const BLOCK_SIZE: usize,
        R: Future<Output = ()> + Send,
        W: Future<Output = ()> + Send,
        BlkDev: BlockDevice<BLOCK_SIZE, R, W>,
    > BlockDeviceBuffer<BLOCK_SIZE, R, W, BlkDev>
{
    /// Construct a new Buffered block device from a block device object
    pub fn new(blk_dev: Arc<Mutex<BlkDev>>) -> Self {
        Self {
            cache: BTreeMap::new(),
            dirty: BTreeMap::new(),
            blk_dev,
            _0: PhantomData {},
            _1: PhantomData {},
        }
    }

    /// Read a block from the device
    pub async fn read_block(&mut self, block: usize) -> &[u8; BLOCK_SIZE] {
        if !self.cache.contains_key(&block) {
            let mut dev = self.blk_dev.async_lock().await;
            let mut buffer = Box::new([0u8; BLOCK_SIZE]);
            let ptr = buffer.as_mut_ptr() as usize;
            self.cache.insert(block, buffer);

            unsafe { dev.async_read(ptr, BLOCK_SIZE as u32, block as u64 * BLOCK_SIZE as u64) }
                .unwrap()
                .await;
        }

        if let Some(buffer) = self.cache.get(&block) {
            &*buffer
        } else {
            unreachable!()
        }
    }

    /// Read a block from the device and get a mutable referene to its contents
    pub async fn read_block_mut(&mut self, block: usize) -> &mut [u8; BLOCK_SIZE] {
        self.dirty.insert(block, ());

        if !self.cache.contains_key(&block) {
            let mut buffer = Box::new([0u8; BLOCK_SIZE]);
            let ptr = buffer.as_mut_ptr() as usize;

            unsafe {
                self.blk_dev.async_lock().await.async_read(
                    ptr,
                    BLOCK_SIZE as u32,
                    block as u64 * BLOCK_SIZE as u64,
                )
            }
            .unwrap()
            .await;

            self.cache.insert(block, buffer);
        }

        if let Some(buffer) = self.cache.get_mut(&block) {
            &mut *buffer
        } else {
            unreachable!()
        }
    }

    /// Write a block from the device
    pub async fn write_block(&mut self, block: usize, data: &[u8; BLOCK_SIZE]) {
        self.cache.insert(block, Box::new(*data));
        self.dirty.insert(block, ());
    }

    /// Flush the changes made to the device
    pub async fn sync_device(&mut self) {
        let mut futures = alloc::vec::Vec::new();

        for v in self.dirty.keys() {
            futures.push(
                unsafe {
                    self.blk_dev.async_lock().await.async_write(
                        self.cache.get_mut(v).unwrap().as_mut_ptr() as usize,
                        BLOCK_SIZE as u32,
                        *v as u64 * BLOCK_SIZE as u64,
                    )
                }
                .unwrap(),
            )
        }

        for future in futures {
            future.await;
        }

        self.dirty.clear();
    }
}
