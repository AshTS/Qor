/// User Identifier Type
pub type UserIdentifier = u16;

/// Group Identifier Type
pub type GroupIdentifier = u16;

/// Time Object
/// 
/// Stores the time as the number of nanoseconds ellapsed since Jan 1, 1970
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeRepr(pub usize);

/// Device ID
pub type DeviceIdentifier = usize;