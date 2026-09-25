bits 64

global _start

section .text

_start:
    xor r8d, r8d
    xor r9d, r9d

    cmp r8d, r9d
    je equal

    mov rax, 60
    mov rdi, 0
    syscall

equal:
    mov rax, 60
    mov rdi, 69
    syscall