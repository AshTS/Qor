// Constants recieved from the linker script
extern "C" {
	static TEXT_START: usize;
    static TEXT_END: usize;
    static RODATA_START: usize;
    static RODATA_END: usize;
    static DATA_START: usize;
    static DATA_END: usize;
    static BSS_START: usize;
    static BSS_END: usize;
    static STACK_START: usize;
    static STACK_END: usize;
    static HEAP_START: usize;
    static HEAP_END: usize;
}

/// Get the text start address as a usize
/// Safety: Because this value should have been read properly from the linker
/// script, this is safe
pub fn text_start() -> usize
{
	unsafe { TEXT_START }
}

/// Get the text end address as a usize
/// Safety: Because this value should have been read properly from the linker
/// script, this is safe
pub fn text_end() -> usize
{
	unsafe { TEXT_END }
}

/// Get the rodata start address as a usize
/// Safety: Because this value should have been read properly from the linker
/// script, this is safe
pub fn rodata_start() -> usize
{
	unsafe { RODATA_START }
}

/// Get the rodata end address as a usize
/// Safety: Because this value should have been read properly from the linker
/// script, this is safe
pub fn rodata_end() -> usize
{
	unsafe { RODATA_END }
}

/// Get the data start address as a usize
/// Safety: Because this value should have been read properly from the linker
/// script, this is safe
pub fn data_start() -> usize
{
	unsafe { DATA_START }
}

/// Get the data end address as a usize
/// Safety: Because this value should have been read properly from the linker
/// script, this is safe
pub fn data_end() -> usize
{
	unsafe { DATA_END }
}

/// Get the bss start address as a usize
/// Safety: Because this value should have been read properly from the linker
/// script, this is safe
pub fn bss_start() -> usize
{
	unsafe { BSS_START }
}

/// Get the bss end address as a usize
/// Safety: Because this value should have been read properly from the linker
/// script, this is safe
pub fn bss_end() -> usize
{
	unsafe { BSS_END }
}

/// Get the stack start address as a usize
/// Safety: Because this value should have been read properly from the linker
/// script, this is safe
pub fn stack_start() -> usize
{
	unsafe { STACK_START }
}

/// Get the stack end address as a usize
/// Safety: Because this value should have been read properly from the linker
/// script, this is safe
pub fn stack_end() -> usize
{
	unsafe { STACK_END }
}

/// Get the heap start address as a usize
/// Safety: Because this value should have been read properly from the linker
/// script, this is safe
pub fn heap_start() -> usize
{
	unsafe { HEAP_START }
}

/// Get the heap end address as a usize
/// Safety: Because this value should have been read properly from the linker
/// script, this is safe
pub fn heap_end() -> usize
{
	unsafe { HEAP_END }
}