
use crate::*;

use super::loading;

use fs::fstrait::Filesystem;

use alloc::vec::Vec;
use libutils::paths::{PathBuffer, OwnedPath};

use super::process::Process;

/// Process Loading Error
#[derive(Debug, Clone)]
pub enum ProcessLoadError
{
    ReadError(fs::structures::FilesystemError),
    NotAnELF,
    NotAnExecutable,
    BadFormat(String)
}


pub fn load_process(interface: &mut fs::vfs::FilesystemInterface, path: PathBuffer, args: &mut Vec<String>, envp: &mut Vec<String>) -> Result<Process, ProcessLoadError>
{
    // Open the file
    let index = interface.path_to_inode(path).map_err(|e| loading::ProcessLoadError::ReadError(e))?;
    let file_data = interface.read_inode(index).map_err(|e| loading::ProcessLoadError::ReadError(e))?;

    // If the file is an ELF file, load that file
    if file_data[0..4] == [0x7F, 'E' as u8, 'L' as u8, 'F' as u8]
    {
        super::elf::load_elf(file_data, path, args, envp)
    }
    else if file_data[0..2] == ['#' as u8, '!' as u8]
    {
        let mut f = String::new();

        for (i, c) in file_data.iter().enumerate()
        {
            if i < 2 { continue }

            if *c == '\n' as u8 { break }

            f.push(*c as char);
        }

        args.insert(0, path.to_string());

        load_process(interface, &OwnedPath::new(f), args, envp)
    }
    else
    {
        Err(loading::ProcessLoadError::NotAnExecutable)
    }

}