pub const CONS_BUFF_SIZE: usize = 1024;

pub struct Console {
    buffer: [u8; CONS_BUFF_SIZE]
}

impl Console {
    pub fn new() -> Self {
        Console { 
            buffer: [0u8; CONS_BUFF_SIZE] 
        }
    }
}