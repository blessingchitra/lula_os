#![allow(unused)]



use core::iter::empty;

use crate::sync::SpinLock;
#[macro_use]
use crate::{kprintln, kprint};

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

pub static UART_RX_BUFF: SpinLock<UartBuff> = SpinLock::new(UartBuff::new());
pub const UART_BUFF_SIZE: usize = 1024;

#[macro_export]
macro_rules! uartreg {
    ($reg:expr) => {
        unsafe { &mut *(($crate::uart::UART0 + $reg)  as *mut u8 ) }
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

#[derive(Debug)]
pub struct UartBuff {
    buffer: [u8; UART_BUFF_SIZE],
    rd:     usize,
    wt:     usize,
}


impl UartBuff {
    pub const fn new() -> Self {
        Self { 
            buffer: [0; UART_BUFF_SIZE], 
            rd:  0, 
            wt:  0 
        }
    }

    pub fn uart_getc(&self) -> Option<u8>
    {
        let can_read =  (uartrd!(LSR) & 0x01) != 0;
        if can_read {
            return Some(uartrd!(RHR));
        }
        None
    }

    /// (Non-Blocking) Write a character (bytes) the uart's *THR*
    /// ### Returns
    /// * `status` - Whether or not we wrote to the uart
    pub fn uart_putc(&self, c: u8) -> bool {
        let can_write = (uartrd!(LSR) & LSR_TX_IDLE) != 0;
        if can_write {
            uartwt!(THR, c);
            return true;
        }
        false
    }

    /// (Blocking) Write a character (bytes) the uart's *THR*
    /// ### Returns
    /// * `status` - Whether or not we wrote to the uart
    /// always returns true. We return bool just
    /// to keep the same interface as its non-blocking version
    pub fn uart_putc_block(&self, c: u8) -> bool {
        while (uartrd!(LSR) & LSR_TX_IDLE) == 0 {
            core::hint::spin_loop();
        }
        uartwt!(THR, c);
        true
    }

    /// If there is not data passed to the function 
    /// we send whatever is currently in the buffer
    pub fn send(&mut self, data: Option<&str>){
        match data {
            Some(data) => {
                if !data.is_empty() {
                    for char in data.as_bytes() {
                        self.uart_putc_block(*char);
                    }
                }
            },
            None => {
                while !(self.rd == self.wt) {
                    let c = self.buffer.get(self.rd).copied();
                    match c {
                        Some(val) => {
                            let processed = self.uart_putc_block(val);
                            if processed { self.pop(); } else { break }
                        },
                        None => break
                    }
                }
            }
        }
    }

    pub fn receive (&mut self){
        loop {
            let char = self.uart_getc();
            match char {
                Some(char) => {
                    let char = match char {
                        b'\r' => b'\n',
                        _ => char
                    };
                    if char == (8 | b'\x7f') { // backspace
                        self.push(b'\x08');
                        self.push(b' ');
                        self.push(b'\x08');
                    }else{
                        self.push(char);
                    }
                },
                None => break
            }
        }
    }

    fn push(&mut self, c: u8) {
        // TODO: double check off by one error
        let buff_full = self.rd.abs_diff(self.wt) == (UART_BUFF_SIZE - 1);
        if !buff_full {
            self.buffer[self.wt] = c;
            let nxt = (self.wt + 1) % UART_BUFF_SIZE;
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
            let char = self.buffer.get(self.rd).copied().unwrap();
            self.rd = (self.rd + 1) % UART_BUFF_SIZE;
        }
    }

    fn isempty(&self) -> bool {
        self.rd == self.wt
    }

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


#[export_name = "uart_putc_block"]
pub fn uart_putc_block(c: u8) -> bool {
    while (uartrd!(LSR) & LSR_TX_IDLE) == 0 {
        core::hint::spin_loop();
    }
    uartwt!(THR, c);
    true
}

pub fn uart_puts(s: &str) {
    for c in s.bytes() {
        uart_putc_block(c);
    }
}


#[export_name = "uart_isr"]
pub extern "C" fn uart_isr()
{
    let mut buff_empty = true;
    {
        let mut spinl_guard = UART_RX_BUFF.lock();
        let buff = spinl_guard.get_mut();
        buff.receive();
        buff.send(None);
        buff_empty = buff.isempty();
    }

    if buff_empty {
        uartwt!(IER, IER_RX_ENABLE)
    }else{
        uartwt!(IER, IER_RX_ENABLE | IER_TX_ENABLE)
    }
}

