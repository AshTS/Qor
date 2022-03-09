use crate::*;

/// Open Syscall
pub fn syscall_open(proc: &mut super::Process, path_ptr: usize, flags: usize, _create_mode: usize) -> Result<usize, usize>
{
    let expanded_path = super::utils::userspace_string_to_path(proc, path_ptr)?;

    proc.open(&expanded_path, flags).map_err( |_| errno::ENOENT )
}