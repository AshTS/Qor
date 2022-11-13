pub mod generic;
pub use generic::*;

pub mod structures;
pub use structures::*;

pub mod vfs;
pub use vfs::*;

// Global Filesystem Interface
static mut GLOBAL_FILESYSTEM_INTERFACE: Option<alloc::sync::Arc<libutils::sync::Mutex<FilesystemInterface>>> = None;

/// Initialize the global filesystem
pub fn init_global_filesystem(_: libutils::sync::InitThreadMarker) {
    assert!( unsafe { &GLOBAL_FILESYSTEM_INTERFACE }.is_none() );

    unsafe { GLOBAL_FILESYSTEM_INTERFACE.insert(alloc::sync::Arc::new(libutils::sync::Mutex::new(FilesystemInterface::new()))) };
}

/// Get an Arc to the global filesystem
pub fn global_filesystem_arc() -> alloc::sync::Arc<libutils::sync::Mutex<FilesystemInterface>> {
    unsafe { &GLOBAL_FILESYSTEM_INTERFACE }.clone().expect("Global Filesystem Not Initialized")
}