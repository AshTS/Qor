/// Getcwd Syscall
pub fn syscall_getcwd(proc: &mut super::Process, buffer_ptr: usize, size: usize) -> usize
{
    let buffer = proc.map_mem(buffer_ptr).unwrap() as *mut u8;

    let mut i = 0;
    for c in proc.data.cwd.as_bytes()
    {
        if i == size
        {
            break;
        }
        else
        {
            unsafe 
            {
                buffer.add(i).write(*c);
            }
        }

        i += 1;
    }

    i
}