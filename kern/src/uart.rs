#![allow(unused)]

pub const RHR             :usize = 0;                   // receive holding register (for input bytes)
pub const THR             :usize = 0;                   // transmit holding register (for output bytes)
pub const IER             :usize = 1;                   // interrupt enable register
pub const IER_RX_ENABLE   :u8    = 1 << 0;
pub const IER_TX_ENABLE   :u8    = 1 << 1;
pub const FCR             :usize = 2;                   // FIFO control register
pub const FCR_FIFO_ENABLE :u8    = 1 << 0;
pub const FCR_FIFO_CLEAR  :u8    = 3 << 1;              // clear the content of the two FIFOs
pub const ISR             :usize = 2;                   // interrupt status register
pub const LCR             :usize = 3;                   // line control register
pub const LCR_EIGHT_BITS  :u8    = 3 << 0;
pub const LCR_BAUD_LATCH  :u8    = 1 << 7;              // special mode to set baud rate
pub const LSR             :usize = 5;                   // line status register
pub const LSR_RX_READY    :u8    = 1 << 0;              // input is waiting to be read from RHR
pub const LSR_TX_IDLE     :u8    = 1 << 5;              // THR can accept another character to send

pub const UART0           : usize = 0x10000000;
pub const UART0_IRQ       : u8 = 10;

#[macro_export]
macro_rules! uartreg {
    ($reg:expr) => {
        unsafe { &mut *($crate::uart::UART0 as *mut u8 ).add($reg) }
    };
    // used for testing
    ($reg:expr, $mock_mem:expr) => {
        &mut $mock_mem[$reg]
    }
}

#[macro_export]
macro_rules! uartrd {
    ($reg:expr) => {
        *uartreg!($reg) 
    };
    // used for testing
    ($reg:expr, $mock_mem:expr) => {
        *uartreg!($reg, $mock_mem)
    };
}

#[macro_export]
macro_rules! uartwt{
    ($reg:expr, $val:expr) => {
        *uartreg!($reg) = $val 
    };
    // used for testing
    ($reg:expr, $mock_mem:expr, $val:expr) => {
        *uartreg!($reg, $mock_mem) = $val
    };
} 

pub fn uart_init()
{
    uartwt!(IER, 0x00);
    uartwt!(LCR, LCR_BAUD_LATCH);

    uartwt!(0, 0x03);   // least significant byte for baud rate = 38.4k
    uartwt!(1, 0x00);   // most  significant byte for baud rate = 38.4k

    // exit `set-baud` mode
    // set word-length to 8bits with no parity
    uartwt!(LCR, LCR_EIGHT_BITS);

    // reset and enable FIFOs
    uartwt!(FCR, FCR_FIFO_ENABLE | FCR_FIFO_CLEAR);

    // enable transmit and receive interrupts
    uartwt!(IER, IER_RX_ENABLE);
}

pub fn uart_putc(c: u8) {
    while (uartrd!(LSR) & LSR_TX_IDLE) == 0 {
        core::hint::spin_loop();
    }
    uartwt!(THR, c);
}

pub fn uart_puts(s: &str) {
    for c in s.bytes() {
        uart_putc(c);
    }
}

pub fn uart_getc() -> Option<u8>
{
    let can_read =  (uartrd!(LSR) & 0x01) != 0;
    if can_read {
        return Some(uartrd!(RHR));
    }
    None
}

pub fn uart_isr()
{
    loop {
        let char = uart_getc();
        match char {
            Some(char) => {
                uart_putc(char);
                uart_putc(('\n' as u8));
            },
            None => break,
        }
    }
}



















