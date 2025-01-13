#! /usr/bin/bash
if [ ! -d dbg ]; then
    mkdir dbg
fi

riscv64-unknown-elf-readelf -S -W kern/target/riscv64gc-unknown-none-elf/debug/kernel > dbg/bin-sections
riscv64-unknown-elf-objdump -D kern/target/riscv64gc-unknown-none-elf/debug/kernel > dbg/bin-disassembly.s
riscv64-unknown-elf-objdump -t kern/target/riscv64gc-unknown-none-elf/debug/kernel > dbg/bin-symbol-table
riscv64-unknown-elf-objdump -T kern/target/riscv64gc-unknown-none-elf/debug/kernel > dbg/bin-dynamic-symbol-table
riscv64-unknown-elf-nm kern/target/riscv64gc-unknown-none-elf/debug/kernel > dbg/bin-symbols

