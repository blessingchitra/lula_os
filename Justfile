
#alias b := build
alias r := run

kernel_path := "./kern/target/riscv64gc-unknown-none-elf/debug/kernel"

kernel:
	cd kern && cargo build -Z build-std=core,alloc \
	 --target riscv64gc-unknown-none-elf 

test:
	cd sim && cargo test --verbose

qemu_args := "-M virt -m 2G -nographic"

raw_run *EXTRA_ARGS:
	qemu-system-riscv64 {{EXTRA_ARGS}} {{qemu_args}}

run:
	qemu-system-riscv64 \
	-machine virt -bios none \
	-kernel {{kernel_path}} -m 128M -smp 1 -nographic \
	-d int,guest_errors -D qemu.log


run-gdb:
	qemu-system-riscv64 \
	-machine virt -bios none \
	-kernel {{kernel_path}} -m 128M -smp 1 -nographic \
	-d int,guest_errors -D qemu.log \
	-S -s


_krun kernel *EXTRA_ARGS:
	cd kern && qemu-system-riscv64 {{EXTRA_ARGS}} {{qemu_args}} -kernel {{kernel}}

# debug: (run "-gdb tcp::1234 -S")
gdb:
	gdb {{kernel_path}} \
		-ex 'target remote localhost:1234'
