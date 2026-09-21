bits 64

global _start

section .text

_start:

    ; MOV
    mov rax, rbx
    mov rbx, rax
    mov r8, rax
    mov rax, r8

    mov eax, 123
    mov rax, 0x123456789abcdef0

    ; LEA
    lea rax, [rel data1]
    lea rbx, [rel data2]

    ; ADD
    add rax, rbx
    add r8, r9
    add eax, ebx
    add r8d, r9d

    ; SUB
    sub rax, rbx
    sub r8, r9
    sub eax, ebx
    sub r8d, r9d

    ; XOR
    xor rax, rbx
    xor r8, r9
    xor eax, ebx
    xor r8d, r9d
    xor rcx, rcx
    xor r8d, eax

    ; AND
    and rax, rbx
    and r8, r9
    and eax, ebx
    and r8d, r9d
    and rsp, -16
    and r8d, 15

    ; OR
    or rax, rbx
    or r8, r9
    or eax, ebx
    or r8d, r9d

    ; CMP
    cmp rax, rbx
    cmp r8, r9
    cmp eax, ebx
    cmp r8d, r9d

    ; TEST
    test rax, rax
    test r8, r8
    test eax, eax
    test r8d, r8d

    ; PUSH / POP
    push rax
    pop rax

    push rbx
    pop rbx

    push r8
    pop r8

    ; CALL
    call test_function

    ; JMP
    jmp after_jmp

before_jmp:
    nop

after_jmp:

    ; conditional jumps
    cmp rax, rbx

    je equal
    jne not_equal
    jl less
    jle less_equal
    jg greater
    jge greater_equal

equal:
    nop

not_equal:
    nop

less:
    nop

less_equal:
    nop

greater:
    nop

greater_equal:
    nop

    ; NOP
    nop

    ; ENDBR64
    endbr64

    ; SYSCALL
    mov eax, 60
    xor edi, edi
    syscall


test_function:
    push rbp
    mov rbp, rsp

    mov rax, 42

    pop rbp
    ret


section .rodata

data1:
    dq 0x1111111111111111

data2:
    dq 0x2222222222222222