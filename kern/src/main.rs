#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]

use kernel::*;
use crate::riscv::Register; 

const NCPU: u32         = 2;
const CSTACKSIZE: usize = (NCPU * (1024 * 4)) as usize; // cpu stack size
const PGSIZE:     usize = 4096;
const PAGESHIFT:  usize = 12;
const MAXVA:      usize = 1 << (9 + 9 + 9 + 12 - 1);

const TRAPFRAME:  u64 = (MAXVA - PGSIZE) as u64;

#[allow(non_upper_case_globals)]
#[no_mangle]
static mut stack0: [u8; CSTACKSIZE] = [0; CSTACKSIZE];

#[export_name = "kern_exec"]
pub unsafe extern "C" fn kern_exec() -> ! {
    let first = riscv::RegTP::read() == 0;
    if first {
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

    }

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
