use crate::*;

use process::process::ProcessState;
use drivers::timer::KernelTime;

/// Incoming representation of time
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct IncomingTime
{
    seconds: usize,
    nano_seconds: usize
}

/// This is beyond unsafe, but this is what happens when we interact with C like
/// this
fn map_ptr<T>(proc: &mut super::Process, ptr: usize) -> &'static mut T
{
    unsafe { (proc.map_mem(ptr).unwrap() as *mut T).as_mut() }.unwrap()
}

/// Nanosleep Syscall
pub fn syscall_nanosleep(proc: &mut super::Process, time: usize, _remaining: usize) -> usize
{
    let time: &'static mut IncomingTime = map_ptr(proc, time);
    let kernel_duration = KernelTime::nanoseconds(time.seconds * 1_000_000_000 + time.nano_seconds);
    let current = unsafe { &drivers::TIMER_DRIVER }.time();

    proc.state = ProcessState::Sleeping { wake_time:  current + kernel_duration };

    proc.program_counter += 4;

    let schedule = process::scheduler::schedule_next();
    process::scheduler::schedule_jump(schedule);
}