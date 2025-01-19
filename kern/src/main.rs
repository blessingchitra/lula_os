#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]


const NCPU: u32         = 2;
const CSTACKSIZE: usize = (NCPU * 1024) as usize; // cpu stack size
const PGSIZE:     usize = 4096;
const PAGESHIFT:  usize = 12;
const MAXVA:      usize = 1 << (9 + 9 + 9 + 12 - 1);

const TRAPFRAME:  u64 = (MAXVA - PGSIZE) as u64;

#[allow(non_upper_case_globals)]
#[no_mangle]
static mut stack0: [u8; CSTACKSIZE] = [0; CSTACKSIZE];

mod plic;
mod uart;
mod riscv;
mod ktrap;
mod console;

#[export_name = "start"]
pub extern "C" fn start()
{
    let mut x = riscv::r_mstatus();
    x &= !riscv::MSTATUS_MPP_MASK  as u64;
    x |= riscv::MSTATUS_MPP_S      as u64;
    riscv::w_mstatus(x);

    riscv::w_mepc((_kmain as *const ()) as u64);

    riscv::w_satp(0); // no paging yet
    riscv::w_stvec((ktrap::_ktrap as * const()) as u64);

    riscv::w_medeleg(0xffff);
    riscv::w_mideleg(0xffff);

    let mut intr = riscv::r_sie(); 
    intr |= riscv::SIE_SEIE as u64;
    intr |= riscv::SIE_SSIE as u64;
    intr |= riscv::SIE_STIE as u64; 

    riscv::w_sie(intr);
    riscv::intr_on();
    
    // supervisor mode has access to all memory
    riscv::w_pmpaddr0(0x3fffffffffffff);
    riscv::w_pmpcfg0(0xf);

    plic::plic_init(0);

    unsafe { core::arch::asm!("mret") };
}


#[export_name = "_kmain"]
pub unsafe extern "C" fn _kmain() -> ! {
    let os = r#"
| |         | |        / __ \ / ____|
| |    _   _| | __ _  | |  | | (___  
| |   | | | | |/ _` | | |  | |\___ \ 
| |___| |_| | | (_| | | |__| |____) |
|______\__,_|_|\__,_|  \____/|_____/ 
-------------------------------------
    "#;
    uart::uart_init();
    uart::uart_puts(os); uart::uart_puts("\n");
    // let mut cons = console::Console::new();
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
