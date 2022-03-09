/// rmdir Syscall
pub fn syscall_rmdir(proc: &mut super::Process, path_ptr: usize) -> Result<usize, usize>
{
    let expanded_path = super::utils::userspace_string_to_path(proc, path_ptr)?;

    proc.rmdir(expanded_path)?;

    Ok(0)
}