use crate::*;


use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;

/// Filesystem Error
#[derive(Debug, Clone)]
pub enum FilesystemError
{
    NotMinix3,
    FileNotFound(String),
    INodeNotFound(usize),
}

/// Minix3 Filesystem Interface
pub struct FilesystemInterface
{

}

impl FilesystemInterface
{
    /// Create a new Filesystem Interface
    pub fn new(block_device_driver: usize) -> Self
    {
        Self
        {
        }
    }

    /// Initialize the filesystem interface
    pub fn init_fs(&mut self)
    {

    }
}