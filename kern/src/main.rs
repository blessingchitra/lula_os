#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(custom_test_frameworks)]

use kernel::*;
use crate::riscv::Register; 
use crate::virtm;
use crate::usr;

const NCPU: u32         = 2;
const CSTACKSIZE: usize = (NCPU * (1024 * 1024 * 4)) as usize; // cpu stack size

#[allow(non_upper_case_globals)]
#[no_mangle]
static mut stack: [u8; CSTACKSIZE] = [0; CSTACKSIZE];


#[unsafe(no_mangle)]
pub unsafe extern "C" fn kern_exec() -> ! {
    let cpu_first = riscv::RegTP::read() == 0;
    if cpu_first {
        let kern_end = virtm::get_data_end();

        kprintln!("System Initialised.");
        kprintln!("Kern End: {:#x}, VA Max: {:#x}", kern_end, virtm::MEM_MAX);
        usr::usr_load_and_exec();
    }

    loop {
        1;
    }
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> !{
    let message =  info.message();
    if let Some(message) = message.as_str() {
        kprintln!("{}", message);
    }
    let loc = info.location();
    if let Some(loc) = loc {
        kprint!("file: {}, line: {}", loc.file(), loc.line())
    }
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

    #[cfg(test)]
    mod test {
    }
}
