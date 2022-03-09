/// Stat structure
#[repr(C)]
pub struct OutputStatStruct
{
    pub dev_id: usize,
    pub inode: usize,
    pub mode: u16,
    pub links: u16,
    pub uid: u16,
    pub gid: u16,
    pub special_dev_id: usize,
    pub size: usize,
    pub blk_size: usize,
    pub blocks_alloced: usize,
    pub atime: usize,
    pub mtime: usize,
    pub ctime: usize
}

/// Stat Syscall
pub fn syscall_stat(proc: &mut super::Process, path_ptr: usize, buffer_ptr: usize) -> Result<usize, usize>
{
    let buffer_ptr = proc.map_mem(buffer_ptr).unwrap() as *mut OutputStatStruct;
    
    let expanded_path = super::utils::userspace_string_to_path(proc, path_ptr)?;

    let stat_data = proc.stat(expanded_path)?;

    unsafe
    {
        let buf = buffer_ptr.as_mut().unwrap();

        buf.dev_id = stat_data.dev_id;
        buf.inode = stat_data.inode;
        buf.mode = stat_data.mode;
        buf.links = stat_data.links;
        buf.uid = stat_data.uid;
        buf.gid = stat_data.gid;
        buf.special_dev_id = stat_data.special_dev_id;
        buf.size = stat_data.size;
        buf.blk_size = stat_data.blk_size;
        buf.blocks_alloced = stat_data.blocks_alloced;
        buf.atime = stat_data.atime;
        buf.mtime = stat_data.mtime;
        buf.ctime = stat_data.ctime;
    }

    Ok(0)
}