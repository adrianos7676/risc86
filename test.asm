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
    xor rax, rax
    jmp uncontidional_jump
    mov rax, 60
    mov rdi, 5
    syscall

uncontidional_jump:
    mov rax, 60
    mov rdi, 20
    syscall