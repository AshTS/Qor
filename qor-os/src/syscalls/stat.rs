use libutils::paths::OwnedPath;
use crate::*;

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
pub fn syscall_stat(proc: &mut super::Process, path_ptr: usize, buffer_ptr: usize) -> usize
{
    let path_ptr = proc.map_mem(path_ptr).unwrap() as *mut u8;
    let buffer_ptr = proc.map_mem(buffer_ptr).unwrap() as *mut OutputStatStruct;
    let mut path = String::new();

    let mut i = 0; 

    loop
    {
        let v = unsafe { path_ptr.add(i).read() } as char;

        if v == '\x00' { break; }

        path.push(v);

        i += 1;
    }

    let mut expanded_path = OwnedPath::new(path);
    expanded_path.canonicalize(&proc.data.cwd);

    kwarnln!("Stat for path: {}", expanded_path);

    match  proc.stat(expanded_path)
    {
        Ok(stat_data) => 
        {
            kdebugln!("{:?}", stat_data);

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

            0
        },
        Err(v) => v,
    }
}