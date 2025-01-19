#![allow(unused)]

use core::iter::empty;

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

static mut uart_rx_buff: UartBuff = UartBuff::new();

struct UartBuff {
    buffer: [u8; 20],
    tx_ier: bool,
    len:    usize,
    rd:     usize,
    wt:     usize,
}


impl UartBuff {
    pub const fn new() -> Self {
        Self { 
            buffer: [0; 20], 
            tx_ier: false,
            len: 20,
            rd:  0, 
            wt:  0 
        }
    }

    // uart receive buff can have a read function 
    fn read (&mut self) -> Option<u8> {
        if(self.rd < self.wt) {
            let val = Some(self.buffer[self.rd]);
            self.rd = ((self.rd + 1) % self.len);
            return val; 
        }
        None
    }

    // uart transmit buff can have a write function 
    fn write(&mut self, value: u8) {
        let nxt = (self.wt + 1) % self.len;
        if((nxt != self.rd)){
            self.buffer[self.wt] = value;
            self.wt = nxt;
        }
        // here we've wrapped around and are about
        // to run into data overrun issues
    }

    fn push(&mut self, c: u8) {
        // TODO: double check off by one error
        let buff_full = self.rd.abs_diff(self.wt) == (self.len - 1);
        if !buff_full {
            let nxt = (self.wt + 1) % self.len;
            self.buffer[self.wt] = c;
            self.wt = nxt;
        }
    }

    fn get(&mut self) -> Option<u8> {
        if !self.isempty() {
            let val = self.buffer.get(self.rd).copied();
            return val;
        }
        None
    }

    fn pop(&mut self) {
        if !self.isempty() {
            self.rd = (self.rd + 1) % self.len;
        }
    }

    fn isempty(&self) -> bool {
        self.rd == self.wt
    }

}


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
    uartwt!(IER, IER_RX_ENABLE | IER_TX_ENABLE);
}

pub fn uart_putc(c: u8) -> bool {
    let can_write = (uartrd!(LSR) & LSR_TX_IDLE) != 0;
    if can_write {
        uartwt!(THR, c);
        return true;
    }
    false
}

/// always returns true. returns bool just
/// to keep the same interface as its non-blocking version
pub fn uart_putc_block(c: u8) -> bool {
    while (uartrd!(LSR) & LSR_TX_IDLE) == 0 {
        core::hint::spin_loop();
    }
    uartwt!(THR, c);
    true
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
    let mut received = false;

    // we try to get the available data
    loop {
        let buff_full =  false;
        if !buff_full {
            let char = uart_getc();
            match char {
                Some(char) => {
                    received = true;
                    let mut chr = char;
                    if char == ('\r' as u8) {
                        chr = '\n' as u8;
                    }
                    unsafe { uart_rx_buff.push(chr); };
                },
                None => break,
            }
        }
        let c = unsafe { uart_rx_buff.get() };
        match c {
            Some(val) => {
                let wrote = uart_putc(val);
                if wrote { unsafe {uart_rx_buff.pop();}; } else {break;}
            },
            None => {}
        }
    }
}



















