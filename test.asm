bits 64

global _start

section .text
_start:
    ; 64-bit: RAX * RCX -> RDX:RAX
    mov rax, 0x123456789abcdef0
    mov rcx, 0x1111111111111111
    mul rcx

    ; 32-bit: EAX * ECX -> EDX:EAX
    mov eax, 0x12345678
    mov ecx, 0x11111111
    mul ecx

    ; exit
    mov eax, 60
    xor edi, edi
    syscall