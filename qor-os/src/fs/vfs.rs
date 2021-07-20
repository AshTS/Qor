use crate::*;

use super::fstrait::Filesystem;
use super::structures::*;

use alloc::collections::BTreeMap;
use alloc::format;

static VFS_INTERFACE: core::sync::atomic::AtomicPtr<FilesystemInterface> = core::sync::atomic::AtomicPtr::new(0 as *mut FilesystemInterface);

/// Get a reference to the vfs interface
pub fn get_vfs_reference() -> Option<&'static mut FilesystemInterface>
{
    let ptr = VFS_INTERFACE.load(core::sync::atomic::Ordering::SeqCst);

    unsafe { ptr.as_mut() }
}
 
/// Virtual Filesystem Interface
pub struct FilesystemInterface
{
    mounts: Vec<Option<Box<dyn Filesystem>>>,
    root: Option<usize>,
    pub index: BTreeMap<String, FilesystemIndex>,
    indexed: BTreeMap<FilesystemIndex, String>
}

impl FilesystemInterface
{
    /// Create a new Filesystem Interface
    pub fn new() -> &'static mut Self
    {
        if !get_vfs_reference().is_none()
        {
            panic!("Cannot initialize multiple Virtual Filesystem Interfaces");
        }

        let singleton = Box::new(Self
        {
            mounts: Vec::new(),
            root: None,
            index: BTreeMap::new(),
            indexed: BTreeMap::new()
        });

        let reference = Box::leak(singleton);

        VFS_INTERFACE.store(reference as *mut FilesystemInterface, core::sync::atomic::Ordering::SeqCst);

        unsafe { (reference as *mut FilesystemInterface).as_mut().unwrap() } 
    }

    /// Mount a filesystem to the vfs
    pub fn mount_fs(&mut self, path: &str, mut fs: Box<dyn Filesystem>) -> Result<(), FilesystemError>
    {
        kdebugln!(Filesystem, "Mounting filesystem to index {} at {}", self.mounts.len(), path);

        // Set the mount id
        let id = self.mounts.len();
        fs.set_mount_id(id, unsafe { (self as *mut FilesystemInterface).as_mut().unwrap() });

        // Add the mount
        self.mounts.push(Some(fs));

        // Add the mapping to the mount paths
        if path == "/"
        {
            self.root = Some(id);

            Ok(())
        }
        else
        {
            Err(FilesystemError::MissingRootMount)
        }
    }

    /// Get the fs mounted at the given index
    pub fn get_fs_mount(&mut self, id: usize) -> Option<&mut Box<dyn Filesystem>>
    {
        if let Some(mount) = self.mounts.iter_mut().nth(id)
        {
            mount.as_mut()
        }
        else
        {
            None
        }
    }

    /// Get the fs mounted at the given index
    pub fn get_fs_mount_error(&mut self, id: usize) -> FilesystemResult<&mut Box<dyn Filesystem>>
    {
        if let Some(mount) = self.mounts.iter_mut().nth(id)
        {
            if let Some(mnt) = mount
            {
                Ok(mnt)
            }
            else
            {
                Err(FilesystemError::UnableToFindDiskMount(id))
            }
        }
        else
        {
            Err(FilesystemError::UnableToFindDiskMount(id))
        }
    }

    /// Get the root filesystem
    pub fn get_root_fs(&mut self) -> Option<&mut Box<dyn Filesystem>>
    {
        if let Some(id) = self.root
        {
            self.get_fs_mount(id)
        }
        else
        {
            None
        }
    }

    /// Get the root filesystem or return an error
    pub fn get_root_fs_error(&mut self) -> FilesystemResult<&mut Box<dyn Filesystem>>
    {
        if let Some(id) = self.root
        {
            self.get_fs_mount_error(id)
        }
        else
        {
            Err(FilesystemError::MissingRootMount)
        }
    }

    /// Index the filesystem from the starting path and starting inode
    pub fn index_from(&mut self, path: &str, inode: FilesystemIndex) -> FilesystemResult<()>
    {
        if self.indexed.contains_key(&inode)
        {
            return Ok(());
        }

        // Add the current path to the index
        if path.len() > 0
        {
            self.index.insert(path.to_string(), inode);
        }
        self.indexed.insert(inode, path.to_string());

        // Get the directory entries (if this is a directory)
        match self.get_dir_entries(inode)
        {
            Ok(entries) =>
            {
                self.index.insert(path.to_string() + "/", inode);

                for entry in entries
                {
                    self.index_from(&format!("{}/{}", path, entry.name), entry.index)?;
                }

                Ok(())
            },
            Err(FilesystemError::INodeIsNotADirectory) => Ok(()),
            Err(e) => Err(e)
        }
    }

    /// Index the full filesystem
    pub fn index(&mut self) -> FilesystemResult<()>
    {
        // Clear the previous index
        self.index = BTreeMap::new();
        self.indexed = BTreeMap::new();

        let root = self.get_root_index()?;
        self.index_from("", root)
    }

    /// Invalidate part of the filesystem index
    pub fn invalidate_index(&mut self, path: &str) -> FilesystemResult<()>
    {
        let mut to_remove = Vec::new();

        for key in self.index.keys()
        {
            if key.starts_with(path)
            {
                to_remove.push(key.to_string());
            }
        }
        
        for key in to_remove
        {
            self.index.remove(&key);
        }

        Ok(())
    }
}

impl Filesystem for FilesystemInterface
{
    /// Initialize the filesystem on the current disk
    fn init(&mut self) -> FilesystemResult<()>
    {
        kdebugln!(Filesystem, "Initialize Virtual Filesystem");
        // Nothing to do here, the virtual file system doesn't need any initialization

        Ok(())
    }

    /// Sync the filesystem with the current disk
    fn sync(&mut self) -> FilesystemResult<()>
    {
        kdebugln!(Filesystem, "Syncing Virtual Filesystem");

        // To sync the entire filesystem just sync all mounted file systems
        for fs in &mut self.mounts
        {
            if let Some(fs) = fs
            {
                fs.sync()?;
            }
        }

        Ok(())
    }

    /// Set the mount_id of the filesystem
    fn set_mount_id(&mut self, _mount_id: usize, _vfs: &'static mut FilesystemInterface)
    {
        panic!("Cannot mount Virtual Filesystem");
    }

    /// Get the index of the root directory of the filesystem
    fn get_root_index(&mut self) -> FilesystemResult<FilesystemIndex>
    {
        self.get_root_fs_error()?.get_root_index()
    }

    /// Convert a path to an inode
    fn path_to_inode(&mut self, path: &str) -> FilesystemResult<FilesystemIndex>
    {
        // If we have the path in the index, just use that
        if let Some(index) = self.index.get(path)
        {
            kdebugln!(Filesystem, "Map path `{}` to inode -> {:?}", path, index);
            Ok(*index)
        }

        // Otherwise, we will walk the filesystem
        else
        {
            // Walking ending index
            let mut walking_end_index = path.len() - 1;

            // Continue until we either hit an empty string or a 
            let mut index = loop
            {
                if walking_end_index == 0
                {
                    break self.get_root_index()?;
                }

                if let Some(index) = self.index.get(&path[0..walking_end_index])
                {
                    break *index;
                }

                walking_end_index -= 1;

                // Walk backwards until the last character is a '/' or we run out of string
                while walking_end_index > 0 && path.chars().nth(walking_end_index - 1) != Some('/')
                {
                    walking_end_index -= 1;
                }
            };

            if path.chars().nth(walking_end_index) == Some('/')
            {
                walking_end_index += 1;
            }

            for name in path[walking_end_index..].split("/")
            {
                let mut found = false;
                for entry in self.get_dir_entries(index)?
                {
                    if entry.name == name
                    {
                        found = true;
                        index = entry.index;
                    }
                }

                if !found
                {
                    kdebugln!(Filesystem, "Map path `{}` to inode -> File Not Found", path);
                    return Err(FilesystemError::FileNotFound(path.to_string()));
                }
            }

            Ok(index)
        }
    }

    /// Convert an inode to a path
    fn inode_to_path(&mut self, inode: FilesystemIndex) -> FilesystemResult<&str>
    {
        // If we have the inode in the index, just use that
        if let Some(path) = self.indexed.get(&inode)
        {
            kdebugln!(Filesystem, "Map inode {:?} to path -> `{}`", inode, path);
            Ok(path)
        }
        else
        {
            todo!()
        }
    }

    /// Get the directory entries for the given inode
    fn get_dir_entries(&mut self, inode: FilesystemIndex) -> FilesystemResult<Vec<DirectoryEntry>>
    {
        kdebugln!(Filesystem, "List Directory Entries at {:?}", inode);
        if let Some(fs) = self.get_fs_mount(inode.mount_id)
        {
            fs.get_dir_entries(inode)
        }
        else
        {
            Err(FilesystemError::UnableToFindDiskMount(inode.mount_id))
        }
    }

    /// Create a file in the directory at the given inode
    fn create_file(&mut self, inode: FilesystemIndex, name: String) -> FilesystemResult<FilesystemIndex>
    {
        kdebugln!(Filesystem, "Create file `{}` at {:?}", name, inode);

        if let Some(fs) = self.get_fs_mount(inode.mount_id)
        {
            fs.create_file(inode, name)
        }
        else
        {
            Err(FilesystemError::UnableToFindDiskMount(inode.mount_id))
        }
    }

    /// Create a directory in the directory at the given inode
    fn create_directory(&mut self, inode: FilesystemIndex, name: String) -> FilesystemResult<FilesystemIndex>
    {
        kdebugln!(Filesystem, "Create directory `{}` at {:?}", name, inode);

        if let Some(fs) = self.get_fs_mount(inode.mount_id)
        {
            fs.create_directory(inode, name)
        }
        else
        {
            Err(FilesystemError::UnableToFindDiskMount(inode.mount_id))
        }
    }

    /// Remove an inode at the given index from the given directory
    fn remove_inode(&mut self, inode: FilesystemIndex, directory: FilesystemIndex) -> FilesystemResult<()>
    {
        kdebugln!(Filesystem, "Remove inode {:?} in directory {:?}", inode, directory);

        if let Some(fs) = self.get_fs_mount(inode.mount_id)
        {
            fs.remove_inode(inode, directory)
        }
        else
        {
            Err(FilesystemError::UnableToFindDiskMount(inode.mount_id))
        }
    }

    /// Read the data stored in an inode
    fn read_inode(&mut self, inode: FilesystemIndex) -> FilesystemResult<Vec<u8>>
    {
        kdebugln!(Filesystem, "Read inode {:?}", inode);

        if let Some(fs) = self.get_fs_mount(inode.mount_id)
        {
            fs.read_inode(inode)
        }
        else
        {
            Err(FilesystemError::UnableToFindDiskMount(inode.mount_id))
        }
    }
}