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

    /// Get a mutable process by pid
    pub fn process_by_pid_mut(&mut self, pid: u16) -> Option<&mut ProcessData>
    {
        self.processes.get_mut(&pid)
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

    /// Load an elf file
    pub fn load_elf(&mut self, interface: &mut fs::FileSystemInterface, name: &str) -> Result<u16, fs::FileSystemError>
    {
        let stat = interface.stat_file(name)?;

        let mut buffer = Box::new(vec![0u8; stat.size as usize]);

        interface.read_file(name, &mut buffer, stat.size as usize)?;
        
        let data = match elf::load_elf(&buffer)
        {
            Err(e) => { panic!("Unable to load Elf: `{}`", e.msg); },
            Ok(data) => { data}
        };

        Ok(process::get_process_manager().unwrap().add_process(data))
    }

    /// Remove a process
    pub fn remove_process(&mut self, pid: u16)
    {
        self.processes.remove_entry(&pid);
    }

    /// Schedule the next process
    pub fn next_scheduled_pid(&mut self) -> Option<u16>
    {
        // Loop to try to find a process
        loop
        {
            // If there are no processes remaining, do not schedule anything
            if self.processes.keys().len() == 0
            {
                return None;
            }
            
            self.next_pid += 1;
            self.next_pid %= self.processes.last_key_value().unwrap().0 + 1 - self.processes.first_key_value().unwrap().0;
            self.next_pid += self.processes.first_key_value().unwrap().0;

            if let Some(proc) = self.process_by_pid(self.next_pid)
            {
                if proc.is_running()
                {
                    self.current = Some(self.next_pid);
                    return self.current;
                }

                if proc.is_dead()
                {
                    self.remove_process(self.next_pid);
                }
            }
        }
    }
}