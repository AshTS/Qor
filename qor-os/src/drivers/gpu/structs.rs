/// Framebuffer Fixed Screen Info Structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FramebufferFixedInfo
{
	pub name: [u8; 16],
	pub buffer_start: u64,

	pub buffer_len: u32,
	pub fb_type: u32,
	pub aux_type: u32,
	pub visual: u32,
	pub x_pan_step: u16,
	pub y_pan_step: u16,
	pub y_wrap_step: u16,

	pub line_length: u32,

	pub mmio_len: u32,
	pub accel: u32,

	pub capabilities: u16,
	pub reserved: [u16; 2]
}

/// Framebuffer bit field definition
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FramebufferBitfield
{
    pub offset: u32,
    pub length: u32,
    pub msb_right: u32
}

/// Framebuffer Variable Screen Info Structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FramebufferVariableInfo
{
    pub x_res: u32,
    pub y_res: u32,
    pub x_res_virtual: u32,
    pub y_res_virtual: u32,
    pub x_offset: u32,
    pub y_offset: u32,

    pub bits_per_pixel: u32,
    pub grayscale: u32,

    pub red: FramebufferBitfield,
    pub green: FramebufferBitfield,
    pub blue: FramebufferBitfield,
    pub transp: FramebufferBitfield,

    pub non_std: u32,

    pub activate: u32,

    pub height: u32,
    pub width: u32,

    pub obsolete_flags: u32,

    pub unused_timing: [u32; 15]
}