/// Minix3 Superblock
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Minix3Superblock {
    pub ninodes: u32,
    pub pad0: u16,
    pub imap_blocks: u16,
    pub zmap_blocks: u16,
    pub first_data_zone: u16,
    pub log_zone_size: u16,
    pub pad1: u16,
    pub max_size: u32,
    pub zones: u32,
    pub magic: u16,
    pub pad2: u16,
    pub block_size: u16,
    pub disk_version: u8,
}

/// Minix3 Inode
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct Minix3Inode {
    pub mode: u16,
    pub nlinks: u16,
    pub uid: u16,
    pub gid: u16,
    pub size: u32,
    pub atime: u32,
    pub mtime: u32,
    pub ctime: u32,
    pub zones: [u32; 10],
}

/// Minix3 Stat Data
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Minix3StatData {
    pub mode: u16,
    pub nlinks: u16,
    pub uid: u16,
    pub gid: u16,
    pub size: u32,
    pub atime: u32,
    pub mtime: u32,
    pub ctime: u32,
}

/// Directory entry inode
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Minix3DirEntry {
    pub inode: u32,
    pub name: [u8; 60],
}

impl core::fmt::Display for Minix3DirEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for c in self.name.iter().filter(|v| **v != 0).map(|v| *v as char) {
            write!(f, "{c}")?;
        }

        Ok(())
    }
}