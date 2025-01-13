#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]

use kernel::*;

const NCPU: u32         = 2;
const CSTACKSIZE: usize = (NCPU * 1024) as usize; // cpu stack size
const PGSIZE:     usize = 4096;
const PAGESHIFT:  usize = 12;
const MAXVA:      usize = 1 << (9 + 9 + 9 + 12 - 1);

const TRAPFRAME:  u64 = (MAXVA - PGSIZE) as u64;

#[allow(non_upper_case_globals)]
#[no_mangle]
static mut stack0: [u8; CSTACKSIZE] = [0; CSTACKSIZE];

use plic::plic_init;
use riscv::*;
use uart::{uart_init, uart_puts, UART0_IRQ};

#[export_name = "start"]
pub extern "C" fn start()
{
    let mut x = r_mstatus();
    x &= !MSTATUS_MPP_MASK  as u64;
    x |= MSTATUS_MPP_S      as u64;
    w_mstatus(x);

    w_mepc((_kmain as *const ()) as u64);

    w_satp(0); // no paging yet
    w_stvec((ktrap as * const()) as u64);

    w_medeleg(0xffff);
    w_mideleg(0xffff);

    let mut intr = r_sie(); 
    intr |= SIE_SEIE as u64;
    intr |= SIE_SSIE as u64;
    intr |= SIE_STIE as u64; 

    w_sie(intr);
    intr_on();
    
    // supervisor mode has access to all memory
    w_pmpaddr0(0x3fffffffffffff);
    w_pmpcfg0(0xf);

    plic_init(0);

    unsafe { core::arch::asm!("mret") };
}


#[export_name = "ktrap"]
pub extern "C" fn ktrap()    
{
    let cause    = r_scause();
    let is_intr = cause >> 63 != 0;
    let code     = cause & 0xffff;

    uart_puts("\n");
    if is_intr {
        uart_puts("An Interrupt Occured\n");
        match code {
            1 => uart_puts("--Software Intr\n"),
            5 => uart_puts("--Timer Intr\n"),
            9 => {
                uart_puts("--External Intr\n");
                let intr_id   = plic_sclaim_r!(0);
                let uart_intr = UART0_IRQ as u32;
                match intr_id {
                    uart_intr => uart_puts("----UART0 INTR\n"),
                    _         => uart_puts("----unknown dev intr\n"),
                }
                plic_sclaim_w!(0, intr_id);
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
    uart_puts("Trap handler exiting\n");
    loop {
        1;
    }
}

#[export_name = "_kmain"]
pub unsafe extern "C" fn _kmain() -> ! {
    uart_init();
    uart_puts("--------------------------\n");
    uart_puts("--------------------------\n");
    uart_puts("Hello friend,\n");
    uart_puts("--------------------------\n");
    uart_puts("--------------------------\n");
    loop {
        1;
    }
}

#[allow(unused_variables)]
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> !{
    loop {
        1;
    }
}

#[cfg(test)]
mod test {

    #[allow(unused_variables)]
    pub fn test_runner(tests: &[&dyn Fn()]) -> ! {
        loop {
            1;
        }
    }

    #[macro_export]
    macro_rules! hades_test {
        (fn $name:ident() { $($tt:tt)* }) => {
            #[test_case]
            fn $name() {
                $crate::debug_print!("{}...", stringify!($name));
                {
                    $($tt)*
                };
                $crate::debug_println!("[ok]");
            }
        };
    }
    #[cfg(test)]
    mod test {
    }
}
