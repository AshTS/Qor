use crate::*;

use super::consts::*;

use crate::mem::PAGE_SIZE;

pub use crate::resources::colors::Pixel;

/// Command header for VirtIO GPU
#[repr(C)]
pub struct CtrlHeader
{
   pub ctrl_type: CtrlType,
   pub flags: u32,
   pub fence_id: u64,
   pub ctx_id: u32,
   pub padding: u32
}

impl core::default::Default for CtrlHeader
{
    fn default() -> Self
    {
        Self
        {
            ctrl_type: CtrlType::Null,
            flags: 0,
            fence_id: 0,
            ctx_id: 0,
            padding: 0
        }
    }
}

/// Pixel Formats
#[repr(u32)]
pub enum Formats
{
   B8G8R8A8Unorm = 1,
   B8G8R8X8Unorm = 2,
   A8R8G8B8Unorm = 3,
   X8R8G8B8Unorm = 4,
   R8G8B8A8Unorm = 67,
   X8B8G8R8Unorm = 68,
   A8B8G8R8Unorm = 121,
   R8G8B8X8Unorm = 134,
}

/// Framebuffer Object
pub struct Framebuffer
{
    pub data: *mut Pixel,
    width: usize,
    height: usize
}

impl Framebuffer
{
    /// Create a new Framebuffer
    pub fn new(width: usize, height: usize) -> Self
    {
        let page_count = (width * height * 4 + PAGE_SIZE - 1) / PAGE_SIZE;
        
        let data = crate::mem::kpzalloc(page_count, "Framebuffer").unwrap() as *mut Pixel;

        for i in 0..width * height
        {
            *unsafe { data.add(i).as_mut().unwrap() } = Pixel::new(0, 0, 0);
        }

        Self
        {
            data: data,
            width,
            height
        }
    }

    /// Access a pixel in the framebuffer
    pub fn pixel_mut(&mut self, x: usize, y: usize) -> &mut Pixel
    {
        assert!(x < self.width);
        assert!(y < self.height);

        let offset = x + self.width * y;

        unsafe { self.data.add(offset).as_mut().unwrap() }
    }

    /// Get the width and height
    pub fn get_size(&self) -> (usize, usize)
    {
        (self.width, self.height)
    }

    /// Get the pointer to the frame buffer
    pub fn get_pointer(&self) -> *mut Pixel
    {
        self.data
    }
}

#[repr(C)]
pub struct Request<RqT, RpT: Default>
{
    pub request: RqT,
    pub response: RpT,
}

impl<RqT, RpT: Default> Request<RqT, RpT>
{
    pub fn new(request: RqT) -> *mut Self
    {
        let ptr = Box::new(Self {request, response: RpT::default()});
        
        Box::leak(ptr) as *mut Self
    }
}

#[repr(C)]
pub struct Request3<RqT, RmT, RpT: Default>
{
    pub request: RqT,
    pub mementries: RmT,
    pub response: RpT,
}

impl<RqT, RmT, RpT: Default> Request3<RqT, RmT, RpT>
{
    pub fn new(request: RqT, meminfo: RmT) -> *mut Self 
    {
        let ptr = Box::new(Self {request, mementries: meminfo, response: RpT::default()});
        
        Box::leak(ptr) as *mut Self
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Rect
{
	pub x: u32,
	pub y: u32,
	pub width: u32,
	pub height: u32,
}

impl Rect
{
	pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self
    {
		Self
        {
			x, y, width, height
		}
	}
}

#[repr(C)]
pub struct ResourceCreate2d
{ 
    pub hdr: CtrlHeader,
    pub resource_id: u32,
    pub format: Formats,
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
pub struct ResourceUnref
{
	pub hdr: CtrlHeader,
	pub resource_id: u32,
	pub padding: u32,
}

#[repr(C)]
pub struct SetScanout
{
	pub hdr: CtrlHeader,
	pub r: Rect,
	pub scanout_id: u32,
	pub resource_id: u32,
}

#[repr(C)]
pub struct ResourceFlush
{
	pub hdr: CtrlHeader,
	pub r: Rect,
	pub resource_id: u32,
	pub padding: u32,
}

#[repr(C)]
pub struct TransferToHost2d
{
	pub hdr: CtrlHeader,
	pub r: Rect,
	pub offset: u64,
	pub resource_id: u32,
	pub padding: u32,
}

#[repr(C)]
pub struct AttachBacking
{
	pub hdr: CtrlHeader,
	pub resource_id: u32,
	pub nr_entries: u32,
}

#[repr(C)]
pub struct MemEntry
{
	pub addr: u64,
	pub length: u32,
	pub padding: u32,
}

#[repr(C)]
pub struct DetachBacking
{
	pub hdr: CtrlHeader,
	pub resource_id: u32,
	pub padding: u32,
}

#[repr(C)]
pub struct CursorPos
{
	pub scanout_id: u32,
	pub x: u32,
	pub y: u32,
	pub padding: u32,
}

#[repr(C)]
pub struct UpdateCursor
{
	pub hdr: CtrlHeader,
	pub pos: CursorPos,
	pub resource_id: u32,
	pub hot_x: u32,
	pub hot_y: u32,
	pub padding: u32,
}

