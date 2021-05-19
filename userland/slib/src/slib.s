.globl main

.globl _start
_start:
    call main
    li a7, 60
    ecall
2:
    j 2b


.globl syscall
syscall:
    mv a7, a0
    mv a0, a1
    mv a1, a2
    mv a2, a3
    mv a3, a4
    mv a4, a5
    mv a5, a6
    ecall
    ret

.globl put
put:
    li a7, 10
    ecall
    ret

.globl open
open:
    li a7, 2
    ecall
    ret

.globl close
close:
    li a7, 3
    ecall
    ret