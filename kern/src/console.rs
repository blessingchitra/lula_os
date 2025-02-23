use core::fmt::Write;
use crate::sync::{SpinLock, SpinLockGuard};
use crate::uart;

pub const CONS_BUFF_SIZE: usize = 1024;

pub struct KConsole <'a> {
    buffer: &'static SpinLock<uart::UartBuff>,
    spinl_guard: SpinLockGuard<'a, uart::UartBuff>
}

impl <'a> KConsole <'a> {
    pub fn new(buffer: &'static SpinLock<uart::UartBuff>) -> Self {
        let spinl_guard = buffer.lock();
        KConsole{ buffer, spinl_guard}
    }
}

pub struct UConsole { }

impl <'a> Write for KConsole<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if !s.is_empty() {
            let buff = self.spinl_guard.get_mut();
            buff.send(Some(s));
            return Ok(());
        }
        Err(core::fmt::Error)
    }
}

#[macro_export]
macro_rules! kprint{
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut cons = crate::console::KConsole::new(&crate::uart::UART_RX_BUFF);
        let _ = write!(&mut cons, $($arg)*);
    }};
}

#[macro_export]
macro_rules! kprintln {
    () => { kprint!("\n"); };
    ($($arg:tt)*) => {{
        crate::kprint!($($arg)*);
        crate::kprint!("\n");
    }};
}   

