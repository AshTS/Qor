.section .text
.global asm_trap_vector

asm_trap_vector:
    # The CPU was interrupted
    csrr	a0, mtval
	wfi
	j asm_trap_vector