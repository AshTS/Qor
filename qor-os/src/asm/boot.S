
# Do not produce compressed instructions, this is for RISCV64GC
.option norvc

# Section which will be placed as 0x8000_0000 (The start location for qemu)
.section .text.init
.global _start
_start:

    # Load the global pointer
    .option push
    .option norelax
    la gp, _global_pointer
    .option pop

    # Make sure we are in machine mode
    csrw satp, zero

    # Make sure only hart 0 will boot
    csrr t0, mhartid
    # If we are not on hart 0, we will jump to an infinite waiting loop
    bnez t0, _start_wfi_loop

    # Clear the BSS section by writing 8 byte double words to it

    # Load the start and end pointers
    la a0, _bss_start
    la a1, _bss_end

    # If the bss section is empty
    bgeu a0, a1, _start_init

_start_zero_bss_loop:
    # Store double word
    sd zero, (a0)
    # Increment by 8
    addi a0, a0, 8
    # Jump back
    bltu a0, a1, _start_zero_bss_loop

_start_init:
    # Initialize the stack pointer
    la sp, _stack_end
    
    # Set up the machine status register
    li t0, 0b11 << 11 | (1 << 7) | (1 << 3)
    csrw mstatus, t0

    # Set the mret address to kmain
    la t1, kmain
    csrw mepc, t1

    # Set the trap vector to the proper address
    la t2, asm_trap_vector
    csrw mtvec, t2

    # Enable Interrupts
    li t3, (1 << 3) | (1 << 7) | (1 << 11)
    csrw mie, t3

    # Set up the return address to just loop when kmain returns
    la ra, _start_kmain_return:

    # Call kmain
    mret
_start_kmain_return:



_start_wfi_loop:
    # Loop forever
    wfi
    j 3b



