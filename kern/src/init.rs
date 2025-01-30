use core::arch::naked_asm;

#[naked]
#[link_section=".init"]
#[export_name ="_entry_2"]
pub unsafe extern "C" fn _entry()
{
    naked_asm!(
        "
        la sp, stack0
        li a0, 1024*4           # 4KB stack
        csrr a1, mhartid
        addi a1, a1, 1
        mul a0, a0, a1
        add sp, sp, a0
        call sys_init
        1:
            j 1b
        "
    );
}