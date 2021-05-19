use core::{u8, usize};

use crate::*;

use alloc::{collections::BTreeMap, format};

pub static mut INTERFACE: Option<FileSystemInterface> = None;

#[repr(C)]
pub struct DirEntry
{
   pub inode: u32,
   pub name:  [u8; 60]
}

impl DirEntry
{
    pub fn new() -> Self
    {
        Self
        {
            inode: 0,
            name: [0; 60]
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SuperBlock
{
  pub ninodes:         u32,
  pub pad0:            u16,
  pub imap_blocks:     u16,
  pub zmap_blocks:     u16,
  pub first_data_zone: u16,
  pub log_zone_size:   u16,
  pub pad1:            u16,
  pub max_size:        u32,
  pub zones:           u32,
  pub magic:           u16,
  pub pad2:            u16,
  pub block_size:      u16,
  pub disk_version:    u8,
}

impl SuperBlock
{
    pub fn new() -> Self
    {
        Self
        {
            ninodes: 0,         
            pad0: 0,            
            imap_blocks: 0,     
            zmap_blocks: 0,     
            first_data_zone: 0, 
            log_zone_size: 0,   
            pad1: 0,            
            max_size: 0,        
            zones: 0,           
            magic: 0,           
            pad2: 0,            
            block_size: 0,      
            disk_version: 0
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct Inode
{
	pub mode:   u16,
	pub nlinks: u16,
	pub uid:    u16,
	pub gid:    u16,
	pub size:   u32,
	pub atime:  u32,
	pub mtime:  u32,
	pub ctime:  u32,
	pub zones:  [u32; 10]
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StatData
{
    pub mode:   u16,
	pub nlinks: u16,
	pub uid:    u16,
	pub gid:    u16,
	pub size:   u32,
	pub atime:  u32,
	pub mtime:  u32,
	pub ctime:  u32,
}

#[derive(Debug)]
pub struct FileSystemError
{
    pub msg: String
}

/// Wraps an inode reference
#[derive(Debug)]
pub struct FilePtr(usize);

#[derive(Debug)]
pub struct FileSystemInterface
{
    block_device_driver: crate::drivers::block::BlockDeviceDriver,
    super_block: Option<SuperBlock>,
    last_first_free_inode: usize,
    last_first_free_zone: usize,
    tree: Option<BTreeMap<String, usize>>
}

impl FileSystemInterface
{
    /// Creates a new file system interface on the given block device
    pub fn new(block_device_index: usize) -> Self
    {
        Self
        {
            block_device_driver: crate::drivers::block::get_driver_by_index(block_device_index),
            super_block: None,
            last_first_free_inode: 0,
            last_first_free_zone: 0,
            tree: None
        }
    }

    /// Get the inode for a file
    pub fn get_inode_by_name(&mut self, name: &str) -> Result<FilePtr, FileSystemError>
    {
        if let Some(tree) = &self.tree
        {
            Ok(FilePtr(*tree.get(name).ok_or(FileSystemError{ msg: format!("Unable to open file `{}`", name) })?))
        }
        else
        {
            self.tree = Some(self.map(1, "/"));
            self.get_inode_by_name(name)
        }
    }

    /// Get the buffer for a file by name
    pub fn read_file(&mut self, name: &str, buffer: &mut [u8], size: usize) -> Result<usize, FileSystemError>
    {
        let inode_number = self.get_inode_by_name(name)?;

        let inode = self.get_inode(inode_number);

        Ok(self.read_inode(inode, buffer, size))
    }

    /// Get the stats for a file by name
    pub fn stat_file(&mut self, name: &str) -> Result<StatData, FileSystemError>
    {
        let inode_number = self.get_inode_by_name(name)?;

        let inode = self.get_inode(inode_number);

        Ok(
            StatData
            {
                mode: inode.mode,
                nlinks: inode.nlinks,
                uid: inode.uid,
                gid: inode.gid,
                size: inode.size,
                atime: inode.atime,
                mtime: inode.mtime,
                ctime: inode.ctime,
            }
        )
    }

    /// Get a block as a buffer
    pub fn get_block_as_buffer(&self, block_index: usize) -> Box<[u8; 1024]>
    {
        let buffer = Box::new([0u8; 1024]);

        let ptr = Box::leak(buffer) as *mut [u8] as *mut u8;

        self.block_device_driver.sync_read(ptr, 1024, block_index as u64 * 1024);

        unsafe { Box::from_raw(ptr as *mut [u8; 1024]) }
    }

    /// Read a buffer in the given inode
    pub fn get_inode(&mut self, inode: FilePtr) -> Inode
    {
        if self.super_block.is_none()
        {
            self.update_super_block();
        }

        let index = (inode.0 - 1) / 1024;

        let buffer = self.get_block_as_buffer(2 + self.super_block.unwrap().imap_blocks as usize + self.super_block.unwrap().zmap_blocks as usize + index);

        let inode_ptr = Box::leak(buffer) as *mut [u8] as *mut Inode;
        let inode = unsafe { *inode_ptr.add((inode.0 - 1) % (1024 / 64)) };

        inode
    }

    /// Read data from a zone
    pub fn read_from_zone_direct(&mut self, zone: usize) -> Box<[u8; 1024]>
    {
        self.get_block_as_buffer(zone)
    }

    /// Read a possibly nested zone into a buffer with the given remaining space
    pub fn read_zone(&mut self, zone: usize, level: usize, buffer: &mut [u8], index: &mut usize, remaining: &mut usize)
    {
        if level == 0
        {
            let data = self.read_from_zone_direct(zone);

            for v in &*data
            {
                if *remaining == 0
                {
                    break;
                }

                buffer[*index] = *v;

                *index += 1;
                *remaining -= 1;
            }
        }
        else
        {
            let data = unsafe { Box::from_raw(Box::leak(self.read_from_zone_direct(zone)) as *mut [u8] as *mut [u32]) };

            for v in &*data
            {
                if *remaining == 0
                {
                    break;
                }

                if *v != 0
                {
                    self.read_zone(*v as usize, level - 1, buffer, index, remaining);
                }
            }
        }
    }

    pub fn get_dir_entries(&mut self, inode: FilePtr) -> Box<[DirEntry]>
    {
        let mut buffer = Box::new([0u8; 1024]);

        let inode = self.get_inode(inode);

        self.read_inode(inode, &mut *buffer, inode.size as usize);

        unsafe {Box::from_raw(Box::leak(buffer) as *mut [u8] as *mut [DirEntry]) }
    }

    /// Traverse the file system
    pub fn map(&mut self, inode: usize, path: &str) -> BTreeMap<String, usize>
    {
        let mut result = BTreeMap::new();

        let mut count = self.get_inode(FilePtr(inode)).size / 64;
        for dir_entry in &*self.get_dir_entries(FilePtr(inode))
        {
            if count == 0
            {
                break;
            }
            else
            {
                count -= 1;
            }

            if dir_entry.name[0] == '.' as u8
            {
                continue;
            }

            let mut p = String::from(path);

            for c in &dir_entry.name
            {
                if *c != 0
                {
                    p.push(*c as char);
                }
                else
                {
                    break;
                }
            }

            result.insert(p.clone(), dir_entry.inode as usize);

            p += "/";

            let inode = self.get_inode(FilePtr(dir_entry.inode as usize));

            if inode.mode & 16384 != 0
            {
                result.append(&mut self.map(dir_entry.inode as usize, &p));
            }
        }

        result
    }

    /// Read the data from an inode
    pub fn read_inode(&mut self, inode: Inode, buffer: &mut [u8], size: usize) -> usize
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

    /// Initialize the File System
    pub fn initialize(&mut self) -> Result<(), FileSystemError>
    {
        self.update_super_block();

        if !self.verify()
        {
            return Err(FileSystemError{msg: format!("The filesystem is not a minix3 file system")});
        }

        Ok(())
    }

    pub fn update_super_block(&mut self)
    {
        let ptr = Box::leak(Box::new(SuperBlock::new())) as *mut SuperBlock as *mut u8;

        self.block_device_driver.sync_read(ptr, 512, 1024);

        self.super_block = Some(unsafe { *(ptr as *mut SuperBlock) })
    }

    /// Verify the file system
    pub fn verify(&mut self) -> bool
    {
        if self.super_block.is_none()
        {
            self.update_super_block()
        }

        self.super_block.unwrap().magic == 0x4d5a
    }

    /// Get the first free inode bit
    pub fn first_free_inode(&mut self) -> Result<usize, FileSystemError>
    {
        if self.super_block.is_none()
        {
            self.update_super_block();
        }

        for i in self.last_first_free_inode / 8096 .. self.super_block.unwrap().imap_blocks as usize
        {
            let buffer = self.get_block_as_buffer(2 + i);

            let start = if self.last_first_free_inode < 8096
            {
                self.last_first_free_inode
            }
            else
            {
                0
            };

            for j in start..8096
            {
                if (buffer[j / 8] >> (j % 8)) & 1 == 0
                {
                    return Ok(i * 8086 + j);
                }
            }
        }

        Err(FileSystemError{msg: format!("There are no remaining free inodes")})
    }

    /// Get the first free zone bit
    pub fn first_free_zone(&mut self) -> Result<usize, FileSystemError>
    {
        if self.super_block.is_none()
        {
            self.update_super_block();
        }

        for i in self.last_first_free_zone / 8096 .. self.super_block.unwrap().zmap_blocks as usize
        {
            let buffer = self.get_block_as_buffer(2 + self.super_block.unwrap().imap_blocks as usize + i);

            let start = if self.last_first_free_zone < 8096
            {
                self.last_first_free_zone
            }
            else
            {
                0
            };

            for j in start..8096
            {
                if (buffer[j / 8] >> (j % 8)) & 1 == 0
                {
                    return Ok(i * 8086 + j);
                }
            }
        }

        Err(FileSystemError{msg: format!("There are no remaining free zones")})
    }
}