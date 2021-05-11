use crate::*;

pub fn syscall_write(process: &mut process::process::ProcessData, ptr: usize)
{
    let ptr = process.map_ptr(ptr) as *const u8;
    let mut i = 0;
    loop
    {
        let v = unsafe { ptr.add(i).read() };

        if v == 0
        {
            break;
        }
        else
        {
            kprint!("{}", v as char);
        }

        i += 1;
    }
}