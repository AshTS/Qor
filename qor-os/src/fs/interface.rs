use crate::*;

use super::structures::*;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;

/// Filesystem Error
#[derive(Debug, Clone)]
pub enum FilesystemError
{
    NotMinix3,
    FileNotFound(String),
}

/// Minix3 Filesystem Interface
pub struct FilesystemInterface
{
    block_driver: drivers::block::BlockDeviceDriver,
    super_block: Option<SuperBlock>,
    tree: BTreeMap<String, usize>
}

impl FilesystemInterface
{
    /// Create a new Filesystem Interface
    pub fn new(block_device_driver: usize) -> Self
    {
        Self
        {
            block_driver: drivers::block::get_driver_by_index(block_device_driver),
            super_block: None,
            tree: BTreeMap::new()
        }
    }

    /// Get a reference to the superblock
    fn superblock_ref(&mut self) -> &SuperBlock
    {
        if self.super_block.is_some()
        {
            self.super_block.as_ref().unwrap()
        }
        else
        {
            let mut ptr = Box::new([0u8; 512]);

            self.block_driver.sync_read(ptr.as_mut() as *mut [u8; 512] as *mut u8, 512, 1024);

            self.super_block = Some(unsafe { *(ptr.as_mut() as *mut [u8; 512] as *mut SuperBlock) });

            self.superblock_ref()
        }
    }

    /// Initialize the interface
    pub fn initialize(&mut self) -> Result<(), FilesystemError>
    {
        kdebugln!(Filesystem, "Initializing the Minix3 Filesystem");

        // Get the superblock
        let superblock = self.superblock_ref();

        // Verify the magic
        if superblock.magic != 0x4d5a
        {
            return Err(FilesystemError::NotMinix3)
        }

        // Mount the file system
        self.mount(1, "/");

        Ok(())
    }

    /// Read a block as a buffer
    fn read_block_to_buffer(&self, index: usize) -> Box<[u8; 1024]>
    {
        let mut buffer = Box::new([0; 1024]);

        self.block_driver.sync_read(buffer.as_mut() as *mut [u8; 1024] as *mut u8, 1024, index as u64 * 1024);

        buffer
    }

    /// Read an inode
    fn get_inode(&mut self, inode_number: usize) -> Inode
    {
        kdebugln!(Filesystem, "Opening inode {}", inode_number);
        // Get the superblock
        let superblock = self.superblock_ref();

        // Conver the inode number to a block index
        let block_index = (inode_number - 1) / 16 + 2 + superblock.imap_blocks as usize + superblock.zmap_blocks as usize;

        // Read the block into a buffer
        let mut buffer = self.read_block_to_buffer(block_index);

        // Read the inode out of the buffer
        let inode = unsafe { (buffer.as_mut() as *mut [u8; 1024] as *mut Inode).add((inode_number - 1) % 16).read() };

        // The buffer is freed implicitly after the return
        inode
    }

    /// Read from a possibly nested zone
    fn read_zone(&mut self, zone: usize, level: usize, buffer: *mut u8, index: &mut usize, remaining: &mut usize)
    {
        // If no bytes are left to be read, terminate
        if *remaining == 0
        {
            return;
        }

        if level == 0
        {
            // Read the block to a buffer
            let data = self.read_block_to_buffer(zone);

            // Read byte by byte
            for v in data.iter()
            {
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
            let data = unsafe { core::mem::transmute::<Box<[u8; 1024]>, Box<[u32; 256]>>(self.read_block_to_buffer(zone)) };

            // Read byte by byte
            for v in data.iter()
            {
                // Skip entries which contain zero
                if *v == 0
                {
                    continue;
                }

                // Otherwise, use it as the zone to go to the next level down
                self.read_zone(*v as usize, level - 1, buffer, index, remaining);

                // If we are done reading the file, break
                if *remaining == 0
                {
                    break;
                }
            }
        }
    }

    /// Read the data from an inode
    fn read_inode(&mut self, inode: Inode, buffer: *mut u8, size: usize) -> usize
    {
        let mut remaining = size;
        let mut index = 0;

        for (i, zone) in inode.zones.iter().enumerate()
        {
            if *zone == 0 {continue; }
            self.read_zone(*zone as usize, i.max(6) - 6, buffer, &mut index, &mut remaining);
        }

        index
    }

    /// Get the directory entries for the given inode number
    fn get_dir_entries(&mut self, inode: usize) -> Vec<DirEntry>
    {
        let inode = self.get_inode(inode);

        let entries = vec![DirEntry{inode: 0, name: [0u8; 60]}; inode.size as usize / 64];

        self.read_inode(inode, entries.as_ptr() as *mut DirEntry as *mut u8, inode.size as usize);
        entries
    }

    /// Mount the given inode at the given path
    fn mount(&mut self, inode: usize, path: &str)
    {
        self.tree.insert(String::from(path), inode);

        for dir_entry in self.get_dir_entries(inode)
        {
            // Convert the name to a string
            let mut name = String::from(path);
            for v in &dir_entry.name
            {
                if *v == 0
                {
                    break;
                }

                name.push(*v as char);
            }

            // Do not recurse down '.' and '..'
            if name.ends_with("/.") || name.ends_with("/..")
            {
                // Insert the entry
                self.tree.insert(name.clone(), dir_entry.inode as usize);
                continue;
            }

            if dir_entry.inode == 0
            {
                continue;
            }

            // Otherwise check the inode to see if this entry is a directory
            let inode = self.get_inode(dir_entry.inode as usize);

            if inode.mode & 16384 != 0
            {
                name.push('/');
                self.mount(dir_entry.inode as usize, &name);
            }
            else
            {
                // Insert the entry
                self.tree.insert(name.clone(), dir_entry.inode as usize);
            }
        }
    }

    /// Read a file to a buffer
    pub fn read_to_buffer(&mut self, path: &str) -> Result<Vec<u8>, FilesystemError>
    {
        // Attempt to get the inode from the path
        let inode_number = if let Some(inode_number) = self.tree.get(path)
        {
            *inode_number
        }
        else
        {
            return Err(FilesystemError::FileNotFound(String::from(path)));
        };

        let inode = self.get_inode(inode_number);

        let data = vec![0u8; inode.size as usize];

        self.read_inode(inode, data.as_slice().as_ptr() as *mut u8, data.len());

        Ok(data)
    }

    /// Test the file system (this is just for development)
    pub fn test(&mut self)
    {
        for (key, _) in self.tree.iter()
        {
            if !key.ends_with("/.") && !key.ends_with("/..")
            {
                kprintln!("{}", key);
            }
        }

        let data = self.read_to_buffer("/root/test.txt").unwrap();

        for v in &data[0..data.len()]
        {
            kprint!("{}", *v as char);
        }

        kprintln!();
    }
}