use crate::String;
  
/// Minix3 Superblock
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Minix3SuperBlock
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

/// Minix3 Inode
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct Minix3Inode
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
pub struct Minix3StatData
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
pub struct Minix3DirEntry
{
  pub inode: u32,
  pub name:  [u8; 60],
}

impl Minix3DirEntry
{
  pub fn to_string(&self) -> String
  {
    let mut s = String::new();

    for c in self.name
    {
        if c != 0
        {
            s.push(c as char);
        }
    }

    s
  }
}