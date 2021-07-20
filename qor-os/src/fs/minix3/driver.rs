use crate::*;

use crate::fs::fstrait::*;
use crate::fs::structures::*;

use super::structures::*;

use alloc::vec;

/// Minix3 Filesystem Driver
pub struct Minix3Filesystem
{
    block_driver: crate::drivers::block::BlockDeviceDriver,
    mount_id: Option<usize>,
    vfs: Option<&'static mut crate::fs::vfs::FilesystemInterface>,
    superblock: Option<Minix3SuperBlock>,
    cache: Vec<(usize, [u8; 1024])>
}

impl Minix3Filesystem
{
    /// Initialize a new Minix3 Filesystem Interface
    pub fn new(driver_id: usize) -> Self
    {
        Self
        {
            block_driver: crate::drivers::block::get_driver_by_index(driver_id),
            mount_id: None,
            vfs: None,
            superblock: None,
            cache: Vec::new()
        }
    }

    /// Read a block as a buffer
    fn read_block_to_buffer(&mut self, index: usize) -> [u8; 1024]
    {
        
        for (idx, data) in &self.cache
        {
            if index == *idx
            {
                return *data;
            }
        }

        let mut buffer = Box::new([0; 1024]);

        let ptr = &mut *buffer as *mut [u8; 1024] as *mut u8;

        self.block_driver.sync_read(ptr, 1024, index as u64 * 1024);


        self.cache.push((index, *buffer));


        *buffer
    }

    /// Read an inode
    fn get_inode(&mut self, inode_number: usize) -> FilesystemResult<Minix3Inode>
    {
        kdebugln!(Filesystem, "Opening inode {} on fs {:?}", inode_number, self.mount_id);

        if let Some(superblock) = self.superblock
        {
            // Conver the inode number to a block index
            let block_index = (inode_number - 1) / 16 + 2 + superblock.imap_blocks as usize + superblock.zmap_blocks as usize;

            // Read the block into a buffer
            let mut buffer = self.read_block_to_buffer(block_index);

            // Read the inode out of the buffer
            let inode = unsafe { (&mut buffer as *mut [u8; 1024] as *mut Minix3Inode).add((inode_number - 1) % 16).read() };

            // The buffer is freed implicitly after the return
            Ok(inode)
        }
        else
        {
            Err(FilesystemError::FilesystemUninitialized)
        }
    }

    /// Read from a possibly nested zone
    fn read_zone(&mut self, zone: usize, level: usize, buffer: *mut u8, index: &mut usize, remaining: &mut usize, offset: &mut usize)
    {
        // If no bytes are left to be read, terminate
        if *remaining == 0
        {
            return;
        }

        if level == 0
        {
            // Read the block to a buffer
            kdebugln!(Filesystem, "Reading zone {}, lvl {}", zone, level);
            let data = self.read_block_to_buffer(zone);

            // Read byte by byte
            for v in data.iter()
            {
                if *offset > 0
                {
                    *offset -= 1;
                    continue;
                }
                
                unsafe { buffer.add(*index).write(*v) };

                *index += 1;
                *remaining -= 1;

                if *remaining == 0
                {
                    break;
                }
            }
        }
        else
        {
            // Read the block to a buffer
            kdebugln!(Filesystem, "Reading zone {}, lvl {}", zone, level);
            let data = unsafe { core::mem::transmute::<[u8; 1024], [u32; 256]>(self.read_block_to_buffer(zone)) };

            // Read byte by byte
            for v in data.iter()
            {
                // Skip entries which contain zero
                if *v == 0
                {
                    continue;
                }

                // Otherwise, use it as the zone to go to the next level down
                self.read_zone(*v as usize, level - 1, buffer, index, remaining, offset);

                // If we are done reading the file, break
                if *remaining == 0
                {
                    break;
                }
            }
        }
    }

    /// Read the data from an inode
    fn read_from_inode(&mut self, inode: Minix3Inode) -> Vec<u8>
    {
        let mut remaining = inode.size as usize;
        let mut buffer = vec![0u8; remaining];
        let mut index = 0;
        let mut offset = 0;

        for (i, zone) in inode.zones.iter().enumerate()
        {
            if *zone == 0 {continue; }
            self.read_zone(*zone as usize, i.max(6) - 6, buffer.as_mut_ptr(), &mut index, &mut remaining, &mut offset);
        }

        buffer
    }
}

impl Filesystem for Minix3Filesystem
{
    /// Initialize the filesystem on the current disk
    fn init(&mut self) -> FilesystemResult<()>
    {
        kdebugln!(Filesystem, "Initializing Minix3 Filesystem");

        // Read the super block
        let mut ptr = Box::new([0u8; 512]);

        self.block_driver.sync_read(ptr.as_mut() as *mut [u8; 512] as *mut u8, 512, 1024);

        let superblock = unsafe { *(ptr.as_mut() as *mut [u8; 512] as *mut Minix3SuperBlock) };

        // Verify the filesystem is a minix3 filesystem
        if superblock.magic != 0x4d5a
        {
            return Err(FilesystemError::BadFilesystemFormat)
        }

        self.superblock = Some(superblock);

        Ok(())
    }

    /// Sync the filesystem with the current disk
    fn sync(&mut self) -> FilesystemResult<()>
    {
        todo!()
    }

    /// Set the mount_id of the filesystem
    fn set_mount_id(&mut self, mount_id: usize, vfs: &'static mut crate::fs::vfs::FilesystemInterface)
    {
        self.mount_id = Some(mount_id);
        self.vfs = Some(vfs);
    }

    /// Get the index of the root directory of the filesystem
    fn get_root_index(&mut self) -> FilesystemResult<FilesystemIndex>
    {
        if let Some(mount_id) = self.mount_id
        {
            Ok(
                FilesystemIndex
                {
                    mount_id,
                    inode: 1,
                }
            )
        }
        else
        {
            Err(FilesystemError::FilesystemNotMounted)
        }
    }

    /// Convert a path to an inode
    fn path_to_inode(&mut self, path: &str) -> FilesystemResult<FilesystemIndex>
    {
        if let Some(vfs) = &mut self.vfs
        {
            vfs.path_to_inode(path)
        }
        else
        {
            Err(FilesystemError::FilesystemNotMounted)
        }
    }

    /// Convert an inode to a path
    fn inode_to_path(&mut self, inode: FilesystemIndex) -> FilesystemResult<&str>
    {
        if let Some(vfs) = &mut self.vfs
        {
            vfs.inode_to_path(inode)
        }
        else
        {
            Err(FilesystemError::FilesystemNotMounted)
        }
    }

    /// Get the directory entries for the given inode
    fn get_dir_entries(&mut self, inode: FilesystemIndex) -> FilesystemResult<alloc::vec::Vec<DirectoryEntry>>
    {
        if Some(inode.mount_id) == self.mount_id
        {
            let inode_data = self.get_inode(inode.inode)?;

            if inode_data.mode & 0x4000 == 0
            {
                return Err(FilesystemError::INodeIsNotADirectory);
            }

            let data = self.read_from_inode(inode_data);

            let dir_entries = unsafe { core::mem::transmute::<&[u8], &[Minix3DirEntry]>(data.as_slice()) };

            let mut result = Vec::new();

            for i in 0..inode_data.size as usize / 64
            {
                let entry = &dir_entries[i];
                let mut name = String::new();

                for c in &entry.name
                {
                    if *c == 0
                    {
                        break;
                    }

                    name.push(*c as char);
                }

                result.push(DirectoryEntry{ index: FilesystemIndex{ mount_id: inode.mount_id, inode: entry.inode as usize }, name: name, entry_type: DirectoryEntryType::Unknown });
            }

            Ok(result)
        }
        else
        {
            if let Some(vfs) = &mut self.vfs
            {
                vfs.get_dir_entries(inode)
            }
            else
            {
                Err(FilesystemError::FilesystemNotMounted)
            }
        }
    }

    /// Create a file in the directory at the given inode
    fn create_file(&mut self, _inode: FilesystemIndex, _name: alloc::string::String) -> FilesystemResult<FilesystemIndex>
    {
        todo!()
    }

    /// Create a directory in the directory at the given inode
    fn create_directory(&mut self, _inode: FilesystemIndex, _name: alloc::string::String) -> FilesystemResult<FilesystemIndex>
    {
        todo!()
    }

    /// Remove an inode at the given index from the given directory
    fn remove_inode(&mut self, _inode: FilesystemIndex, _directory: FilesystemIndex) -> FilesystemResult<()>
    {
        todo!()
    }

    /// Read the data stored in an inode
    fn read_inode(&mut self, inode: FilesystemIndex) -> FilesystemResult<Vec<u8>>
    {
        if Some(inode.mount_id) == self.mount_id
        {
            let inode = self.get_inode(inode.inode)?;
            Ok(self.read_from_inode(inode))
        }
        else
        {
            if let Some(vfs) = &mut self.vfs
            {
                vfs.read_inode(inode)
            }
            else
            {
                Err(FilesystemError::FilesystemNotMounted)
            }
        }
        
        
    }
}

