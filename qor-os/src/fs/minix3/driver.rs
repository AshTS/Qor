use crate::*;

use crate::fs::fstrait::*;
use crate::fs::structures::*;

use super::structures::*;

use alloc::vec;

use crate::process::descriptor::*;

use libutils::paths::PathBuffer;

use super::super::ioctl::*;

/// Minix3 Filesystem Driver
pub struct Minix3Filesystem
{
    block_driver: &'static mut crate::drivers::virtio::drivers::block::BlockDriver,
    mount_id: Option<usize>,
    vfs: Option<&'static mut crate::fs::vfs::FilesystemInterface>,
    superblock: Option<Minix3SuperBlock>,
    cache: Vec<(usize, [u8; 1024])>,
    rewritten: Vec<(usize, [u8; 1024])>,
    mount_inodes: Vec<(FilesystemIndex, FilesystemIndex, String)>
}

impl Minix3Filesystem
{
    /// Initialize a new Minix3 Filesystem Interface
    pub fn new(driver_id: usize) -> Self
    {
        Self
        {
            block_driver: crate::drivers::virtio::get_block_driver(driver_id).unwrap(),
            mount_id: None,
            vfs: None,
            superblock: None,
            cache: Vec::new(),
            rewritten: Vec::new(),
            mount_inodes: Vec::new(),
        }
    }

    /// Read a block as a buffer
    fn read_block_to_buffer(&mut self, index: usize) -> [u8; 1024]
    {
        for (idx, data) in &self.rewritten
        {
            if index == *idx
            {
                return *data;
            }
        }
        
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

    /// Edit the contents of a block
    fn edit_block(&mut self, index: usize, new_data: [u8; 1024]) -> FilesystemResult<()>
    {
        for (idx, data) in &mut self.rewritten
        {
            if index == *idx
            {
                *data = new_data;
                return Ok(())
            }
        }

        self.rewritten.push((index, new_data));

        Ok(())
    }

    /// Edit the contents at a specific region in the block
    fn edit_block_region(&mut self, index: usize, start: usize, new_data: &[u8]) -> FilesystemResult<usize>
    {
        let mut i = start;

        let mut rewritten_index = 0;

        for (idx, data) in &mut self.rewritten
        {
            if index == *idx
            {
                for v in new_data
                {
                    data[i] = *v;
                    i += 1;

                    if i == 1024 { break; }
                }

                return Ok(rewritten_index)
            }

            rewritten_index += 1;
        }

        let mut prev_data = self.read_block_to_buffer(index);

        for v in new_data
        {
            prev_data[i] = *v;
            i += 1;

            if i == 1024 { break; }
        }

        self.rewritten.push((index, prev_data));

        Ok(self.rewritten.len() - 1)
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

    /// Get a mutable buffer into editable memory
    fn get_mut_buffer(&mut self, block: usize) -> FilesystemResult<&mut [u8; 1024]>
    {
        let mut rewritten_index = 0;

        for (idx, _) in &mut self.rewritten
        {
            if block == *idx
            {
                break;
            }

            rewritten_index += 1;
        }

        if rewritten_index == self.rewritten.len()
        {
            let buffer =  self.read_block_to_buffer(block);
            self.rewritten.push((block, buffer));
        }

        Ok(&mut self.rewritten[rewritten_index].1)
    }

    /// Edit an inode
    fn get_mut_inode(&mut self, inode_number: usize) -> FilesystemResult<& mut Minix3Inode>
    {
        if let Some(superblock) = self.superblock
        {
            // Conver the inode number to a block index
            let block_index = (inode_number - 1) / 16 + 2 + superblock.imap_blocks as usize + superblock.zmap_blocks as usize;

            // Get a reference to that memory
            let buffer_ref = self.get_mut_buffer(block_index)?;

            // Get the reference to the specific inode
            let inode = unsafe { (buffer_ref as *mut [u8; 1024] as *mut Minix3Inode).add((inode_number - 1) % 16).as_mut().unwrap() };

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

    /// Add a directory entry at the given inode
    fn add_directory_entry_raw(&mut self, inode: usize, entry: Minix3DirEntry) -> FilesystemResult<()>
    {
        // Get a mutable reference to the inode
        let inode_ref = self.get_mut_inode(inode)?;

        // Get the original size
        let orig_entry_count = inode_ref.size / 64;

        // Increment the size
        inode_ref.size += 64;

        let zone_index = orig_entry_count / 16;

        if zone_index < 7
        {
            let next = if inode_ref.zones[zone_index as usize] == 0
            {
                let next = self.next_free_zone()?;
                self.claim_zone(next)?;

                Some(next as u32)
            }
            else
            {
                None
            };


            // Get a mutable reference to the inode
            let inode_ref = self.get_mut_inode(inode)?;

            if let Some(next) = next
            {
                inode_ref.zones[zone_index as usize] = next;
            }

            // Get the zone
            let zone = inode_ref.zones[zone_index as usize];

            let buffer = unsafe { core::mem::transmute::<&mut[u8; 1024], &mut[Minix3DirEntry; 16]>(self.get_mut_buffer(zone as usize)?) };

            buffer[orig_entry_count as usize % 16] = entry;
        }
        else
        {
            todo!()
        }

        Ok(())
    }

    /// Add a directory entry from the inode and name to the given inode
    fn add_directory_entry(&mut self, dest: usize, inode: usize, name: &str) -> FilesystemResult<()>
    {
        let mut ent = Minix3DirEntry
        {
            inode: inode as u32,
            name: [0; 60],
        };

        for (i, c) in name.chars().enumerate()
        {
            ent.name[i] = c as u8;
        }

        self.add_directory_entry_raw(dest, ent)
    }

    /// Get the next available free inode
    fn next_free_inode(&mut self) -> FilesystemResult<usize>
    {
        if let Some(superblock) = self.superblock
        {
            let mut i = 0;

            let num_blocks = superblock.imap_blocks;

            for b in 0..num_blocks
            {
                let buffer = self.read_block_to_buffer(2 + b as usize);

                for v in &buffer
                {
                    if *v == 0xFF
                    {
                        i += 8;
                        continue;
                    }

                    let mut walker = 0x01;

                    while walker > 0
                    {
                        if *v & walker == 0
                        {
                            return Ok(i);
                        }

                        i += 1;
                        walker <<= 1;
                    }
                }
            }

            Err(FilesystemError::OutOfSpace)
        }
        else
        {
            Err(FilesystemError::FilesystemUninitialized)
        }
    }

    /// Claim an inode
    fn claim_inode(&mut self, mut inode: usize) -> FilesystemResult<()>
    {
        inode -= 0;

        let block = 2 + inode / (8 * 1024);
        let byte = (inode / 8) % 1024;
        let bit = inode % 8;

        let buffer = self.get_mut_buffer(block)?;

        buffer[byte] |= 0x01 << bit;

        Ok(())
    }

    /// Free an inode
    fn free_inode(&mut self, mut inode: usize) -> FilesystemResult<()>
    {
        inode -= 0;

        let block = 2 + inode / (8 * 1024);
        let byte = (inode / 8) % 1024;
        let bit = inode % 8;

        let buffer = self.get_mut_buffer(block)?;

        buffer[byte] &= !(0x01 << bit);

        Ok(())
    }

    /// Get the next available free inode
    fn next_free_zone(&mut self) -> FilesystemResult<usize>
    {
        if let Some(superblock) = self.superblock
        {
            let mut i = 0;

            let num_blocks = superblock.zmap_blocks;

            for b in 0..num_blocks
            {
                let buffer = self.read_block_to_buffer(2 + b as usize + superblock.imap_blocks as usize);

                for v in &buffer
                {
                    if *v == 0xFF
                    {
                        i += 8;
                        continue;
                    }

                    let mut walker = 0x01;

                    while walker > 0
                    {
                        if *v & walker == 0 && i as u16 >= superblock.first_data_zone + 500
                        {
                            kwarnln!("Free Zone: {}", i);
                            return Ok(i);
                        }

                        i += 1;
                        walker <<= 1;
                    }
                }
            }

            Err(FilesystemError::OutOfSpace)
        }
        else
        {
            Err(FilesystemError::FilesystemUninitialized)
        }
    }

    /// Claim a zone
    fn claim_zone(&mut self, zone: usize) -> FilesystemResult<()>
    {
        if let Some(superblock) = self.superblock
        {
            let block = 2 + superblock.imap_blocks as usize + zone / (8 * 1024);
            let byte = (zone / 8) % 1024;
            let bit = zone % 8;

            let buffer = self.get_mut_buffer(block)?;

            buffer[byte] |= 0x01 << bit;

            Ok(())
        }
        else
        {
            Err(FilesystemError::FilesystemUninitialized)
        }
    }

    /// Free a zone
    fn free_zone(&mut self, zone: usize) -> FilesystemResult<()>
    {
        if let Some(superblock) = self.superblock
        {
            let block = 2 + superblock.imap_blocks as usize + zone / (8 * 1024);
            let byte = (zone / 8) % 1024;
            let bit = zone % 8;

            let buffer = self.get_mut_buffer(block)?;

            buffer[byte] &= !(0x01 << bit);

            Ok(())
        }
        else
        {
            Err(FilesystemError::FilesystemUninitialized)
        }
    }

    /// Recursive zone allocation
    fn recursive_zone_alloc(&mut self, level: usize, remaining: &mut usize) -> FilesystemResult<usize>
    {
        if *remaining == 0
        {
            return Ok(0);
        }

        let zone = self.next_free_zone()?;
        self.claim_zone(zone)?;

        if level > 0
        {
            let buffer = self.get_mut_buffer(zone)?;
            let zones = unsafe { core::mem::transmute::<&mut [u8; 1024], &mut [u32; 256]>(buffer) };

            for z in zones
            {
                *z = self.recursive_zone_alloc(level - 1, remaining)? as u32;
            }
        }
        else
        {
            *remaining -= 1;
        }

        Ok(zone)
    }

    /// Allocate zones
    fn allocate_zones(&mut self, inode: &mut Minix3Inode, mut count: usize) -> FilesystemResult<()>
    {
        // TODO: Make this acknowledge any previously allocated zones, right
        // now, it assumes zones are free which means there will be a memory
        // leak if a nonempty file is passed, therefore this check is in place:
        assert_eq!(inode.zones, [0; 10]);

        for (i, slot) in inode.zones.iter_mut().enumerate()
        {
            *slot = self.recursive_zone_alloc(i.max(6) - 6, &mut count)? as u32;

            if count == 0
            {
                break;
            }
        }

        Ok(())
    }

    /// Recursive copy to zones
    fn recursive_copy_to_zones(&mut self, zone: usize, level: usize, data: &[u8], index: &mut usize) -> FilesystemResult<()>
    {
        if *index >= data.len()
        {
            return Ok(());
        }

        if level == 0
        {
            for byte in self.get_mut_buffer(zone)?.iter_mut()
            {
                *byte = data[*index];

                *index += 1;

                if *index >= data.len()
                {
                    break;
                }
            }
        }
        else
        {
            for slot in unsafe { core::mem::transmute::<[u8; 1024], [u32; 256]>(self.read_block_to_buffer(zone)) }.iter()
            {
                self.recursive_copy_to_zones(*slot as usize, level - 1, data, index)?;

                if *index >= data.len()
                {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Copy data to zones
    fn copy_to_zones(&mut self, inode: &mut Minix3Inode, data: &[u8]) -> FilesystemResult<()>
    {
        let mut index = 0;

        if data.len() == 0
        {
            return Ok(())
        }

        for (i, zone) in inode.zones.iter().enumerate()
        {
            self.recursive_copy_to_zones(*zone as usize, i.max(6) - 6, data, &mut index)?;

            if index >= data.len()
            {
                break;
            }
        }

        Ok(())
    }

    /// Recursive Free Zones
    fn recursive_free_zones(&mut self, zone: usize, level: usize) -> FilesystemResult<()>
    {
        if level == 0
        {
            self.free_zone(zone)?;
        }
        else
        {
            let current = self.read_block_to_buffer(zone);
            let zone_numbers = unsafe { core::mem::transmute::<[u8; 1024], [u32; 256]>(current) };

            for zone in &zone_numbers
            {
                if *zone == 0
                {
                    break;
                }

                self.recursive_free_zones(*zone as usize, level - 1)?;
            }
        }

        Ok(())
    }

    /// Free zones
    fn free_zones(&mut self, inode: &mut Minix3Inode) -> FilesystemResult<()>
    {
        for (i, zone) in inode.zones.iter_mut().enumerate()
        {
            if *zone == 0
            {
                break;
            }

            self.recursive_free_zones(*zone as usize, i.max(6) - 6)?;
            *zone = 0;
        }

        Ok(())
    }

    /// Write data to a file
    fn write_to_file(&mut self, inode_number: usize, data: &[u8]) -> FilesystemResult<()>
    {
        let mut inode = self.get_inode(inode_number)?;

        // TODO: This is not as efficent as I would like it to be, it currently
        // will free and then reallocate zones
        self.free_zones(&mut inode)?;
        self.allocate_zones(&mut inode, (data.len() + 1023) / 1024)?;
        self.copy_to_zones(&mut inode, data)?;

        inode.size = data.len() as u32;

        *(self.get_mut_inode(inode_number)?) = inode;

        Ok(())
    }

    /// Allocate a file
    fn allocate_file(&mut self, data: String, mode: u16) -> FilesystemResult<usize>
    {
        let next_inode = self.next_free_inode()?;
        self.claim_inode(next_inode)?;

        let inode = self.get_mut_inode(next_inode)?;

        *inode = Minix3Inode
        {
            mode,
            nlinks: 1,
            uid: 1000,
            gid: 1000,
            size: 50,
            atime: 0,
            mtime: 0,
            ctime: 0,
            zones: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        };

        self.write_to_file(next_inode, data.as_bytes())?;

        Ok(next_inode)
    }

    /// Allocate a new directory
    fn new_directory(&mut self, dest: usize, name: String) -> FilesystemResult<usize>
    {
        let inode = self.allocate_file(String::new(), 0x4000 | 0o777)?;

        self.add_directory_entry(inode, inode, ".")?;
        self.add_directory_entry(inode, dest, "..")?;

        self.add_directory_entry(dest, inode, &name)?;

        Ok(inode)
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
        kdebugln!(Filesystem, "{} Zones Rewritten", self.rewritten.len());

        for (block, data) in &self.rewritten
        {
            kdebugln!(Filesystem, "Writing to Block {}", block);

            let ptr = data.as_ptr() as *mut u8;

            self.block_driver.sync_write(ptr, 1024, 1024 * *block as u64);
        }

        // Clear the rewritten buffer
        self.rewritten = Vec::new();

        Ok(())
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
    fn path_to_inode(&mut self, path: PathBuffer) -> FilesystemResult<FilesystemIndex>
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
    fn inode_to_path(&mut self, inode: FilesystemIndex) -> FilesystemResult<PathBuffer>
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

            // Add any mounted filesystems
            for (place, root, name) in &self.mount_inodes
            {
                if *place == inode
                {
                    result.push(DirectoryEntry{ index: *root, name: name.clone(), entry_type: DirectoryEntryType::Directory });
                }
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
    fn create_file(&mut self, inode: FilesystemIndex, name: alloc::string::String) -> FilesystemResult<FilesystemIndex>
    {
        if Some(inode.mount_id) == self.mount_id
        {
            let file_inode = self.allocate_file(String::new(), 0o100777)?;

            self.add_directory_entry(inode.inode, file_inode, &name)?;

            Ok(FilesystemIndex { mount_id: inode.mount_id, inode: file_inode } )
        }
        else
        {
            if let Some(vfs) = &mut self.vfs
            {
                vfs.create_file(inode, name)
            }
            else
            {
                Err(FilesystemError::FilesystemNotMounted)
            }
        }
    }

    /// Create a directory in the directory at the given inode
    fn create_directory(&mut self, inode: FilesystemIndex, name: alloc::string::String) -> FilesystemResult<FilesystemIndex>
    {
        if Some(inode.mount_id) == self.mount_id
        {
            let dir_inode = self.new_directory(inode.inode, name)?;

            Ok(FilesystemIndex { mount_id: inode.mount_id, inode: dir_inode } )
        }
        else
        {
            if let Some(vfs) = &mut self.vfs
            {
                vfs.create_directory(inode, name)
            }
            else
            {
                Err(FilesystemError::FilesystemNotMounted)
            }
        }
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

    /// Write data to an inode
    fn write_inode(&mut self, inode: FilesystemIndex, data: &[u8]) -> FilesystemResult<()>
    {
        if Some(inode.mount_id) == self.mount_id
        {
            self.write_to_file(inode.inode, data)
        }
        else
        {
            if let Some(vfs) = &mut self.vfs
            {
                vfs.write_inode(inode, data)
            }
            else
            {
                Err(FilesystemError::FilesystemNotMounted)
            }
        }
    }

    /// Mount a filesystem at the given inode
    fn mount_fs_at(&mut self, inode: FilesystemIndex, root: FilesystemIndex, name: String) -> FilesystemResult<()>
    {
        self.mount_inodes.push((inode, root, name));

        Ok(())
    }

    /// Open a filedescriptor for the given inode
    fn open_fd(&mut self, inode: FilesystemIndex, mode: usize) -> FilesystemResult<Box<dyn crate::process::descriptor::FileDescriptor>>
    {
        if let Some(vfs) = &mut self.vfs
        {
            if Some(inode.mount_id) == self.mount_id
            {
                Ok(Box::new(InodeFileDescriptor::new(vfs, inode, mode).unwrap()))
            }
            else
            {
                vfs.open_fd(inode, mode)   
            }
        }
        else
        {
            Err(FilesystemError::FilesystemNotMounted)
        }
    }

    /// Execute an ioctl command on an inode
    fn exec_ioctl(&mut self, inode: FilesystemIndex, cmd: IOControlCommand) -> FilesystemResult<usize>
    {
        if let Some(vfs) = &mut self.vfs
        {
            if Some(inode.mount_id) == self.mount_id
            {
                // Nothing to do here (yet)
                Ok(usize::MAX)
            }
            else
            {
                vfs.exec_ioctl(inode, cmd)   
            }
        }
        else
        {
            Err(FilesystemError::FilesystemNotMounted)
        }
    }
}

