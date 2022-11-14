pub mod structures;
pub use structures::*;

use crate::types::TimeRepr;
use crate::*;
use crate::{
    drivers::{BlockDevice, BlockDeviceBuffer},
    types::DeviceIdentifier,
};
use alloc::boxed::Box;
use core::future::Future;

use super::{
    DirectoryEntry, FileMode, FileStat, FileSystem, FileSystemError, FilesystemInterface,
    FilesystemResult, InodePointer, DirectoryEntryType,
};

pub struct Minix3Filesystem<
    ReadFuture: Future<Output = ()> + Send,
    WriteFuture: Future<Output = ()> + Send,
    BlkDev: BlockDevice<1024, ReadFuture, WriteFuture> + Send,
> {
    blockdevice: BlockDeviceBuffer<1024, ReadFuture, WriteFuture, BlkDev>,
    superblock: Option<Minix3Superblock>,
    mount_id: Option<DeviceIdentifier>,
    mounted_inodes: alloc::vec::Vec<(InodePointer, InodePointer, alloc::string::String)>,
}

impl<
        ReadFuture: Future<Output = ()> + Send,
        WriteFuture: Future<Output = ()> + Send,
        BlkDev: BlockDevice<1024, ReadFuture, WriteFuture> + Send,
    > Minix3Filesystem<ReadFuture, WriteFuture, BlkDev>
{
    /// Construct a new Minix3 Filesystem driver around a buffered block device
    pub fn new(blockdevice: BlockDeviceBuffer<1024, ReadFuture, WriteFuture, BlkDev>) -> Self {
        Self {
            blockdevice,
            superblock: None,
            mount_id: None,
            mounted_inodes: alloc::vec::Vec::new(),
        }
    }

    /// Construct an inode within the current device
    pub fn inode(&self, inode: usize) -> FilesystemResult<InodePointer> {
        if let Some(mount_id) = self.mount_id {
            Ok(InodePointer {
                device_id: mount_id,
                index: inode,
            })
        } else {
            Err(fs::FileSystemError::UnmountedDevice)
        }
    }

    /// Get a reference to the super block or return an error that the filesystem was not initialized
    pub fn superblock_ref(&self) -> FilesystemResult<Minix3Superblock> {
        self.superblock
            .ok_or(FileSystemError::UninitializedFilesystem)
    }

    /// Get the inode at the given index, returning errors if it is not stored on this device
    pub async fn inode_at(&mut self, inode: InodePointer) -> FilesystemResult<Minix3Inode> {
        if self.mount_id == Some(inode.device_id) {
            let sb = self.superblock_ref()?;

            let first_inode_block = 2 + sb.imap_blocks as usize + sb.zmap_blocks as usize;
            let inode_block_index = (inode.index - 1) / (1024 / 64);
            let inode_offset_index = (inode.index - 1) % (1024 / 64);

            let block = self
                .blockdevice
                .read_block(first_inode_block + inode_block_index)
                .await;

            // Safety: Becaue the inode structure is composed of numbers, without illegal states, we can safely transmute
            let inodes = unsafe { core::mem::transmute::<&[u8; 1024], &[Minix3Inode; 16]>(block) };

            Ok(inodes[inode_offset_index])
        } else {
            Err(fs::FileSystemError::UnmountedDevice)
        }
    }

    /// Get a mutable reference to the inode at the given index, returning errors if it is not stored on this device
    pub async fn mut_inode_at(
        &mut self,
        inode: InodePointer,
    ) -> FilesystemResult<&mut Minix3Inode> {
        if self.mount_id == Some(inode.device_id) {
            let sb = self.superblock_ref()?;

            let first_inode_block = 2 + sb.imap_blocks as usize + sb.zmap_blocks as usize;
            let inode_block_index = (inode.index - 1) / (1024 / 64);
            let inode_offset_index = (inode.index - 1) % (1024 / 64);

            let block = self
                .blockdevice
                .read_block_mut(first_inode_block + inode_block_index)
                .await;

            // Safety: Becaue the inode structure is composed of numbers, without illegal states, we can safely transmute
            let inodes =
                unsafe { core::mem::transmute::<&mut [u8; 1024], &mut [Minix3Inode; 16]>(block) };

            Ok(&mut inodes[inode_offset_index])
        } else {
            Err(fs::FileSystemError::UnmountedDevice)
        }
    }

    /// Get the contents of a file from a Minix3 Inode
    async fn file_contents(&mut self, inode: Minix3Inode) -> FilesystemResult<alloc::vec::Vec<u8>> {
        let mut buffer = alloc::vec::Vec::new();
        let mut remaining = inode.size as usize;

        for (i, zone) in inode.zones.iter().enumerate() {
            if i < 7 {
                self.read_block_recursive(0, *zone as usize, &mut buffer, &mut remaining)
                    .await?;
            } else if i == 7 {
                self.read_block_recursive(1, *zone as usize, &mut buffer, &mut remaining)
                    .await?;
            } else if i == 8 {
                self.read_block_recursive(2, *zone as usize, &mut buffer, &mut remaining)
                    .await?;
            } else if i == 9 {
                self.read_block_recursive(3, *zone as usize, &mut buffer, &mut remaining)
                    .await?;
            }
        }

        Ok(buffer)
    }

    /// Recursively read block data
    #[async_recursion::async_recursion]
    async fn read_block_recursive(
        &mut self,
        level: usize,
        block_index: usize,
        buffer: &mut alloc::vec::Vec<u8>,
        remaining: &mut usize,
    ) -> FilesystemResult<()> {
        if level == 0 {
            self.read_block(block_index, buffer, remaining).await
        } else {
            // Read in the data from the block as u32's
            let block = self.blockdevice.read_block(block_index).await;

            // Safety: Becaue the inode structure is composed of numbers, without illegal states, we can safely transmute
            let indexes = unsafe { core::mem::transmute::<&[u8; 1024], &[u32; 256]>(block) };

            // Loop over the indexes, reading from the nonzero ones
            for index in indexes {
                self.read_block_recursive(level - 1, *index as usize, buffer, remaining)
                    .await?;

                if *remaining == 0 {
                    break;
                }
            }

            Ok(())
        }
    }

    /// Read a single block into a vector buffer
    async fn read_block(
        &mut self,
        block_index: usize,
        buffer: &mut alloc::vec::Vec<u8>,
        remaining: &mut usize,
    ) -> FilesystemResult<()> {
        let block_data = self.blockdevice.read_block(block_index).await;

        let len_to_copy = (*remaining).min(block_data.len());

        buffer.extend_from_slice(&block_data[0..len_to_copy]);
        *remaining -= len_to_copy;

        Ok(())
    }

    /// Get the directory entries in minix3 format for the given inode
    async fn minix3_dir_ents(&mut self, inode: InodePointer) -> FilesystemResult<alloc::vec::Vec<Minix3DirEntry>> {
        let inode = self.inode_at(inode).await?;
        let data = self.file_contents(inode).await?;

        let directory_entries =
            unsafe { core::mem::transmute::<&[u8], &[Minix3DirEntry]>(data.as_slice()) };

        Ok((&directory_entries[0..data.len()/64])
            .iter().cloned()
            .collect())
    }
}

#[async_trait::async_trait]
impl<
        ReadFuture: Future<Output = ()> + Send,
        WriteFuture: Future<Output = ()> + Send,
        BlkDev: BlockDevice<1024, ReadFuture, WriteFuture> + Send,
    > FileSystem for Minix3Filesystem<ReadFuture, WriteFuture, BlkDev>
{
    /// Initialize the filesystem on the current disk
    async fn init(&mut self) -> FilesystemResult<()> {
        // Read in the superblock
        let raw_block_device = self.blockdevice.read_block(1).await;

        // Safety: The superblock structure is just numbers, so we can safely transmute the memory
        let superblock =
            unsafe { core::mem::transmute::<&[u8; 1024], &Minix3Superblock>(raw_block_device) };

        // Make sure the magic is correct
        if superblock.magic != 0x4d5a {
            return Err(FileSystemError::BadFilesystemFormat);
        }

        // Finaly, insert the superblock
        self.superblock = Some(*superblock);

        kdebugln!(unsafe "{:#?}", self.superblock);

        Ok(())
    }

    /// Sync the filesystem on the current disk
    async fn sync(&mut self) -> FilesystemResult<()> {
        self.blockdevice.sync_device().await;
        Ok(())
    }

    /// Set the mount_if of the filesystem
    async fn set_mount_id(
        &mut self,
        mount_id: DeviceIdentifier,
        _: &mut FilesystemInterface,
    ) -> FilesystemResult<()> {
        self.mount_id = Some(mount_id);
        Ok(())
    }

    /// Get the root inode of the filesystem
    async fn get_root_inode(&mut self) -> FilesystemResult<InodePointer> {
        self.inode(1)
    }

    /// Stat the given inode
    async fn stat_inode(&mut self, inode_ptr: InodePointer) -> FilesystemResult<FileStat> {
        let inode = self.inode_at(inode_ptr).await?;

        let stat = FileStat {
            index: inode_ptr,
            mode: FileMode::from_bits(inode.mode),
            links: inode.nlinks,
            uid: inode.uid,
            gid: inode.gid,
            special_dev_id: 0,
            size: inode.size as usize,
            blk_size: 1024,
            blocks_allocated: 0,
            atime: TimeRepr(inode.atime as usize),
            mtime: TimeRepr(inode.mtime as usize),
            ctime: TimeRepr(inode.ctime as usize),
        };

        Ok(stat)
    }

    /// Get the directory entries from the given inode
    async fn dir_entries(
        &mut self,
        inode_ptr: InodePointer,
    ) -> FilesystemResult<alloc::vec::Vec<DirectoryEntry>> {
        let inode = self.inode_at(inode_ptr).await?;

        if FileMode::from_bits(inode.mode).entry_type() != DirectoryEntryType::Directory {
            return Err(FileSystemError::InodeIsADirectory(inode_ptr));
        }

        let orig_dir_ents = self.minix3_dir_ents(inode_ptr).await?;

        let mut result = alloc::vec::Vec::new();

        for dir_ent in orig_dir_ents {
            let this_inode_ptr = self.inode(dir_ent.inode as usize)?;
            let this_inode = self.inode_at(this_inode_ptr).await?;

            result.push(DirectoryEntry {
                index: this_inode_ptr,
                name: dir_ent.to_string(),
                entry_type: FileMode::from_bits(this_inode.mode).entry_type()
            })
        }

        Ok(result)
    }

    /// Mount a filesystem at the given inode
    async fn mount_fs_at(
        &mut self,
        inode: InodePointer,
        root: InodePointer,
        name: alloc::string::String,
    ) -> FilesystemResult<()> {
        self.mounted_inodes.push((inode, root, name));

        Ok(())
    }

    /// Allocate a new file with the given mode
    async fn create_file(
        &mut self,
        _inode: InodePointer,
        _mode: FileMode,
        _name: alloc::string::String,
    ) -> FilesystemResult<InodePointer> {
        todo!()
    }

    /// Allocate a new directory
    async fn create_directory(
        &mut self,
        _inode: InodePointer,
        _name: alloc::string::String,
    ) -> FilesystemResult<InodePointer> {
        todo!()
    }

    /// Remove an inode
    async fn remove_inode(&mut self, _inode: InodePointer) -> FilesystemResult<()> {
        todo!()
    }

    /// Increment the number of hard links to an inode
    async fn increment_links(&mut self, inode: InodePointer) -> FilesystemResult<usize> {
        self.mut_inode_at(inode).await?.nlinks += 1;
        Ok(self.inode_at(inode).await?.nlinks.into())
    }

    /// Decrement the number of hard links to an inode
    async fn decrement_links(&mut self, inode: InodePointer) -> FilesystemResult<usize> {
        self.mut_inode_at(inode).await?.nlinks -= 1;
        Ok(self.inode_at(inode).await?.nlinks.into())
    }

    /// Read the data from an inode
    async fn read_inode(&mut self, inode: InodePointer) -> FilesystemResult<alloc::vec::Vec<u8>> {
        let inode = self.inode_at(inode).await?;
        self.file_contents(inode).await
    }

    /// Write data to an inode
    async fn write_inode(&mut self, _inode: InodePointer, _data: &[u8]) -> FilesystemResult<()> {
        todo!()
    }
}
