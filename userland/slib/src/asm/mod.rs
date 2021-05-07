global_asm!(include_str!("syscall.S"));

extern "C"
{
	pub fn make_syscall(syscall: usize, arg0: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize, arg5: usize) -> usize;
}

extern "C"
{
	pub fn _start();
}