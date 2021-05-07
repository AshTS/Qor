#![no_std]
#![no_main]

#[no_mangle]
extern "C"
fn _start()
{
    slib::write("Hello From Userland!\n");
    slib::exit(4);
}