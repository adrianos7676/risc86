global _start

section .text
_start:

    ; ========================================
    ; CMP + JE
    ; ZF = 1 -> branch TAKEN
    ; ========================================

    mov rax, 10
    mov rbx, 10

    cmp rax, rbx
    je .equal

.equal:

    ; ========================================
    ; exit(0)
    ; ========================================

    mov eax, 60
    xor edi, edi
    syscall