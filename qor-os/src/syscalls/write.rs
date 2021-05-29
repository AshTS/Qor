use crate::*;

/// Write Syscall
pub fn syscall_write(proc: &mut super::Process, ptr: usize) -> usize
{
    let ptr = proc.map_mem(ptr).unwrap() as *mut u8;


    let mut i = 0;

    loop
    {
        let c = unsafe { ptr.add(i).read() };

        if c == 0
        {
            break;
        }

        kprint!("{}", c as char);
        i += 1;
    }

    i
}