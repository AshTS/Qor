/// Minix3 Superblock
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
    /// Create a new, zeroed superblock
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

/// Minix3 Inode
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

/// Minix3 Stat Data
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

/// Directory entry inode
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DirEntry
{
  pub inode: u32,
  pub name:  [u8; 60],
}