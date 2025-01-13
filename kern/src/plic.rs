use crate::uart::UART0_IRQ;

pub const PLIC          : usize = 0x0c000000;
pub const PLIC_PRIORITY : usize = PLIC + 0x0;
pub const PLIC_PENDING  : usize = PLIC + 0x1000;

#[macro_export]
macro_rules! plic_enable {
    ($hart:expr, $value:expr) => {{
        unsafe {
            let enable_ptr = 
                (($crate::plic::PLIC + 0x2080) + ($hart * 0x100)) 
                    as *mut u32;
            *enable_ptr = $value;
        }
    }};
}

#[macro_export]
macro_rules! plic_spriority {
    ($hart:expr, $value:expr) => {{
        unsafe{
            let priority_ptr = 
                (($crate::plic::PLIC + 0x201000) + ($hart * 0x2000)) 
                    as *mut u32;
            *priority_ptr = $value;
        }
    }};
}


#[macro_export]
macro_rules! plic_sclaim_r {
    ($hart:expr) => {{
        unsafe {
            let sclaim_ptr = 
                ($crate::plic::PLIC + 0x201004 + ($hart * 0x2000)) 
                    as *const u32;
            *sclaim_ptr
        }
    }};
}

#[macro_export]
macro_rules! plic_sclaim_w {
    ($hart:expr, $value:expr) => {{
        unsafe {
            let sclaim_ptr = 
                ($crate::plic::PLIC + 0x201004 + ($hart * 0x2000)) 
                    as *mut u32;
            *sclaim_ptr = $value;
        }
    }};
}


pub fn plic_init(hart: usize)
{
    unsafe {
        // set desired IRQ priorities non-zero (otherwise disabled).
        let ptr = 
            (PLIC + (UART0_IRQ as usize * 4usize)) as *mut u32;
        *ptr = 1;
    };
    plic_enable!(hart, 1 << UART0_IRQ);
    plic_spriority!(hart, 0);
}