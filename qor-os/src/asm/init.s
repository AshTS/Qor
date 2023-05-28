# Do not produce compressed instructions
.option norvc

.section .text
.global init_proc_location
init_proc_location:
    li a0, 0
    li a7, 61
    # ecall
    j init_proc_location