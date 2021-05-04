use crate::*;
use alloc::format;

pub struct ElfError
{
    pub msg: String
}

pub struct ElfParser
{

}

pub fn load_elf(buffer: &[u8]) -> Result<ElfParser, ElfError>
{
    kdebugln!(ElfParsing, "Attempting to parse elf file");
    Err(ElfError{msg: format!("Elf Loader is not yet implemented")})
}