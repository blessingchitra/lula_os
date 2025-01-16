use crate::{plic_sclaim_r, plic_sclaim_w};
use crate::riscv::{r_scause, r_sepc, w_sepc};
use crate::uart::{uart_isr, uart_puts, UART0_IRQ};

#[naked]
#[export_name = "_ktrap"]
pub unsafe extern "C" fn _ktrap()
{
    unsafe {
        core::arch::naked_asm!(
            "
            addi sp, sp, -256
            sd ra, 0(sp)
            sd sp, 8(sp)
            sd gp, 16(sp)
            sd tp, 24(sp)
            sd t0, 32(sp)
            sd t1, 40(sp)
            sd t2, 48(sp)
            sd a0, 72(sp)
            sd a1, 80(sp)
            sd a2, 88(sp)
            sd a3, 96(sp)
            sd a4, 104(sp)
            sd a5, 112(sp)
            sd a6, 120(sp)
            sd a7, 128(sp)
            sd t3, 216(sp)
            sd t4, 224(sp)
            sd t5, 232(sp)
            sd t6, 240(sp)

            call ktrap_isr

            ld ra, 0(sp)
            ld sp, 8(sp)
            ld gp, 16(sp)
            ld t0, 32(sp)
            ld t1, 40(sp)
            ld t2, 48(sp)
            ld a0, 72(sp)
            ld a1, 80(sp)
            ld a2, 88(sp)
            ld a3, 96(sp)
            ld a4, 104(sp)
            ld a5, 112(sp)
            ld a6, 120(sp)
            ld a7, 128(sp)
            ld t3, 216(sp)
            ld t4, 224(sp)
            ld t5, 232(sp)
            ld t6, 240(sp)
            addi sp, sp, 256
            sret
            "
        );
    };
}

#[export_name = "ktrap_isr"]
pub extern "C" fn ktrap_isr()
{
    let sepc      = r_sepc();
    let cause     = r_scause();
    let is_intr  = cause >> 63 != 0;
    let code      = cause & 0xffff;
    let intr_id   = plic_sclaim_r!(0);

    #[allow(unused_variables)]
    if is_intr {
        match code {
            1 => uart_puts("--Software Intr\n"),
            5 => uart_puts("--Timer Intr\n"),
            9 => {
                let uart_intr = UART0_IRQ as u32;
                match intr_id {
                    uart_intr => uart_isr(),
                    // _  => uart_puts("----unknown dev intr\n"),
                }
            },
            _ => uart_puts("--Unkwown Intr\n"),
        }
    }else {
        uart_puts("An Exception Occured\n");
        match code {
            0 => uart_puts("Instruction address misaligned"),
            1 => uart_puts("Instruction access fault"),
            2 => uart_puts("Illegal instruction"),
            _ => uart_puts("Unknown/unhandled exception"),
        }
    }
    plic_sclaim_w!(0, intr_id);
    w_sepc(sepc);
}