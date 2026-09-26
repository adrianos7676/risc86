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
    call first_call

    mov rax, 60
    mov rdi, 20
    syscall

first_call:
    call second_call
    ret

second_call:
    mov rax, 60
    mov rdi, 42
    syscall