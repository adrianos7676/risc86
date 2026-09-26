nasm -f elf64 test.asm -o test.o
ld -nostdlib test.o -o test