/// Stores backup data for traps
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapFrame
{
	pub regs:       [usize; 32], // 0 - 255
	pub fregs:      [usize; 32], // 256 - 511
	pub satp:       usize,       // 512 - 519
	pub trap_stack: *mut u8,     // 520
	pub hartid:     usize,       // 528
}

impl TrapFrame
{
	/// Create a new trap frame (and allocate space for its stack)
	pub fn new(stack_size: usize) -> Self
	{
		// Allocate space for the stack
		let stack_start = crate::mem::kpzalloc(stack_size).unwrap();

		let trap_stack = (stack_start + stack_size * crate::mem::PAGE_SIZE) as *mut u8;

		Self
		{
			regs: [0; 32],
			fregs: [0; 32],
			satp: 0,
			trap_stack,
			hartid: 0
		}
	}

	/// Create a zeroed trap frame
	pub fn zeroed() -> Self
	{
		Self
		{
			regs: [0; 32],
			fregs: [0; 32],
			satp: 0,
			trap_stack: 0 as *mut u8,
			hartid: 0
		}
	}
}