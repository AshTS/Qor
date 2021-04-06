.section .text
.global asm_trap_vector

asm_trap_vector:
    # The CPU was interrupted
    mret