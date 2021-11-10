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

.globl lseek
lseek:
    li a7, 8
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

.globl sigaction
sigaction:
    li a7, 13
    ecall
    ret

.globl sigreturn
sigreturn:
    li a7, 15
    ecall
    ret


.globl ioctl
ioctl:
    li a7, 16
    ecall
    ret

.globl pipe
pipe:
    li a7, 22
    ecall
    ret

.globl dup
dup:
    li a7, 32
    ecall
    ret

.globl dup2
dup2:
    li a7, 33
    ecall
    ret

.globl pause
pause:
    li a7, 34
    ecall
    ret

.globl nanosleep
nanosleep:
    li a7, 35
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
    exit_loop:
    j exit_loop

.globl wait
wait:
    li a7, 61
    ecall
    ret

.globl kill
kill:
    li a7, 62
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

.globl mkdir
mkdir:
    li a7, 83
    ecall
    ret

.globl sync
sync:
    li a7, 162
    ecall
    ret