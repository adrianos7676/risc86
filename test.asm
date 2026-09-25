global _start

section .text

_start:
    xor r8d, r8d
    xor r9d, r9d

    test r8, r9
    je equal

    mov rax, 60
    mov rdi, 1
    syscall

equal:
    mov rax, 60
    mov rdi, 69
    syscall