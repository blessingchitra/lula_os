#![allow(unused)]

extern crate kernel;
use kernel::*;
use uart::RHR;

#[derive(Debug)]
pub struct Memory
{
    mem:  Vec<u8>,
}

impl Memory
{
    pub fn new (size: usize) -> Self {
        Self { 
            mem: vec![0; size as usize] 
        }
    }

    fn write(&mut self, pos: usize, value: u8)
    {
        if pos >= self.mem.len() { return; }
        self.mem[pos] = value;
    }

    fn read(&self, pos: usize) -> Option<u8>
    {
        if pos >= self.mem.len() { return None; }
        Some(self.mem[pos])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mem_modify_rhr_using_indices()
    {
        let mut memory = Memory::new(8);
        memory.write(RHR as usize, 0x05);
        let val = memory.read(RHR as usize).unwrap_or(0);
        assert_eq!(val, 0x05);
    }

    #[test]
    fn mem_modify_rhr_using_macros()
    {
        let mut memory = Memory::new(8);
        uartwt!(RHR as usize, memory.mem, 0x05);
        let val = uartrd!(RHR as usize, memory.mem);
        assert_eq!(val, 0x05);
    }
}



























