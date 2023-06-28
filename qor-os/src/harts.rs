#[no_mangle]
#[used]
pub static WAITING_FLAG: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

pub fn enable_other_harts() {
    WAITING_FLAG.store(1, core::sync::atomic::Ordering::Relaxed);
}