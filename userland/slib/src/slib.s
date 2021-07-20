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

.globl read
read:
    li a7, 0
    ecall
    ret

.globl write
write:
    li a7, 1
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

.globl mmap
mmap:
    li a7, 9
    ecall
    ret

.globl munmap
munmap:
    li a7, 11
    ecall
    ret

.globl fork
fork:
    li a7, 57
    ecall
    ret

.globl execve
execve:
    li a7, 59
    ecall
    ret

.globl exit
exit:
    li a7, 60
    ecall
    ret

.globl wait
wait:
    li a7, 61
    ecall
    ret

.globl getdents
getdents:
    li a7, 78
    ecall
    ret

.globl getcwd
getcwd:
    li a7, 79
    ecall
    ret

.globl chdir
chdir:
    li a7, 80
    ecall
    ret