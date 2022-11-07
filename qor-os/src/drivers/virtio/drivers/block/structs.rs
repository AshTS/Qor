use core::mem::size_of;

#[repr(C)]
pub struct Header {
    pub blktype: u32,
    pub reserved: u32,
    pub sector: u64,
}

impl Header {
    pub fn new() -> Self {
        Self {
            blktype: 0,
            reserved: 0,
            sector: 0,
        }
    }
}

#[repr(C)]
pub struct Data {
    pub data: *mut u8,
}

impl Data {
    pub fn new() -> Self {
        Self { data: 0 as *mut u8 }
    }
}

#[repr(C)]
pub struct Status {
    pub status: u8,
}

impl Status {
    pub fn new() -> Self {
        Self { status: 0 }
    }
}

pub const STATUS_OFFSET: usize = size_of::<Header>() + size_of::<Data>();

#[repr(C)]
#[repr(align(32))]
pub struct Request {
    pub header: Header,
    pub data: Data,
    pub status: Status,
    pub head: u16,
}

impl Request {
    pub fn new() -> Self {
        Self {
            header: Header::new(),
            data: Data::new(),
            status: Status::new(),
            head: 0,
        }
    }
}
