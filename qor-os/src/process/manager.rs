use core::u16;

use crate::*;

use alloc::collections::BTreeMap;

use super::process::ProcessData;

/// Process Manager
pub struct ProcessManager
{
    processes: BTreeMap<u16, ProcessData>,
    next_pid: u16,
    current: Option<u16>
}

impl ProcessManager
{
    /// Initialize the process manager
    pub fn new() -> Self
    {
        Self
        {
            processes: BTreeMap::new(),
            next_pid: 0,
            current: None
        }
    }

    /// Get process by pid
    pub fn process_by_pid(&self, pid: u16) -> Option<&ProcessData>
    {
        self.processes.get(&pid)
    }


    /// Add a new process (from process data)
    pub fn add_process(&mut self, mut data: ProcessData) -> u16
    {
        data.start();
        let pid = data.get_pid();

        if self.processes.insert(pid, data).is_some()
        {
            panic!("Attempted to add new process with pid {}, but that pid was already in use", pid);
        }

        pid
    }

    // FIXME: This scheduler is a massive hack
    /// Increment the pid counter and return the new value
    pub fn get_next_pid(&mut self) -> u16
    {
        let last_key = self.processes.len() as u16;

        if last_key == 0
        {
            return 0;
        }

        self.next_pid += 1;
        self.next_pid %= last_key;

        self.current = Some(self.next_pid);

        self.next_pid
    }

    /// Get mutable reference to the currently running process
    pub fn get_mut_current(&mut self) -> Option<&mut ProcessData>
    {
        if let Some(current) = self.current
        {
            self.processes.get_mut(&current)
        }
        else
        {
            None
        }   
    }
}