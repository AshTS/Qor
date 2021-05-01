#[repr(C)]
/// Header of the request
pub struct BlockDeviceRequestHeader
{
	pub blktype:  u32,
	pub reserved: u32,
	pub sector:   u64,
}

#[repr(C)]
/// Pointer to the data for the request
pub struct BlockDeviceRequestData
{
	pub data: *mut u8,
}

#[repr(C)]
/// Status of the request
pub struct BlockDeviceRequestStatus
{
	pub status: u8,
}

#[repr(C)]
/// Block Device Driver Request object
pub struct BlockDeviceRequest
{
	pub header: BlockDeviceRequestHeader,
	pub data:   BlockDeviceRequestData,
	pub status: BlockDeviceRequestStatus,
	pub head:   u16,
}

impl BlockDeviceRequest
{
	/// Create a new Block Device Request Object
	pub fn new() -> Self
	{
		Self
		{
			header: BlockDeviceRequestHeader {blktype: 0, reserved: 0, sector: 0},
			data: BlockDeviceRequestData {data: 0 as *mut u8},
			status: BlockDeviceRequestStatus {status: 0},
			head: 0
		}
	}
}