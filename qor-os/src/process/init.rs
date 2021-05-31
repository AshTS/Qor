/// init process
pub fn init_proc()
{
    loop
    {
        unsafe 
        {
            asm!("li a7, 61");
            asm!("li a0, 0");
            asm!("ecall");
        }
    }
}