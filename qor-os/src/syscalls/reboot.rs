use crate::*;

const REBOOT_CMD_HALT: u32 = 0xcdef0123;

/// Reboot Syscall
pub fn syscall_reboot(_proc: &mut super::Process, magic1: usize, magic2: usize, cmd: usize, _args: usize) -> usize
{
    // Verify the magic
    if magic1 as u32 != 0xfee1dead || magic2 as u32 != 0x516f7200
    {
        return errno::EINVAL;
    }

    // Check the command
    if cmd as u32 == REBOOT_CMD_HALT
    {
        halt::kernel_halt();
        return 0;
    }
    else
    {
        return errno::EINVAL;
    }
}