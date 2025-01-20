use crate::uart;
use core::fmt::Write;

pub const CONS_BUFF_SIZE: usize = 1024;

pub struct KConsole;

pub struct UConsole {
    buffer: [u8; CONS_BUFF_SIZE]
}


impl Write for KConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if !s.is_empty() {
            uart::uart_puts(s);
            return Ok(());
        }
        Err(core::fmt::Error)
    }
}

#[macro_export]
macro_rules! println {
    ($($val:tt)*) => {{
        use core::fmt::Write;
        let mut cons = crate::console::KConsole;
        let _ = write!(&mut cons, $($val)*);
    }};
}