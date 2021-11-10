.globl main

.globl _start
_start:
    call main
    li a7, 60
    ecall
2:
    j 2b