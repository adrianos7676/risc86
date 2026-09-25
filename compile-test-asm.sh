nasm -f elf64 test.asm -o test.o
/opt/homebrew/bin/x86_64-linux-gnu-ld -nostdlib test.o -o test