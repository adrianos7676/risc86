global _start

section .text
_start:
    mov rax, 1
    mov rdi, 1
    lea rsi, [rel message]
    mov rdx, 30
    syscall

    mov rax, 60
    xor rdi, rdi
    syscall

section .data
message:
    db "Hello world from x86 on RiscV", 10