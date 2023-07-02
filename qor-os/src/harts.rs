use libutils::sync::InitThreadMarker;

include!(concat!(env!("OUT_DIR"), "/core_count.rs"));

#[no_mangle]
#[used]
pub static WAITING_FLAG: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);


#[no_mangle]
#[used]
pub static STACK_COUNTER: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

pub static SYNC_FLAG: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(true);
pub static SYNC_COUNT: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

pub fn enable_other_harts(_marker: InitThreadMarker) {
    STACK_COUNTER.store(unsafe { crate::asm::KERNEL_STACK_END } - 0x10000, core::sync::atomic::Ordering::Release);
    WAITING_FLAG.store(1, core::sync::atomic::Ordering::Release);
}

pub fn machine_mode_is_primary_hart() -> bool {
    riscv::register::mhartid::read() == 0
}

pub fn machine_mode_sync() {
    let is_primary_hart = riscv::register::mhartid::read() == 0;

    if is_primary_hart {
        SYNC_COUNT.store(0, core::sync::atomic::Ordering::Release);

        SYNC_FLAG.store(false, core::sync::atomic::Ordering::Release);

        // while SYNC_COUNT.load(core::sync::atomic::Ordering::Acquire) + 1 < CORE_COUNT { core::hint::spin_loop() }

        loop {
            let v = SYNC_COUNT.load(core::sync::atomic::Ordering::Acquire);
            // kdebugln!(unsafe "{}", v);
            if v + 1 >= CORE_COUNT { break; }
        }

        SYNC_FLAG.store(true, core::sync::atomic::Ordering::Release);
    }
    else {
        while SYNC_FLAG.load(core::sync::atomic::Ordering::Acquire) { core::hint::spin_loop() }
        SYNC_COUNT.fetch_add(1, core::sync::atomic::Ordering::AcqRel);
        while !SYNC_FLAG.load(core::sync::atomic::Ordering::Acquire) { core::hint::spin_loop() }
    }
}