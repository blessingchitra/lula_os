## RISCV Tooling
compiler  - `riscv64-unknown-elf-gcc -mabi=ilp32 -march=rv32i -S main.c -o main.s` \
assembler - `riscv64-unknown-elf-as -mabi=ilp32 -march=rv32i main.s -o main.o ` \
linker    - `riscv64-unknown-elf-ld -m elf32lriscv main.o func.o -o main.x`  \
symbools  - `riscv64-unknown-elf-nm main.o` \
disassembler - `riscv64-unknown-elf-objdump -D main.o` \
readelf - `riscv64-unknown-elf-readelf -h main.x`