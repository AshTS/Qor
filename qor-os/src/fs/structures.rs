use crate::types::{DeviceIdentifier, TimeRepr, UserIdentifier};

/// Inode Index
pub type InodeIndex = usize;

/// Pointer to a device and an inode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InodePointer {
    pub device_id: DeviceIdentifier,
    pub index: InodeIndex,
}

/// Individual Permissions Value
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permissions(u16);

/// Mode structure
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileMode(u16);

/// Directory Entry File Types
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryEntryType {
    Unknown,
    RegularFile,
    Directory,
    CharDevice,
    BlockDevice,
    FirstInFirstOut,
    Socket,
    SymbolicLink,
}

/// Directory Entry
#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub index: InodePointer,
    pub name: alloc::string::String,
    pub entry_type: DirectoryEntryType,
}

/// File Stat Data
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FileStat {
    pub index: InodePointer,
    pub mode: FileMode,
    pub links: u16,
    pub uid: UserIdentifier,
    pub gid: UserIdentifier,
    pub special_dev_id: DeviceIdentifier,
    pub size: usize,
    pub blk_size: usize,
    pub blocks_allocated: usize,
    pub atime: TimeRepr,
    pub mtime: TimeRepr,
    pub ctime: TimeRepr,
}

impl Permissions {
    /// Read Mode
    pub fn read() -> Permissions {
        Permissions(1)
    }

    /// Write Mode
    pub fn write() -> Permissions {
        Permissions(2)
    }

    /// Execute Mode
    pub fn execute() -> Permissions {
        Permissions(4)
    }

    /// Read Write Mode
    pub fn read_write() -> Permissions {
        Self::read() | Self::write()
    }

    /// Read Execute Mode
    pub fn read_execute() -> Permissions {
        Self::read() | Self::execute()
    }

    /// Write Execute Mode
    pub fn write_execute() -> Permissions {
        Self::write() | Self::execute()
    }

    /// Read Write Execute Mode
    pub fn read_write_execute() -> Permissions {
        Self::read() | Self::write() | Self::execute()
    }
}

impl core::ops::BitOr for Permissions {
    type Output = Permissions;

    fn bitor(self, rhs: Self) -> Self::Output {
        Permissions(self.0 | rhs.0)
    }
}

impl FileMode {
    /// Create a new file mode from components
    pub fn from_components(
        entry_type: DirectoryEntryType,
        user: Permissions,
        group: Permissions,
        owner: Permissions,
    ) -> Self {
        Self(((entry_type as u16) << 13) | (user.0 << 6) | (group.0 << 3) | owner.0)
    }

    /// Create a file mode from the raw bits
    pub fn from_bits(data: u16) -> Self {
        Self(data)
    }

    /// Get the entry type
    pub fn entry_type(&self) -> DirectoryEntryType {
        unsafe { core::mem::transmute((self.0 >> 13) & 0b111) }
    }

    /// Set the entry type
    #[allow(clippy::unusual_byte_groupings)]
    pub fn set_entry_type(&mut self, entry_type: DirectoryEntryType) {
        self.0 &= 0b000_111_111_111;
        self.0 |= (entry_type as u16) << 13;
    }

    /// Get the user permissions
    pub fn user_perms(&self) -> Permissions {
        Permissions((self.0 >> 6) & 0b111)
    }

    /// Set the user permissions
    #[allow(clippy::unusual_byte_groupings)]
    pub fn set_user_perms(&mut self, perms: Permissions) {
        self.0 &= 0b111_000_111_111;
        self.0 |= perms.0 << 6;
    }

    /// Get the group permissions
    pub fn group_perms(&self) -> Permissions {
        Permissions((self.0 >> 3) & 0b111)
    }

    /// Set the group permissions
    #[allow(clippy::unusual_byte_groupings)]
    pub fn set_group_perms(&mut self, perms: Permissions) {
        self.0 &= 0b111_111_000_111;
        self.0 |= perms.0 << 3;
    }

    /// Get the owner permissions
    pub fn owner_perms(&self) -> Permissions {
        Permissions(self.0 & 0b111)
    }

    /// Set the owner permissions
    #[allow(clippy::unusual_byte_groupings)]
    pub fn set_owner_perms(&mut self, perms: Permissions) {
        self.0 &= 0b111_111_111_000;
        self.0 |= perms.0;
    }
}
