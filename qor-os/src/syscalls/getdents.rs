/// Write a directory entry at the given pointer, returning the number of bytes written
fn write_dir_entry(mut ptr: usize, inode: usize, offset: usize, name: &str) -> usize
{
    let length = 8 + 8 + 2 + name.len() + 1;

    // Write the inode number
    unsafe { (ptr as *mut u64).write(inode as u64); }
    ptr += 8;

    // Write the offset
    unsafe { (ptr as *mut u64).write(offset as u64); }
    ptr += 8;

    // Write the length
    unsafe { (ptr as *mut u16).write(length as u16); }
    ptr += 2;

    // Write the string
    for (i, c) in name.chars().enumerate()
    {
        unsafe { ((ptr + i) as *mut u8).write(c as u8); }
    }

    // Write the zero terminator
    unsafe { ((ptr + name.len()) as *mut u8).write(0) }

    length
}

/// Getdents Syscall
pub fn syscall_getdents(proc: &mut super::Process, fd: usize, buffer_ptr: usize, size: usize) -> usize
{
    let buffer_ptr = proc.map_mem(buffer_ptr).unwrap();

    let mut amount_written = 0;

    if let Some(entries) = proc.get_dir_entries(fd)
    {
        for entry in entries
        {
            let length = 8 + 8 + 2 + entry.name.len() + 1;
            if amount_written + length >= size
            {
                break;
            }

            let size = write_dir_entry(buffer_ptr + amount_written, entry.index.inode as usize, amount_written, &entry.name);

            amount_written += size;
        }

        amount_written
    }
    else
    {
        usize::MAX
    }
}