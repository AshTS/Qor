/// sigreturn Syscall
pub fn syscall_sigreturn(proc: &mut super::Process)
{
    proc.return_from_signal();
}