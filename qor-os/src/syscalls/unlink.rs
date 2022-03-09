/// unlink Syscall
pub fn syscall_unlink(proc: &mut super::Process, path_ptr: usize) -> Result<usize, usize>
{
    let expanded_path = super::utils::userspace_string_to_path(proc, path_ptr)?;

    proc.unlink(expanded_path)?;

    Ok(0)
}