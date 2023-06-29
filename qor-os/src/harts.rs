pub const CORE_COUNT: usize = 2;

#[no_mangle]
#[used]
pub static WAITING_FLAG: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

pub static SYNC_FLAG: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(true);
pub static SYNC_COUNT: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

pub fn enable_other_harts() {
    WAITING_FLAG.store(1, core::sync::atomic::Ordering::Relaxed);
}

pub fn machine_mode_sync() {
    let is_primary_hart = riscv::register::mhartid::read() == 0;

    if is_primary_hart {
        SYNC_COUNT.store(0, core::sync::atomic::Ordering::Release);

        SYNC_FLAG.store(false, core::sync::atomic::Ordering::Release);
        SYNC_FLAG.store(true, core::sync::atomic::Ordering::Release);

        while SYNC_COUNT.load(core::sync::atomic::Ordering::Acquire) + 1 < CORE_COUNT { core::hint::spin_loop() }
    }
    else {
        while SYNC_FLAG.load(core::sync::atomic::Ordering::Acquire) { core::hint::spin_loop() }
        while !SYNC_FLAG.load(core::sync::atomic::Ordering::Acquire) { core::hint::spin_loop() }

        SYNC_COUNT.fetch_add(1, core::sync::atomic::Ordering::AcqRel);
    }
}