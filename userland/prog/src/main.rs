#![no_std]
#![no_main]

#[no_mangle]
extern "C"
fn _start()
{
    slib::exit(5);
}