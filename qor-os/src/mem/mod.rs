//! Memory management for the Qor Kernel

/// Size of a page of memory
pub const PAGE_SIZE: usize = 4096;

/// Type to conveniently refer to a page of memory
pub type Page = [u8; PAGE_SIZE];

/// Global Page Allocator
pub static PAGE_ALLOCATOR: GlobalPageAllocator = GlobalPageAllocator::new();

pub mod allocator;
pub use allocator::*;

pub mod error;
pub use error::*;

pub mod lds;
pub use lds::*;

pub mod pages;
pub use pages::*;

pub mod pagetable;
pub use pagetable::*;

/// Set a page table as the global page table
pub fn set_page_table(page_table: &PageTable)
{
    let addr = page_table as *const PageTable as usize;

    unsafe { riscv::register::satp::set(riscv::register::satp::Mode::Sv39, 0, addr >> 12) };
}

/// Identity map the kernel
pub fn identity_map_kernel(page_table: &mut PageTable)
{
    // Identity map the kernel
    page_table.identity_map(lds::text_start(), lds::text_end(), RWXFlags::ReadExecute, UGFlags::None);
    page_table.identity_map(lds::rodata_start(), lds::rodata_end(), RWXFlags::ReadExecute, UGFlags::None);
    page_table.identity_map(lds::data_start(), lds::data_end(), RWXFlags::ReadWrite, UGFlags::None);
    page_table.identity_map(lds::bss_start(), lds::bss_end(), RWXFlags::ReadWrite, UGFlags::None);
    page_table.identity_map(lds::stack_start(), lds::stack_end(), RWXFlags::ReadWrite, UGFlags::None);
    page_table.identity_map(lds::heap_start(), lds::heap_end(), RWXFlags::ReadWrite, UGFlags::None);

    // Identity map test handle
    page_table.identity_map(0x10_0000, 0x10_0fff, RWXFlags::ReadWrite, UGFlags::None);

    // Identity map the real time clock
    page_table.identity_map(0x10_1000, 0x10_1fff, RWXFlags::ReadWrite, UGFlags::None);

    // Identity map the CLINT
    page_table.identity_map(0x200_0000, 0x200_b000, RWXFlags::ReadWrite, UGFlags::None);

    // Identity map the PLIC
    page_table.identity_map(0xc00_0000, 0xc00_2000, RWXFlags::ReadWrite, UGFlags::None);
    page_table.identity_map(0xc20_0000, 0xc20_2000, RWXFlags::ReadWrite, UGFlags::None);

    // Identity map UART
    page_table.identity_map(0x1000_0000, 0x1000_0000, RWXFlags::ReadWrite, UGFlags::None);

    // Identity map VirtIO
    page_table.identity_map(0x1000_1000, 0x1000_8000, RWXFlags::ReadWrite, UGFlags::None);
}