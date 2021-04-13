.section .text
.global asm_trap_vector

asm_trap_vector:
    # The CPU was interrupted
	j asm_trap_vector

asm_trap_vector_s:
    # The CPU was interrupted
	j asm_trap_vector_s